use crate::{anime, AppState};

pub async fn import_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Starting import queue worker");

        loop {
            let id = state.import_queue.pop().await;
            tracing::info!("Importing anime {}", id);

            let local_data = anime::get_local_anime_data(state.db.clone(), id).await;

            if let Ok(local_data) = local_data {
                let now = chrono::Utc::now().naive_utc();
                let updated_at = local_data.updated_at;
                let diff = now - updated_at;

                if diff.num_hours() < 12 {
                    tracing::info!("Anime {} was updated less than 12 hours ago, skipping", id);
                    continue;
                }
            } else {
                tracing::info!("Anime {} not found in local database", id);
            }

            // Adds delay between sending requests to Anilist
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            let result = anime::get_animes_from_anilist(state.reqwest.clone(), vec![id]).await;

            if result.retry_after != -1 {
                tracing::warn!("Rate limit hit, sleeping for {}", result.retry_after);
                tracing::info!("Adding {} back to queue", id);
                tokio::time::sleep(tokio::time::Duration::from_secs(result.retry_after as u64))
                    .await;
            }

            let ani_result = match result.response {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to get anime: {}, readding to queue", e);
                    state.import_queue.push(id);
                    continue;
                }
            };

            let anime = ani_result.data.anime1.unwrap();

            let insert_result = sqlx::query!(
                r#"
                INSERT INTO animes (id, romaji_title, english_title, status, picture, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE romaji_title = ?, english_title = ?, status = ?, picture = ?, updated_at = ?
                "#,
                id,
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
                state.import_queue.push(id);
                continue;
            }

            // for related in anime.relations.edges.iter() {
            //     let insert_result = sqlx::query!(
            //         r#"
            //         INSERT INTO animes (id, title, picture, updated_at)
            //         VALUES (?, ?, ?, ?)
            //         ON DUPLICATE KEY UPDATE title = ?, updated_at = ?, picture = ?
            //         "#,
            //         related.node.id_mal,
            //         related.node.title.romaji,
            //         related.node.title.romaji,
            //         chrono::Utc::now(),
            //         related.node.title.romaji,
            //         related.node.title.romaji,
            //         chrono::Utc::now(),
            //     )
            //     .execute(&state.db)
            //     .await;

            //     if let Err(e) = insert_result {
            //         tracing::error!("Failed to insert related anime: {}", e);
            //     }

            //     let relation_isert_result = sqlx::query!(
            //         r#"
            //         INSERT INTO anime_relations (base_anime_id, related_anime_id, relation)
            //         VALUES (?, ?, ?)
            //         ON DUPLICATE KEY UPDATE base_anime_id = ?, related_anime_id = ?, relation = ?
            //         "#,
            //         id,
            //         related.id,
            //         related.relation_type,
            //         id,
            //         related.id,
            //         related.relation_type,
            //     )
            //     .execute(&state.db)
            //     .await;

            //     if let Err(e) = relation_isert_result {
            //         tracing::error!("Failed to insert relation: {}", e);
            //     }
            // }

            // let relation_isert_result = sqlx::query!(
            //     r#"
            //     INSERT INTO anime_relations (base_anime_id, related_anime_id, relation)
            //     VALUES (?, ?, ?)
            //     ON DUPLICATE KEY UPDATE base_anime_id = ?, related_anime_id = ?, relation = ?
            //     "#,
            //     id,

            //     ani_result
            //     id,
            //     ani_result.response.data.media.status,
            // )
            // .execute(&state.db)
            // .await;

            tracing::info!("Imported anime {}", id);
        }
    });
}
