use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use axum::extract::State;
use axum::http::header::RETRY_AFTER;
use axum::response::sse::Event;
use axum::response::Sse;
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use chrono::{Duration as ChronoDuration, Utc};
use futures::Stream;
use serde::Serialize;
use serde_json::json;

use crate::helpers::json_response;
use crate::models::anime::{get_anime_relations, get_released_animes_by_id};
use crate::models::anime_users::{
    get_animes_for_user, get_user_list_entries, update_watch_priority, WatchPriorityUpdate,
};
use crate::models::user::{is_user_importing, update_list_entries_mal, DBUser, SafeUser};
use crate::AppState;

#[derive(Clone)]
pub struct MalRefreshLimiter {
    buckets: Arc<Mutex<HashMap<String, MalRefreshBucket>>>,
    capacity: f64,
    refill_interval: Duration,
}

struct MalRefreshBucket {
    tokens: f64,
    last_refill: Instant,
}

impl MalRefreshLimiter {
    pub fn new(capacity: u32, refill_interval: Duration) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            capacity: f64::from(capacity),
            refill_interval,
        }
    }

    fn reserve(&self, user_id: &str) -> Result<(), Duration> {
        self.reserve_at(user_id, Instant::now())
    }

    fn reserve_at(&self, user_id: &str, now: Instant) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().expect("MAL refresh limiter poisoned");
        let bucket = buckets
            .entry(user_id.to_owned())
            .or_insert(MalRefreshBucket {
                tokens: self.capacity,
                last_refill: now,
            });

        let elapsed = now.duration_since(bucket.last_refill);
        bucket.tokens = (bucket.tokens
            + elapsed.as_secs_f64() / self.refill_interval.as_secs_f64())
        .min(self.capacity);
        bucket.last_refill = now;

        if bucket.tokens < 1.0 {
            return Err(self.refill_interval.mul_f64(1.0 - bucket.tokens));
        }

        bucket.tokens -= 1.0;
        Ok(())
    }
}

#[axum::debug_handler]
pub async fn get_user(Extension(user): Extension<DBUser>) -> impl IntoResponse {
    let safe_user: SafeUser = user.into();
    json_response!(StatusCode::OK, safe_user)
}

#[axum::debug_handler]
pub async fn get_import_status(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> impl IntoResponse {
    match is_user_importing(&state.db, &user.id).await {
        Ok(is_importing) => json_response!(StatusCode::OK, { "isImporting": is_importing }),
        Err(error) => {
            tracing::error!(
                error = error.to_string(),
                "failed to get user import status"
            );
            json_response!(StatusCode::INTERNAL_SERVER_ERROR, {
                "message": "Failed to get import status"
            })
        }
    }
}

#[axum::debug_handler]
pub async fn refresh_mal_list(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> impl IntoResponse {
    if let Err(remaining) = state.mal_refresh_limiter.reserve(&user.id) {
        let retry_after = remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0);
        let retry_minutes = retry_after.div_ceil(60);
        let minute_label = if retry_minutes == 1 {
            "minute"
        } else {
            "minutes"
        };
        return (
            StatusCode::TOO_MANY_REQUESTS,
            [(RETRY_AFTER, retry_after.to_string())],
            Json(json!({
                "message": format!("Try again in {} {}", retry_minutes, minute_label)
            })),
        )
            .into_response();
    }

    match update_list_entries_mal(state, user).await {
        Ok(()) => json_response!(StatusCode::OK, { "message": "MAL anime list refreshed" }),
        Err(error) => {
            tracing::error!(
                error = error.to_string(),
                "failed to refresh MAL anime list"
            );
            json_response!(StatusCode::BAD_GATEWAY, {
                "message": "Failed to refresh MAL anime list"
            })
        }
    }
}

#[derive(Serialize)]
struct SingleEntry {
    anime_id: u32,
    watch_status: String,
    watch_priority: u32,
}

#[axum::debug_handler]
pub async fn get_list(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> impl IntoResponse {
    let user_id = user.id.clone();

    let now = Utc::now().naive_utc();
    let five_minutes_ago = now - ChronoDuration::minutes(5);

    if user.list_last_update < five_minutes_ago {
        let user = user.clone();
        let state = state.clone();
        tokio::spawn(async move {
            let _ = update_list_entries_mal(state, user).await;
        });
    }

    // TODO: handle errors correctly
    let anime_ids = get_animes_for_user(&state.db, user_id.as_str())
        .await
        .unwrap();
    let animes = get_released_animes_by_id(&state.db, &anime_ids)
        .await
        .unwrap();
    let list_entries = get_user_list_entries(&state.db, user_id.as_str())
        .await
        .unwrap();
    let anime_relations = get_anime_relations(&state.db, &anime_ids).await.unwrap();
    let importing = is_user_importing(&state.db, &user_id).await.unwrap_or(true);

    json_response!(StatusCode::OK, {
        "animes": animes,
        "list_entries": list_entries,
        "relations": anime_relations,
        "isImporting": importing
    })
}

#[axum::debug_handler]
pub async fn update_list_order(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Json(data): Json<WatchPriorityUpdate>,
) -> impl IntoResponse {
    update_watch_priority(&state.db, user.id, data).await;
    StatusCode::CREATED
}

#[axum::debug_handler]
pub async fn join_sse(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.tx.subscribe();

    let stream = BroadcastStream::new(rx)
        .map(move |msg| match msg {
            Ok(event) => {
                if event.user_id == user.id {
                    Some(Event::default().json_data(event).unwrap())
                } else {
                    None
                }
            }
            Err(e) => {
                tracing::error!("sse stream error: {}", e);
                None
            }
        })
        .filter_map(|res| res.map(Ok))
        .throttle(Duration::from_millis(50));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}
