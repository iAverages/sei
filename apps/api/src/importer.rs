use crate::{AppState, ImportQueueItem};

pub fn import_anime_from_ids(state: AppState, ids: Vec<i32>) {
    for id in ids {
        state.import_queue.push(ImportQueueItem {
            anime_id: id,
            user_id: None,
            anime_watch_status: None,
        });
    }
}
