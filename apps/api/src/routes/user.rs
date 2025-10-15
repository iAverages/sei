use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use axum::extract::State;
use axum::response::sse::Event;
use axum::response::Sse;
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use chrono::{Duration as ChronoDuration, Utc};
use futures::Stream;
use serde::Serialize;
use serde_json::json;

use crate::helpers::json_response;
use crate::mal::get_mal_user_list;
use crate::models::anime::{get_anime_relations, get_released_animes_by_id};
use crate::models::anime_users::{
    get_animes_for_user, get_user_list_entries, update_watch_priority, WatchPriorityUpdate,
};
use crate::models::user::{is_user_importing, update_list_entries_mal, DBUser, SafeUser};
use crate::AppState;

#[axum::debug_handler]
pub async fn get_user(Extension(user): Extension<DBUser>) -> impl IntoResponse {
    let safe_user: SafeUser = user.into();
    json_response!(StatusCode::OK, safe_user)
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
    let importing = is_user_importing(&state.db, &user_id).await;

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
