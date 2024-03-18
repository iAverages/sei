use crate::{AppState, ImportQueueItem};

pub fn start_background_job(state: AppState) {
    tracing::info!("Starting background job...");
    let db = state.db.clone();
    let import_queue = state.import_queue.clone();

    tokio::spawn(async move {
        loop {
            // let needs_revalidating = sqlx::query!(
            //     "SELECT id FROM animes WHERE updated_at < DATE_SUB(NOW(), INTERVAL 1 hour)"
            // )
            // .fetch_all(&db)
            // .await;

            // let needs_revalidating = match needs_revalidating {
            //     Ok(needs_revalidating) => needs_revalidating,
            //     Err(err) => {
            //         tracing::error!("Error fetching animes: {:?}", err);
            //         tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
            //         continue;
            //     }
            // };

            // for anime in needs_revalidating {
            //     import_queue.push(ImportQueueItem::Anime {
            //         anime_id: anime.id,
            //         times_in_queue: 0,
            //     });
            // }

            tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;
        }
    });
}
