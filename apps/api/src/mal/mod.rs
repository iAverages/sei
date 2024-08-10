use anyhow::Context;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::user::DBUser;

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimePicture {
    pub large: String,
    pub medium: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeBroadcast {
    pub day_of_the_week: String,
    pub start_time: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListNode {
    pub id: u32,
    pub title: String,
    pub main_picture: AnimePicture,
    pub status: String,
    pub broadcast: Option<AnimeBroadcast>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MalListStatusStatus {
    Watching,
    Completed,
    OnHold,
    Dropped,
    PlanToWatch,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MalListStatus {
    pub status: String,
    pub score: i32,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListItem {
    pub node: AnimeListNode,
    pub list_status: MalListStatus,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MalAnimeListResponse {
    pub data: Vec<AnimeListItem>,
    pub paging: Value,
}
pub async fn get_mal_user_list(
    reqwest: Client,
    user: DBUser,
) -> Result<MalAnimeListResponse, anyhow::Error> {
    tracing::info!("Getting MAL anime list for user {}", user.id);
    let res = reqwest
        .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status,node.status,node.num_episodes,node.broadcast&limit=1000&nsfw=1")
        .bearer_auth(user.mal_access_token)
        .send()
        .await
        .expect("Failed to get MAL anime");

    let text = res.text().await?;
    let anime: MalAnimeListResponse = serde_json::from_str(&text)
        .with_context(|| format!("Unable to deserialise response. Body was: \"{}\"", text))?;

    let paging = anime.paging.clone();

    tracing::info!("Got {} anime from MAL", anime.data.len());

    Ok(MalAnimeListResponse {
        data: anime.data,
        paging,
    })
}
