use crate::AppState;

pub fn import_anime_from_ids(state: AppState, ids: Vec<i32>) {
    for id in ids {
        state.import_queue.push(id);
    }
}
