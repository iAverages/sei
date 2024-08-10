// Needs to do the following
// Only allow ids to be in queue once
// do not readd ids we have seen since processing started

use std::collections::hash_map::Entry::Vacant;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use chrono::{TimeZone, Utc};
use reqwest::Client;
use serde::Serialize;
use sqlx::{MySql, Pool};
use tokio::time::sleep;

use crate::anilist::{
    get_anime_from_anilist_result, get_animes_from_anilist, MAX_ANILIST_PER_QUERY,
};
use crate::models::anime::{insert_animes, InsertAnime};
use crate::models::anime_users::link_user_to_anime;

#[derive(Clone, Debug)]
pub enum AnimeWatchStatus {
    Watching,
    Completed,
    OnHold,
    Dropped,
    PlanToWatch,
}

#[derive(Debug)]
pub struct ParseAnimeWatchStatusError;

impl FromStr for AnimeWatchStatus {
    type Err = ParseAnimeWatchStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "watching" => Ok(AnimeWatchStatus::Watching),
            "completed" => Ok(AnimeWatchStatus::Completed),
            "on_hold" => Ok(AnimeWatchStatus::OnHold),
            "dropped" => Ok(AnimeWatchStatus::Dropped),
            "plan_to_watch" => Ok(AnimeWatchStatus::PlanToWatch),
            _ => Err(ParseAnimeWatchStatusError),
        }
    }
}

impl From<AnimeWatchStatus> for String {
    fn from(val: AnimeWatchStatus) -> Self {
        let str = match val {
            AnimeWatchStatus::Watching => "watching",
            AnimeWatchStatus::Completed => "completed",
            AnimeWatchStatus::OnHold => "on_hold",
            AnimeWatchStatus::Dropped => "dropped",
            AnimeWatchStatus::PlanToWatch => "plan_to_watch",
        };

        str.to_string()
    }
}

#[derive(Clone)]
pub struct AnimeUserEntry {
    pub anime_id: u32,
    pub user_id: String,
    pub status: AnimeWatchStatus,
}

pub struct Importer {
    reqwest: Client,
    db: Pool<MySql>,

    // The current queue we are processing
    queue: HashMap<u32, Vec<AnimeUserEntry>>,

    // IDs we have seen recently.
    // IDs here have been processed in the
    // last queue job and will be removed
    // after all items in queue are processed
    seen_recently: HashSet<u32>,

    // List of IDs to not attempt to import
    // Mainly used for ids that do not exist on anilist
    ignore_ids: HashSet<u32>,
}

impl Importer {
    pub fn new(reqwest: Client, db: Pool<MySql>) -> Self {
        Importer {
            reqwest,
            db,
            queue: HashMap::new(),
            seen_recently: HashSet::new(),
            ignore_ids: HashSet::new(),
        }
    }

    pub fn stats(&self) -> ImporterStatus {
        ImporterStatus {
            queue_total: self.queue.len(),
            ignored_ids: self.ignore_ids.iter().cloned().collect(),
        }
    }

    pub fn add(&mut self, id: u32, user_entry: AnimeUserEntry) -> bool {
        if self.ignore_ids.contains(&id) {
            tracing::warn!("Tried to insert id {:?}, but it is ignored", id);
            return false;
        }

        let inserted;
        if let Vacant(e) = self.queue.entry(id) {
            inserted = true;
            e.insert(vec![user_entry]);
        } else {
            let current = self.queue.get_mut(&id).unwrap();
            let has = current
                .iter()
                .find(|entry| entry.user_id == user_entry.user_id);
            if let Some(_) = has {
                current.push(user_entry);
                inserted = true;
            } else {
                tracing::warn!(
                    user_id = user_entry.user_id,
                    anime_id = id,
                    "Failed to add user entry to queue."
                );
                panic!("Failed to add user entry to queue.");
            }
        }

        tracing::debug!(
            anime = id,
            user_id = id,
            "Anime {} queue with user",
            if inserted { "added to" } else { "already in" }
        );

        inserted
    }

    pub fn add_all(&mut self, entries: Vec<AnimeUserEntry>) {
        entries.iter().for_each(|entry| {
            let anime_id = entry.anime_id.clone();
            self.add(anime_id, entry.clone());
        });
    }

    pub async fn process(&mut self) {
        let items = self.get_items_to_process(MAX_ANILIST_PER_QUERY);

        if items.is_empty() {
            tracing::trace!("No items in queue to process");
            return;
        }

        let ids: Vec<u32> = items.iter().map(|item| item.0).collect();

        tracing::debug!("Processing {:?} items", ids.len());
        let animes = get_animes_from_anilist(&self.reqwest, ids).await;

        // Handle anilist rate limits
        if animes.rate_limit_reset != -1 {
            tracing::warn!(
                "Rate limited by anilist for {:?}, waiting now...",
                animes.rate_limit_reset
            );
            let now = Utc::now();
            let target_time = Utc
                .timestamp_opt(animes.rate_limit_reset.into(), 0)
                .single()
                .expect("Failed to parse anilist rate limit reset into duration");
            sleep(
                (target_time - now)
                    .to_std()
                    .expect("Failed to convert chrone duration to std duration"),
            )
            .await;
        }

        if let Ok(anilist_response) = animes.response {
            if let Some(errors) = anilist_response.errors.clone() {
                // Only handle first for now, never seen it return multiple errors
                // for the requests we do
                let error = errors.first();
                if let Some(error) = error {
                    if error.status == 404 {
                        // Same here, never seen it return multiple
                        let loc = error.locations.first();
                        if let Some(loc) = loc {
                            let total = animes.query.query.lines().count();
                            let lpq = total / items.len();
                            let error_index = (loc.line - 1) as usize / lpq;
                            let errored_item =
                                items.get(error_index).expect("No item for error given");
                            tracing::error!(
                                "Anime {:?} has not found on anilist, adding to ignore list",
                                errored_item.0,
                            );

                            self.ignore_ids.insert(errored_item.0);
                            self.queue.remove(&errored_item.0);
                            // Just end this loop and pickup again on the next
                            // TODO: Retry here instead of exiting?
                            return;
                        }
                    }
                }
            }

            tracing::info!("Got animes from anilist");

            let mut anime_data = vec![];

            for anime_index in 0..MAX_ANILIST_PER_QUERY {
                let anime = get_anime_from_anilist_result(anilist_response.clone(), anime_index);

                if anime.is_none() {
                    tracing::debug!("Last found animeIndex was {:?}", anime_index);
                    break;
                }
                let anime = anime.unwrap();
                // TODO: Can I make the animes not have optional everything here?
                let anime_id = anime.id_mal.unwrap();
                anime_data.push(anime);
                self.queue.remove(&anime_id);
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

            let _ = insert_animes(&self.db, formatted).await;
            let _ = link_user_to_anime(&self.db, items).await;
        }
    }

    fn get_items_to_process(&self, max: usize) -> Vec<(u32, Vec<AnimeUserEntry>)> {
        self.queue
            .iter()
            .take(max)
            .map(|(&key, value)| (key, value.clone()))
            .collect()
    }
}

#[derive(Serialize)]
pub struct ImporterStatus {
    queue_total: usize,
    ignored_ids: Vec<u32>,
}
