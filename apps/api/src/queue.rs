use std::time::Duration;

use sqlx::{MySql, Pool};
use tokio::time::timeout;

use crate::{
    anime::{self, CoverImage, RelationsEdge, Title, MAX_ANILIST_PER_QUERY},
    AppState, ImportQueueItem,
};

#[derive(Debug, Clone)]
pub struct InsertAnime {
    pub status: String,
    pub title: Title,
    pub id_mal: i64,
    pub type_: String,
    pub cover_image: CoverImage,
    pub season: Option<String>,
    pub season_year: Option<i32>,
}

pub async fn insert_anime(db: &Pool<MySql>, anime: InsertAnime) -> Result<(), sqlx::Error> {
    sqlx::query!(
                r#"
                INSERT INTO animes (id, romaji_title, english_title, status, picture, season, season_year, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE romaji_title = ?, english_title = ?, status = ?, picture = ?, season = ?, season_year = ?, updated_at = ?
                "#,
                anime.id_mal,
                anime.title.romaji,
                anime.title.english,
                anime.status,
                anime.cover_image.large,
                anime.season,
                anime.season_year,
                chrono::Utc::now(),
                anime.title.romaji,
                anime.title.english,
                anime.status,
                anime.cover_image.large,
                anime.season,
                anime.season_year,
                chrono::Utc::now(),
).execute(db).await?;

    Ok(())
}

pub fn import_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Starting import queue worker...");

        loop {
            let mut ids = vec![];
            for _ in 0..MAX_ANILIST_PER_QUERY {
                let id = timeout(Duration::from_millis(500), state.import_queue.pop()).await;

                if let Ok(id) = id {
                    ids.push(id);
                } else {
                    break;
                }
            }

            if ids.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }

            tracing::info!(
                "Importing anime: {}",
                ids.clone()
                    .iter()
                    .map(|id| id.anime_id.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            let mut import_ids: Vec<i32> = vec![];

            for id in ids.clone() {
                let local_data = anime::get_local_anime_data(state.db.clone(), id.anime_id).await;

                if let Ok(local_data) = local_data {
                    let now = chrono::Utc::now().naive_utc();
                    let updated_at = local_data.updated_at;
                    let diff = now - updated_at;

                    if diff.num_hours() < 1 {
                        tracing::info!(
                            "Anime {} was updated less than 1 hour ago, skipping",
                            id.anime_id
                        );
                        continue;
                    }
                    tracing::info!(
                        "Anime {} was last updated more than 1 hour ago, reimporting",
                        id.anime_id
                    );
                    import_ids.push(id.anime_id);
                } else {
                    tracing::info!("Anime {} not found in local database", id.anime_id);
                    import_ids.push(id.anime_id);
                }
            }

            if import_ids.is_empty() {
                for item in ids.iter() {
                    if item.user_id.is_none() {
                        continue;
                    }
                    sqlx::query!(
                        r#"
                        INSERT INTO anime_users (anime_id, user_id)
                        VALUES
                            (?, ?)
                        "#,
                        item.anime_id,
                        item.user_id
                    )
                    .execute(&state.db)
                    .await
                    .expect("Failed to insert anime_user");
                }
                continue;
            }

            // Adds delay between sending requests to Anilist
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let import_ids2 = import_ids.clone();
            let result = anime::get_animes_from_anilist(state.reqwest.clone(), import_ids2).await;

            if result.retry_after != -1 {
                tracing::warn!("Rate limit hit, sleeping for {}", result.retry_after);
                tracing::info!(
                    "Adding {} back to queue",
                    import_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(result.retry_after as u64))
                    .await;
            }

            let ani_result = match result.response {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to get anime: {}, readding to queue", e);
                    import_ids.iter().for_each(|id| {
                        state.import_queue.push(ImportQueueItem {
                            anime_id: *id,
                            user_id: None,
                            anime_watch_status: None,
                        })
                    });
                    continue;
                }
            };

            for i in 0..import_ids.len() {
                let anime = match i {
                    0 => ani_result.data.anime1.as_ref(),
                    1 => ani_result.data.anime2.as_ref(),
                    2 => ani_result.data.anime3.as_ref(),
                    3 => ani_result.data.anime4.as_ref(),
                    4 => ani_result.data.anime5.as_ref(),
                    5 => ani_result.data.anime6.as_ref(),
                    6 => ani_result.data.anime7.as_ref(),
                    7 => ani_result.data.anime8.as_ref(),
                    8 => ani_result.data.anime9.as_ref(),
                    9 => ani_result.data.anime10.as_ref(),
                    10 => ani_result.data.anime11.as_ref(),
                    11 => ani_result.data.anime12.as_ref(),
                    12 => ani_result.data.anime13.as_ref(),
                    13 => ani_result.data.anime14.as_ref(),
                    14 => ani_result.data.anime15.as_ref(),
                    15 => ani_result.data.anime16.as_ref(),
                    16 => ani_result.data.anime17.as_ref(),
                    17 => ani_result.data.anime18.as_ref(),
                    18 => ani_result.data.anime19.as_ref(),
                    19 => ani_result.data.anime20.as_ref(),
                    _ => None,
                };

                if anime.is_none() {
                    tracing::warn!(
                        "Anime {} not found in response, readding to queue...",
                        import_ids[i]
                    );
                    state.import_queue.push(ImportQueueItem {
                        anime_id: import_ids[i],
                        user_id: None,
                        anime_watch_status: None,
                    });
                    continue;
                }

                let anime = anime.unwrap();

                let insert_result = insert_anime(
                    &state.db,
                    InsertAnime {
                        cover_image: anime.cover_image.clone(),
                        id_mal: anime.id_mal,
                        status: anime.status.clone(),
                        title: anime.title.clone(),
                        type_: anime.type_.clone(),
                        season: anime.season.clone(),
                        season_year: anime.season_year,
                    },
                )
                .await;

                if let Err(e) = insert_result {
                    tracing::error!("Failed to insert anime: {}", e);
                    import_ids.iter().for_each(|id| {
                        state.import_queue.push(ImportQueueItem {
                            anime_id: *id,
                            user_id: None,
                            anime_watch_status: None,
                        })
                    });
                    continue;
                }

                let relation = anime.relations.clone();
                if relation.is_none() {
                    continue;
                }
                let relation = relation.unwrap();

                let relation: Vec<RelationsEdge> = relation
                    .edges
                    .into_iter()
                    .filter(|r| {
                        r.relation_type == "PREQUEL"
                            || r.relation_type == "SEQUEL"
                            || r.relation_type == "SIDE_STORY"
                    })
                    .filter(|r| r.node.id_mal.is_some())
                    .collect();

                for related in relation {
                    let insert_result = insert_anime(
                        &state.db,
                        InsertAnime {
                            cover_image: related.node.cover_image.clone(),
                            id_mal: related.node.id_mal.unwrap(), // Safe to unwrap, we filter out None above
                            status: related.node.status.clone(),
                            title: related.node.title.clone(),
                            type_: related.node.type_.clone(),
                            season: related.node.season.clone(),
                            season_year: related.node.season_year,
                        },
                    )
                    .await;

                    if let Err(e) = insert_result {
                        tracing::error!("Failed to insert related anime: {}", e);
                        continue;
                    }

                    let relation_isert_result = sqlx::query!(
                            r#"
                            INSERT INTO anime_relations (base_anime_id, related_anime_id, relation)
                            VALUES (?, ?, ?)
                            ON DUPLICATE KEY UPDATE base_anime_id = ?, related_anime_id = ?, relation = ?
                            "#,
                            anime.id_mal,
                            related.node.id_mal,
                            related.relation_type,
                            anime.id_mal,
                            related.node.id_mal,
                            related.relation_type,
                        )
                        .execute(&state.db)
                        .await;

                    if let Err(e) = relation_isert_result {
                        tracing::error!("Failed to insert relation link: {}", e);
                    }
                }
            }

            for item in ids.iter() {
                if item.user_id.is_none() {
                    continue;
                }
                sqlx::query!(
                    r#"
                    INSERT INTO anime_users (anime_id, user_id, status)
                    VALUES
                        (?, ?, ?)
                    "#,
                    item.anime_id,
                    item.user_id,
                    item.anime_watch_status
                )
                .execute(&state.db)
                .await
                .expect("Failed to insert anime_user");
            }

            tracing::info!(
                "Imported anime {}",
                import_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
        }
    });
}
