use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::models::anime::FullAnime;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistFetchAnimeResponse {
    pub data: HashMap<String, AnilistApiAnime>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistFetchAnimeErrorResponse {
    pub errors: Vec<AnilistError>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistError {
    pub message: String,
    pub status: i64,
    pub locations: Vec<AnilistErrorLocation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistErrorLocation {
    pub line: i64,
    pub column: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistApiAnime {
    pub status: String,
    pub id_mal: i64,
    pub title: AnimeTitle,
    pub season: Option<String>,
    pub season_year: Option<i64>,
    pub cover_image: AnimeCoverImage,
    pub relations: AnimeRelations,
}
impl From<AnilistApiAnime> for FullAnime {
    fn from(val: AnilistApiAnime) -> Self {
        Self {
            id: val.id_mal as i32,
            status: val.status,
            picture: Some(val.cover_image.large),
            romaji_title: val.title.romaji,
            season: val.season,
            season_year: val.season_year.map(|value| value as i32),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeTitle {
    pub romaji: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeCoverImage {
    pub large: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeRelations {
    pub edges: Vec<AnimeRelationEdge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeRelationEdge {
    pub relation_type: String,
    pub node: AnimeRelationEdgeNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeRelationEdgeNode {
    pub id_mal: Option<i64>,
}
