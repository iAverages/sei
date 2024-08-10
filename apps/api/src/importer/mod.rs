// Needs to do the following
// Only allow ids to be in queue once
// do not readd ids we have seen since processing started

use std::collections::HashSet;

use reqwest::Client;
use sqlx::{MySql, Pool};

use crate::anilist::{
    get_anime_from_anilist_result, get_animes_from_anilist, MAX_ANILIST_PER_QUERY,
};
use crate::models::anime::{insert_animes, InsertAnime};

pub struct Importer {
    reqwest: Client,
    db: Pool<MySql>,
    // The current queue we are processing
    queue: HashSet<u32>,

    // IDs we have seen recently.
    // IDs here have been processed in the
    // last queue job and will be removed
    // after all items in queue are processed
    seen_recently: HashSet<u32>,
}

impl Importer {
    pub fn new(reqwest: Client, db: Pool<MySql>) -> Self {
        Importer {
            reqwest,
            db,
            queue: HashSet::new(),
            seen_recently: HashSet::new(),
        }
    }

    pub fn add(&mut self, id: u32) -> bool {
        tracing::debug!("Adding {:?} to queue", id);
        self.queue.insert(id)
    }

    pub async fn process(&mut self) {
        tracing::debug!(
            "Processing all items in queue: amount {:?}",
            self.queue.len()
        );
        let items = self.get_items_to_process(MAX_ANILIST_PER_QUERY);
        let animes = get_animes_from_anilist(&self.reqwest, items).await;

        if let Ok(anilist_response) = animes.response {
            tracing::info!("Got animes from anilist");

            let mut anime_data = vec![];

            for anime_index in 0..MAX_ANILIST_PER_QUERY {
                let anime = get_anime_from_anilist_result(anilist_response.clone(), anime_index);

                if anime.is_none() {
                    tracing::debug!("Last found animeIndex was {:?}", anime_index - 1);
                    break;
                }
                let anime = anime.unwrap();
                // TODO: Can I make the animes not have optional everything here?
                let anime_id = anime.id_mal.unwrap();
                anime_data.push(anime);
                self.seen_recently.insert(anime_id);
            }

            tracing::info!("Got {:?} animes from anilist", anime_data.len());

            let formatted = anime_data
                .iter()
                // TODO: CLONE AAAAAAAa
                .map(|anime| InsertAnime {
                    id_mal: anime.id_mal.unwrap(),
                    title: anime.title.clone(),
                    status: anime.status.clone(),
                    season: anime.season.clone(),
                    cover_image: anime.cover_image.clone(),
                    season_year: anime.season_year,
                })
                .collect();

            let _insert = insert_animes(&self.db, formatted).await;
        }
    }

    fn get_items_to_process(&self, max: usize) -> Vec<&u32> {
        self.queue.iter().take(max).collect()
    }
}
