use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Serialize;
use serde_json::json;
use sqlx::{query_as, MySql, QueryBuilder};

use crate::{
    anime::{self},
    helpers::json_response,
    models::user::User,
    types::CurrentUser,
    AppState,
};

struct AnimeIdResponse {
    anime_id: i32,
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

    let local_animes_future = anime::get_local_user_list(state.db, full_user.clone());
    let mal_animes_future = anime::get_mal_user_list(state.reqwest, full_user.clone());

    let (local_animes, mal_animes) = tokio::join!(local_animes_future, mal_animes_future);

    let local_animes = local_animes.expect("Failed to get local animes");
    let mal_animes = mal_animes.expect("Failed to get mal animes");

    if mal_animes.data.len() != local_animes.animes.len() {
        tracing::info!("Mal animes: {:?}", mal_animes.data.len());
        tracing::info!("Local animes: {:?}", local_animes.animes.len());

        mal_animes.data.iter().for_each(|anime| {
            if !local_animes
                .animes
                .iter()
                .any(|local| local.id == anime.node.id)
            {
                state.import_queue.push(anime.node.id);
            }
        });
    }

    json_response!(StatusCode::OK, local_animes)

    // tracing::info!("Anime list: {:?}", animes.data.len());

    // let full_user_t = full_user.clone();
    // let animes_t = animes.clone();
    // let db_t = state.db.clone();

    // tokio::spawn(async move {
    //     tracing::info!("Inserting anime_user");
    //     let animes = animes_t;
    //     let full_user = full_user_t;
    //     // let db = db_t;
    //     let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
    //         r#"
    //         INSERT IGNORE INTO anime_users (anime_id, user_id)
    //         "#,
    //     );
    //     // let user_id = Arc::new(full_user.id);
    //     query_builder.push_values(animes.data, |mut b, item| {
    //         // b.push_bind(item.node.id).push_bind(user_id.as_str());
    //         state.import_queue.push(item.node.id);
    //     });

    //     // query_builder
    //     //     .build()
    //     //     .execute(&db)
    //     //     .await
    //     //     .expect("Failed to insert anime_user");

    //     tracing::info!("Inserted anime_user");
    // });

    // let db_order = sqlx::query_as!(
    //     AnimeIdResponse,
    //     r#"
    //     SELECT anime_id FROM anime_users WHERE user_id = ? ORDER BY watch_priority
    //     "#,
    //     full_user.id
    // )
    // .fetch_all(&state.db)
    // .await
    // .expect("Failed to get anime order");

    // let mut ordered_animes = vec![];

    // for item in db_order {
    //     let anime = animes
    //         .data
    //         .iter()
    //         .find(|anime| anime.node.id == item.anime_id);

    //     if anime.is_none() {
    //         continue;
    //     }

    //     ordered_animes.push(anime.unwrap());
    // }

    // json_response!(StatusCode::OK, {"data": ordered_animes})
}

#[derive(serde::Deserialize, Serialize)]
pub struct ListUpdateRequest {
    pub ids: Vec<i32>,
}

#[axum::debug_handler]
pub async fn update_list_order(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(data): Json<ListUpdateRequest>,
) -> impl IntoResponse {
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        r#"
        INSERT INTO anime_users (anime_id, user_id, watch_priority) 
        "#,
    );

    let mut index = 1;
    let user_id = Arc::new(user.id);

    query_builder.push_values(data.ids, |mut b, id| {
        b.push_bind(id)
            .push_bind(user_id.to_string())
            .push_bind(index);
        index += 1;
    });

    query_builder
        .push(
            r#"
            ON DUPLICATE KEY UPDATE watch_priority = VALUES(watch_priority)
            "#,
        )
        .build()
        .execute(&state.db)
        .await
        .expect("Failed to update anime_user");
    // for (i, id) in data.ids.iter().enumerate() {
    //     let priority = i as f64;
    //     sqlx::query!(
    //         r#"
    //         UPDATE anime_users SET watch_priority = ? WHERE user_id = ? AND anime_id = ?
    //         "#,
    //         priority,
    //         user.id,
    //         id
    //     )
    //     .execute(&mut *tx)
    //     .await
    //     .expect("Failed to update anime_user");
    // }

    json_response!(StatusCode::OK, {"status": "ok"})
}
