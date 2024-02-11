use std::time::Duration;

use tokio::time::timeout;

use crate::{anime, AppState, ImportQueueItem};

pub async fn import_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Starting import queue worker");

        loop {
            let mut ids = vec![];
            for _ in 0..10 {
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

                    if diff.num_hours() < 12 {
                        tracing::info!(
                            "Anime {} was updated less than 12 hours ago, skipping",
                            id.anime_id
                        );
                        continue;
                    }
                } else {
                    tracing::info!("Anime {} not found in local database", id.anime_id);
                    import_ids.push(id.anime_id);
                }
            }

            if import_ids.is_empty() {
                for item in ids.iter() {
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
                            user_id: "".to_string(),
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
                    _ => None,
                };

                if anime.is_none() {
                    tracing::warn!("Anime {} not found in response", import_ids[i]);
                    continue;
                }

                let anime = anime.unwrap();

                let insert_result = sqlx::query!(
                r#"
                INSERT INTO animes (id, romaji_title, english_title, status, picture, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE romaji_title = ?, english_title = ?, status = ?, picture = ?, updated_at = ?
                "#,
                anime.id_mal,
                anime.title.romaji,
                anime.title.english,
                anime.status,
                anime.cover_image.large,
                chrono::Utc::now(),
                anime.title.romaji,
                anime.title.english,
                anime.status,
                anime.cover_image.large,
                chrono::Utc::now(),
            )
            .execute(&state.db)
            .await;

                if let Err(e) = insert_result {
                    tracing::error!("Failed to insert anime: {}", e);
                    import_ids.iter().for_each(|id| {
                        state.import_queue.push(ImportQueueItem {
                            anime_id: *id,
                            user_id: "".to_string(),
                        })
                    });
                    continue;
                }

                let relation = anime.relations.clone();
                if relation.is_none() {
                    continue;
                }
                let relation = relation.unwrap();

                for related in relation.edges.into_iter() {
                    let insert_result = sqlx::query!(
                        r#"
                        INSERT INTO animes (id, romaji_title, english_title, status, picture, updated_at)
                        VALUES (?, ?, ?, ?, ?, ?)
                        ON DUPLICATE KEY UPDATE romaji_title = ?, english_title = ?, status = ?, picture = ?, updated_at = ?
                            "#,
                            related.node.id_mal,
                            related.node.title.romaji,
                            related.node.title.english,
                            related.node.status,
                            related.node.cover_image.large,
                            chrono::Utc::now(),
                            related.node.title.romaji,
                            related.node.title.english,
                            related.node.status,
                            related.node.cover_image.large,
                            chrono::Utc::now(),
                    )
                    .execute(&state.db)
                    .await;

                    if let Err(e) = insert_result {
                        tracing::error!("Failed to insert related anime: {}", e);
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
                        tracing::error!("Failed to insert relation: {}", e);
                    }
                }
            }

            for item in ids.iter() {
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
