use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::query_as;

use crate::{helpers::json_response, models::user::User, types::CurrentUser, AppState};

#[derive(Deserialize, Serialize)]
struct AnimePicture {
    large: String,
    medium: String,
}

#[derive(Deserialize, Serialize)]
struct AnimeListNode {
    id: i32,
    title: String,
    main_picture: AnimePicture,
}

#[derive(Deserialize, Serialize)]
struct AnimeListItem {
    node: AnimeListNode,
    list_status: Value,
}

#[derive(Deserialize, Serialize)]
struct AnimeListResponse {
    data: Vec<AnimeListItem>,
    paging: Value,
}

#[axum::debug_handler]
pub async fn get_anime(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
) -> impl IntoResponse {
    let full_user = query_as!(
        User,
        r#"
        SELECT * FROM users WHERE id = ?
        "#,
        user.id
    )
    .fetch_one(&state.db)
    .await
    .expect("Failed to get user");

    let res = state
        .reqwest
        .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000")
        .bearer_auth(full_user.mal_access_token)
        .send()
        .await
        .expect("Failed to get MAL anime");

    let anime = res
        .json::<AnimeListResponse>()
        .await
        .expect("Failed to parse MAL anime");

    json_response!(StatusCode::OK, anime)
}
