use axum::extract::State;
use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use chrono::{Duration, Utc};
use serde::Serialize;
use serde_json::json;

use crate::helpers::json_response;
use crate::mal::get_mal_user_list;
use crate::models::anime::{get_anime_relations, get_released_animes_by_id};
use crate::models::anime_users::{
    get_animes_for_user, get_user_list_entries, update_watch_priority, WatchPriorityUpdate,
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
        // TODO: handle animes deleted from list
        tokio::spawn(async move {
            let response = get_mal_user_list(state.reqwest, user).await;
            if let Err(error) = response {
                tracing::error!(
                    error = error.to_string(),
                    "failed to fetch mal list for user"
                );
                return;
            }

            let animes = response.unwrap();
            let _ = state
                .importer
                .add_items(
                    animes.data.into_iter().map(|anime| anime.node.id).collect(),
                    Some(user_id.clone()),
                )
                .await;
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

    json_response!(StatusCode::OK, {
        "animes": animes,
        "list_entries": list_entries,
        "relations": anime_relations,
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
