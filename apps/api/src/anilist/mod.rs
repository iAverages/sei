pub mod api_types;

use std::fmt::{Display, Formatter};

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use self::api_types::{AniListAnimeItem, AnilistResponse};

fn get_header_i32(response: &Response, key: String, default: i32) -> i32 {
    let default_str: String = default.to_string().to_owned();
    let default_str = default_str.as_str();
    response
        .headers()
        .get(key)
        .unwrap_or(&reqwest::header::HeaderValue::from_str(default_str).unwrap())
        .to_str()
        .unwrap_or(default_str)
        .parse::<i32>()
        .unwrap_or(default)
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GqlQuery {
    query: String,
    variables: Value,
}

impl Display for GqlQuery {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Query: {}\nvariables: {}", self.query, self.variables)
    }
}

pub struct AniListResult {
    pub response: Result<AnilistResponse, anyhow::Error>,
    pub rate_limit_remaining: i32,
    pub rate_limit_current: i32,
    pub rate_limit_reset: i32,
    pub retry_after: i32,
}

pub async fn get_animes_from_anilist(reqwest: &Client, ids: Vec<&u32>) -> AniListResult {
    let gql_query = generate_gql_query(ids);

    let res = reqwest
        .post("https://graphql.anilist.co")
        .json(&json!(gql_query))
        .send()
        .await;

    let res = match res {
        Ok(res) => res,
        Err(e) => {
            tracing::error!("Failed to get anime from Anilist: {}", e);
            return AniListResult {
                response: Err(anyhow::Error::new(e)),
                rate_limit_current: -1,
                rate_limit_remaining: -1,
                rate_limit_reset: -1,
                retry_after: -1,
            };
        }
    };

    let rate_limit_remaining = get_header_i32(&res, "x-ratelimit-remaining".to_string(), -1);
    let rate_limit_current = get_header_i32(&res, "x-ratelimit-limit".to_string(), -1);
    let rate_limit_reset = get_header_i32(&res, "x-ratelimit-reset".to_string(), -1);
    let retry_after = get_header_i32(&res, "retry-after".to_string(), -1);

    let text = res.text().await.unwrap();
    let anime: Result<AnilistResponse, serde_json::Error> = serde_json::from_str(&text);
    let anime = match anime {
        Ok(json) => Ok(json),
        Err(e) => {
            tracing::error!("Response: {:?}", text);
            tracing::error!("Failed to parse Anilist response: {}", e);
            Err(anyhow::Error::new(e))
        }
    };

    AniListResult {
        response: anime,
        rate_limit_current,
        rate_limit_remaining,
        rate_limit_reset,
        retry_after,
    }
}

// max this can be is 17, if higher is needed, need to change the queue processor
// to ensure it can grab the data from the query response
pub const MAX_ANILIST_PER_QUERY: usize = 35;

pub fn generate_gql_query(ids: Vec<&u32>) -> GqlQuery {
    let mut ids = ids;
    if ids.len() > MAX_ANILIST_PER_QUERY {
        tracing::error!("Too many ids: {}", ids.len());
        ids.truncate(MAX_ANILIST_PER_QUERY);
    }

    let mut query = "query media(".to_owned();
    let mut variables = json!({});

    for (i, id) in ids.iter().enumerate() {
        let i = i + 1;
        query.push_str(&format!("$anime{}: Int,", i));
        let variable_name = "anime".to_owned() + &i.to_string();
        variables[variable_name] = json!(id);
    }

    query.push_str(") {");

    let media_selection = String::from(ANILIST_MEDIA_SELECTION);
    for i in 1..ids.len() + 1 {
        let media_selection = media_selection.replace("{}", &i.to_string());
        query.push_str(&media_selection);
    }

    query.push('}');

    GqlQuery { query, variables }
}

const ANILIST_MEDIA_SELECTION: &str = r#"
anime{}: Media(idMal: $anime{}, type: ANIME) {
    status
    idMal
    title {
      romaji
    }
    season
    seasonYear
    coverImage {
      large
    }
    relations {
      edges {
        relationType(version: 2)
        node {
          idMal
        }
      }
    }
  }
"#;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistItems {
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

pub fn get_anime_from_anilist_result(
    result: AnilistResponse,
    i: usize,
) -> Option<AniListAnimeItem> {
    match i {
        0 => result.data.anime1,
        1 => result.data.anime2,
        2 => result.data.anime3,
        3 => result.data.anime4,
        4 => result.data.anime5,
        5 => result.data.anime6,
        6 => result.data.anime7,
        7 => result.data.anime8,
        8 => result.data.anime9,
        9 => result.data.anime10,
        10 => result.data.anime11,
        11 => result.data.anime12,
        12 => result.data.anime13,
        13 => result.data.anime14,
        14 => result.data.anime15,
        15 => result.data.anime16,
        16 => result.data.anime17,
        17 => result.data.anime18,
        18 => result.data.anime19,
        19 => result.data.anime20,
        20 => result.data.anime21,
        21 => result.data.anime22,
        22 => result.data.anime23,
        23 => result.data.anime24,
        24 => result.data.anime25,
        25 => result.data.anime26,
        26 => result.data.anime27,
        27 => result.data.anime28,
        28 => result.data.anime29,
        29 => result.data.anime30,
        30 => result.data.anime31,
        31 => result.data.anime32,
        32 => result.data.anime33,
        33 => result.data.anime34,
        34 => result.data.anime35,
        _ => None,
    }
}
