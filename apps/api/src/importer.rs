use crate::{AppState, ImportQueueItem};

pub fn import_anime_from_ids(state: AppState, ids: Vec<i32>) {
    for id in ids {
        state.import_queue.push(ImportQueueItem::Anime {
            anime_id: id,
            times_in_queue: 0,
        });
    }
}
