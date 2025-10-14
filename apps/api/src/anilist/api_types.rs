use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistFetchAnimeResponse {
    pub data: HashMap<String, AnilistApiAnime>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistApiAnime {
    pub status: String,
    pub id_mal: i64,
    pub title: AnimeTitle,
    pub season: String,
    pub season_year: i64,
    pub cover_image: AnimeCoverImage,
    pub relations: AnimeRelations,
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
    pub id_mal: i64,
}
