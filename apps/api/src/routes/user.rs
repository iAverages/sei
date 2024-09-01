use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::json;

use crate::helpers::json_response;
use crate::importer::{AnimeUserEntry, AnimeWatchStatus};
use crate::mal::get_mal_user_list;
use crate::models::anime::get_released_animes_by_id;
use crate::models::anime_users::{
    get_user_entrys, link_user_to_anime, update_watch_priority, DBAnimeUser, WatchPriorityUpdate,
};
use crate::models::user::{DBUser, SafeUser};
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
    let five_minutes_ago = now - Duration::minutes(5);

    if user.list_last_update < five_minutes_ago {
        // Update list in background
        let user = user.clone();
        let user_id = user.id.clone();
        let state = state.clone();
        tokio::spawn(async move {
            let mal_user_list = get_mal_user_list(state.reqwest, user).await;

            match mal_user_list {
                Ok(mal) => {
                    let ids = mal
                        .data
                        .iter()
                        .map(|item| AnimeUserEntry {
                            status: item
                                .list_status
                                .status
                                .parse::<AnimeWatchStatus>()
                                .map_err(|_| AnimeWatchStatus::Watching)
                                .expect("Failed to parse watch status"),
                            user_id: user_id.clone(),
                            anime_id: item.node.id,
                        })
                        .collect::<Vec<_>>();
                    let mut importer = state.importer.lock().await;
                    importer.add_all(ids);
                }
                Err(err) => {
                    // TODO: Handle better?
                    tracing::error!("{:?}", err)
                }
            }
        });
    }

    // TODO: handle unwrap
    let entries = get_user_entrys(&state.db, user_id).await.unwrap();

    let anime_ids: Vec<i32> = entries.iter().map(|entry| entry.anime_id).collect();
    let animes = get_released_animes_by_id(&state.db, anime_ids)
        .await
        .unwrap();
    let entries = entries
        .iter()
        .map(|entry| SingleEntry {
            anime_id: entry.anime_id as u32,
            watch_priority: entry.watch_priority as u32,
            watch_status: entry.status.clone().into(),
        })
        .collect::<Vec<_>>();

    json_response!(StatusCode::OK, {
        "animes": animes,
        "list_entries": entries
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
