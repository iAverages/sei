use std::{collections::HashMap, sync::Arc};

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Execute, MySql, QueryBuilder};
use tower::builder;

use crate::{models::user::User, AppError};

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
    pub id: i32,
    pub title: String,
    pub main_picture: AnimePicture,
    pub status: String,
    pub broadcast: Option<AnimeBroadcast>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AnimeListItem {
    pub node: AnimeListNode,
    pub list_status: Value,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MalAnimeListResponse {
    pub data: Vec<AnimeListItem>,
    pub paging: Value,
}

pub struct AnimeTableRow {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: Option<String>,
    pub picture: String,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimeRelationshipType {
    Sequel,
    Prequel,
    AlternativeSetting,
    AlternativeVersion,
    SideStory,
    ParentStory,
    Summary,
    FullStory,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow, Clone)]
pub struct LocalAnimeRelation {
    pub base_anime_id: i32,
    pub related_anime_id: i32,
    pub relation: String,

    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: Option<String>,
    pub picture: String,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiringStatus {
    Finished,
    Releasing,
    NotYetReleased,
    Cancelled,
    Hiatus,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct LocalAnime {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: Option<String>,
    pub status: String,
    pub picture: String,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub relation: Vec<LocalAnimeRelation>,
}

pub struct DBAnime {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: Option<String>,
    pub picture: String,
    pub status: String,
    pub updated_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn get_local_anime_data(
    db: sqlx::Pool<sqlx::MySql>,
    id: i32,
) -> Result<LocalAnime, anyhow::Error> {
    let anime = sqlx::query_as!(
        AnimeTableRow,
        r#"
    SELECT * FROM animes WHERE id = ?
    "#,
        id
    )
    .fetch_one(&db)
    .await?;

    let relations = sqlx::query_as!(
        LocalAnimeRelation,
        r#"
        SELECT
            *
        FROM
            anime_relations
        INNER JOIN animes ON animes.id = anime_relations.related_anime_id
        WHERE
            base_anime_id = ?;
    "#,
        id
    )
    .fetch_all(&db)
    .await?;

    Ok(LocalAnime {
        id: anime.id,
        english_title: anime.english_title,
        romaji_title: anime.romaji_title,
        status: anime.status,
        picture: anime.picture,
        updated_at: anime.updated_at,
        created_at: anime.created_at,
        relation: relations,
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListStatus {
    Importing,
    Updating,
    Imported,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LocalAnineListResult {
    pub animes: Vec<LocalAnime>,
    pub status: ListStatus,
}

pub async fn get_local_user_list(
    db: sqlx::Pool<sqlx::MySql>,
    user: User,
) -> Result<LocalAnineListResult, anyhow::Error> {
    let animes = sqlx::query_as!(
        DBAnime,
        r#"
    SELECT animes.* FROM animes
    INNER JOIN anime_users ON anime_users.anime_id = animes.id
    WHERE anime_users.user_id = ?
    "#,
        user.id
    )
    .fetch_all(&db)
    .await?;

    if animes.is_empty() {
        return Ok(LocalAnineListResult {
            animes: vec![],
            status: ListStatus::Importing,
        });
    }

    let ids: Vec<i32> = animes.iter().map(|a| a.id).collect();

    let params = (0..ids.len()).map(|_| "?").collect::<Vec<&str>>();

    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            anime_relations
        INNER JOIN animes ON animes.id = anime_relations.related_anime_id
        WHERE
            base_anime_id IN ( 
        "#,
    );

    for (i, id) in ids.iter().enumerate() {
        query_builder.push_bind(id);
        if i < ids.len() - 1 {
            query_builder.push(", ");
        }
    }

    query_builder.push(")");

    let a = query_builder.build_query_as::<LocalAnimeRelation>();

    tracing::info!("Query: {}", a.sql().to_string());
    let relations = a.fetch_all(&db).await?;

    let mut relations_map: HashMap<i32, Vec<LocalAnimeRelation>> = HashMap::new();

    for relation in relations {
        relations_map
            .entry(relation.base_anime_id)
            .or_insert_with(Vec::new)
            .push(relation);
    }

    let local_animes: Vec<LocalAnime> = animes
        .into_iter()
        .map(|anime| {
            let relations = relations_map.get(&anime.id).cloned().unwrap_or_default();

            LocalAnime {
                id: anime.id,
                english_title: anime.english_title,
                romaji_title: anime.romaji_title,
                status: anime.status,
                picture: anime.picture,
                updated_at: anime.updated_at,
                created_at: anime.created_at,
                relation: relations,
            }
        })
        .collect();

    Ok(LocalAnineListResult {
        animes: local_animes,
        status: ListStatus::Imported,
    })
    // Ok(anime
    //     .into_iter()
    //     .map(|a| {
    //         let relations = relations.clone();
    //         let relationed_anime = relations
    //             .into_iter()
    //             .filter(|r| r.base_anime_id == a.id)
    //             .collect();

    //         LocalAnime {
    //             id: a.id,
    //             english_title: a.english_title,
    //             romaji_title: a.romaji_title,
    //             status: a.status,
    //             picture: a.picture,
    //             updated_at: a.updated_at,
    //             created_at: a.created_at,
    //             relation: relationed_anime,
    //         }
    //     })
    //     .collect())

    // let relattions = sqlx::query_as!(
    //     LocalAnimeRelation,
    //     r#"
    //     SELECT
    //         *
    //     FROM
    //         anime_relations
    //     INNER JOIN animes ON animes.id = anime_relations.related_anime_id
    //     WHERE
    //         base_anime_id IN ?
    // "#,
    //     ids
    // );
}

pub async fn get_mal_user_list(
    reqwest: Client,
    user: User,
) -> Result<MalAnimeListResponse, anyhow::Error> {
    tracing::info!("Getting MAL anime list for user {}", user.id);
    let res = reqwest
        .get("https://api.myanimelist.net/v2/users/@me/animelist?fields=list_status,node.status,node.num_episodes,node.broadcast&limit=1000&nsfw=1")
        .bearer_auth(user.mal_access_token)
        .send()
        .await
        .expect("Failed to get MAL anime");
    let anime = res.json::<MalAnimeListResponse>().await?;
    let paging = anime.paging.clone();

    let anime: Vec<AnimeListItem> = anime
        .data
        .into_iter()
        .filter(|item| item.list_status["status"] != "completed")
        .collect();

    tracing::info!("Got {} anime from MAL", anime.len());

    Ok(MalAnimeListResponse {
        data: anime,
        paging,
    })
}

const ANILIST_MEDIA_SELECTION: &str = r#"
anime{}: Media(idMal: $anime{}, type: ANIME) {
    idMal
    title {
      romaji
      english
    }
    type
    coverImage {
      large
      color
    }
    bannerImage
    status
    episodes
    nextAiringEpisode {
      id
      airingAt
      episode
    }
    airingSchedule {
      edges {
        node {
          id
          airingAt
          episode
        }
      }
    }
    relations {
      edges {
        id
        relationType(version: 2) 
        node {
          idMal
          title {
            romaji
            english
          }
          type
          coverImage {
            large
            color
          }
          bannerImage
          status
          episodes
          nextAiringEpisode {
            id
            airingAt
            episode
          }
          airingSchedule {
            edges {
              node {
                id
                airingAt
                episode
              }
            }
          }
        }
      }
  }
}
"#;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnilistResponse {
    pub data: AnilistItems,
}

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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniListAnimeItem {
    pub status: String,
    pub episodes: Option<i64>,
    pub next_airing_episode: Option<NextAiringEpisode>,
    pub airing_schedule: AiringSchedule,
    pub relations: Option<Relations>,
    pub title: Title,
    pub id_mal: i64,
    pub type_: String,
    pub cover_image: CoverImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverImage {
    pub large: String,
    pub color: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NextAiringEpisode {
    pub id: i64,
    pub airing_at: i64,
    pub episode: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringSchedule {
    pub edges: Vec<AiringScheduleEdge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringScheduleEdge {
    pub node: AiringScheduleNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringScheduleNode {
    pub id: i64,
    pub airing_at: i64,
    pub episode: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relations {
    pub edges: Vec<RelationsEdge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationsEdge {
    pub id: i64,
    pub relation_type: String,
    pub node: RelationsNode,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationsNode {
    pub status: String,
    pub episodes: Option<i64>,
    pub next_airing_episode: Option<NextAiringEpisode>,
    pub airing_schedule: AiringSchedule,
    pub relations: Option<Relations>,
    pub title: Title,
    pub id_mal: Option<i64>,
    pub type_: String,
    pub cover_image: CoverImage,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Title {
    pub romaji: Option<String>,
    pub english: Option<String>,
}

pub struct AniListResult {
    pub response: Result<AnilistResponse, anyhow::Error>,
    pub rate_limit_remaining: i32,
    pub rate_limit_current: i32,
    pub rate_limit_reset: i32,
    pub retry_after: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct GqlQuery {
    query: String,
    variables: Value,
}

fn generate_gql_query(ids: Vec<i32>) -> GqlQuery {
    let mut ids = ids;
    if ids.len() > 10 {
        tracing::error!("Too many ids: {}", ids.len());
        ids.truncate(10);
    }

    let mut query = "query media(".to_owned();
    let media_selection = String::from(ANILIST_MEDIA_SELECTION);
    let mut variables = json!({});

    for (i, id) in ids.iter().enumerate() {
        let i = i + 1;
        query.push_str(&format!("$anime{}: Int,", i));
        let variable_name = "anime".to_owned() + &i.to_string();
        variables[variable_name] = json!(id);
    }

    query.push_str(") {");

    for i in 1..ids.len() + 1 {
        let media_selection = media_selection.replace("{}", &i.to_string());
        query.push_str(&media_selection);
    }

    query.push('}');

    tracing::info!("Query: {}", query);

    GqlQuery { query, variables }
}

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

pub async fn get_animes_from_anilist(reqwest: Client, ids: Vec<i32>) -> AniListResult {
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
    tracing::error!("json: {}", text);

    let anime: Result<AnilistResponse, serde_json::Error> = serde_json::from_str(&text);

    AniListResult {
        response: anime.map_err(anyhow::Error::new),
        rate_limit_current,
        rate_limit_remaining,
        rate_limit_reset,
        retry_after,
    }
}
