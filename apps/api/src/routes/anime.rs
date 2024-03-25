use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Serialize;
use serde_json::json;
use sqlx::{query_as, Execute, MySql, QueryBuilder};

use crate::{
    anime::{
        self, get_anime_with_relations, get_local_anime_data, ListStatus, LocalAnineListResult,
    },
    helpers::json_response,
    models::user::User,
    types::CurrentUser,
    AppError, AppState, ImportQueueItem,
};

struct AnimeIdResponse {
    anime_id: i32,
}

#[axum::debug_handler]
pub async fn get_anime_list(
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

    let local_animes_future = anime::get_local_user_list(state.db.clone(), full_user.clone());
    let mal_animes_future = anime::get_mal_user_list(state.reqwest, full_user.clone());

    let (local_animes, mal_animes) = tokio::join!(local_animes_future, mal_animes_future);

    let local_animes = local_animes.expect("Failed to get local animes");
    let mal_animes = mal_animes.expect("Failed to get mal animes");
    let status;

    tracing::info!("Mal animes: {:?}", mal_animes.data.len());
    tracing::info!("Local animes: {:?}", local_animes.animes.len());

    if mal_animes.data.len() == local_animes.animes.len() {
        status = ListStatus::Imported;
    } else if local_animes.animes.is_empty() {
        status = ListStatus::Importing;
    } else {
        status = ListStatus::Updating;
    }

    mal_animes.data.iter().for_each(|anime| {
        let local_anime = local_animes
            .animes
            .iter()
            .find(|local| local.id == anime.node.id);

        if local_anime.is_none() {
            state.import_queue.push(ImportQueueItem::UserAnime {
                anime_id: anime.node.id,
                user_id: full_user.id.clone(),
                anime_watch_status: anime.list_status.status.clone(),
                times_in_queue: 0,
            });
            return;
        }

        let local_anime = local_anime.unwrap();

        if local_anime.watch_status != anime.list_status.status {
            state.import_queue.push(ImportQueueItem::UserAnime {
                anime_id: anime.node.id,
                user_id: full_user.id.clone(),
                anime_watch_status: anime.list_status.status.clone(),
                times_in_queue: 0,
            });
            return;
        }
    });

    let user_order = query_as!(
        AnimeIdResponse,
        r#"
        SELECT anime_id FROM anime_users WHERE user_id = ? ORDER BY watch_priority
        "#,
        full_user.id
    );

    let db_order = user_order
        .fetch_all(&state.db)
        .await
        .expect("Failed to get anime order");

    let mut ordered_animes = vec![];

    for item in db_order {
        let anime = local_animes
            .animes
            .iter()
            .find(|anime| anime.id == item.anime_id)
            .cloned();

        if anime.is_none() {
            continue;
        }

        ordered_animes.push(anime.unwrap());
    }

    let res = LocalAnineListResult {
        animes: ordered_animes,
        status,
    };

    json_response!(StatusCode::OK, res)

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

// The number of parameters in MySQL must fit in a `u16`.
const BIND_LIMIT: usize = 65535;

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

    let groups = data.ids.chunks(BIND_LIMIT / 3);

    for group in groups {
        query_builder.push_values(group.iter(), |mut b, id| {
            b.push_bind(id).push_bind(user_id.as_str()).push_bind(index);
            index += 1;
        });

        let q = query_builder
            .push(
                r#"
                ON DUPLICATE KEY UPDATE watch_priority = VALUES(watch_priority)
                "#,
            )
            .build();

        tracing::info!("SQL: {:?} ", q.sql());

        q.execute(&state.db)
            .await
            .expect("Failed to update anime_user");
    }

    // query_builder.push_values(data.ids.into_iter().take(BIND_LIMIT / 3), |mut b, id| {
    //     b.push_bind(id)
    //         .push_bind(user_id.to_string())
    //         .push_bind(index);
    //     index += 1;
    // });

    // let q = query_builder
    //     .push(
    //         r#"
    //         ON DUPLICATE KEY UPDATE watch_priority = VALUES(watch_priority)
    //         "#,
    //     )
    //     .build();

    // tracing::info!("SQL: {:?} ", q.sql());

    // q.execute(&state.db)
    //     .await
    //     .expect("Failed to update anime_user");
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

#[axum::debug_handler]
pub async fn get_anime(
    State(state): State<AppState>,
    Path(anime_id): Path<i32>,
) -> impl IntoResponse {
    let animes = get_anime_with_relations(state.db, anime_id).await;

    let animes = animes.expect("Failed to get anime");

    json_response!(StatusCode::OK, {"data": animes})
}
