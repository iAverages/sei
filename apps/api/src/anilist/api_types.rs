use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub romaji: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relations {
    pub edges: Vec<RelationsEdge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationsEdge {
    pub relation_type: String,
    pub node: RelationsNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationsNode {
    pub id_mal: Option<i32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnilistErrorLocation {
    pub line: i32,
    pub column: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnilistError {
    pub message: String,
    pub status: i32,
    pub locations: Vec<AnilistErrorLocation>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistResponse {
    pub data: AnilistData,
    pub errors: Option<Vec<AnilistError>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverImage {
    pub large: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniListAnimeItem {
    pub status: String,
    pub relations: Option<Relations>,
    pub title: Title,
    pub id_mal: Option<u32>,
    pub season: Option<String>,
    pub season_year: Option<u32>,
    pub cover_image: CoverImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistData {
    pub anime1: Option<AniListAnimeItem>,
    pub anime2: Option<AniListAnimeItem>,
    pub anime3: Option<AniListAnimeItem>,
    pub anime4: Option<AniListAnimeItem>,
    pub anime5: Option<AniListAnimeItem>,
    pub anime6: Option<AniListAnimeItem>,
    pub anime7: Option<AniListAnimeItem>,
    pub anime8: Option<AniListAnimeItem>,
    pub anime9: Option<AniListAnimeItem>,
    pub anime10: Option<AniListAnimeItem>,
    pub anime11: Option<AniListAnimeItem>,
    pub anime12: Option<AniListAnimeItem>,
    pub anime13: Option<AniListAnimeItem>,
    pub anime14: Option<AniListAnimeItem>,
    pub anime15: Option<AniListAnimeItem>,
    pub anime16: Option<AniListAnimeItem>,
    pub anime17: Option<AniListAnimeItem>,
    pub anime18: Option<AniListAnimeItem>,
    pub anime19: Option<AniListAnimeItem>,
    pub anime20: Option<AniListAnimeItem>,
    pub anime21: Option<AniListAnimeItem>,
    pub anime22: Option<AniListAnimeItem>,
    pub anime23: Option<AniListAnimeItem>,
    pub anime24: Option<AniListAnimeItem>,
    pub anime25: Option<AniListAnimeItem>,
    pub anime26: Option<AniListAnimeItem>,
    pub anime27: Option<AniListAnimeItem>,
    pub anime28: Option<AniListAnimeItem>,
    pub anime29: Option<AniListAnimeItem>,
    pub anime30: Option<AniListAnimeItem>,
    pub anime31: Option<AniListAnimeItem>,
    pub anime32: Option<AniListAnimeItem>,
    pub anime33: Option<AniListAnimeItem>,
    pub anime34: Option<AniListAnimeItem>,
    pub anime35: Option<AniListAnimeItem>,
}
