use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::user::User;

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimePicture {
    pub large: String,
    pub medium: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListNode {
    pub id: i32,
    pub title: String,
    pub main_picture: AnimePicture,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListItem {
    pub node: AnimeListNode,
    pub list_status: Value,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListResponse {
    pub data: Vec<AnimeListItem>,
    pub paging: Value,
}

pub async fn get_user_list(
    reqwest: Client,
    user: User,
) -> Result<AnimeListResponse, anyhow::Error> {
    let res = reqwest
        .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status&limit=1000&nsfw=1")
        .bearer_auth(user.mal_access_token)
        .send()
        .await
        .expect("Failed to get MAL anime");
    let anime = res.json::<AnimeListResponse>().await?;
    let paging = anime.paging.clone();

    let anime = anime
        .data
        .into_iter()
        .filter(|item| item.list_status["status"] != "completed")
        .collect();

    Ok(AnimeListResponse {
        data: anime,
        paging,
    })
}
