pub mod api_types;
mod gql_query;
mod ratelimiter;

use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use self::api_types::{AnilistApiAnime, AnilistFetchAnimeResponse};
use self::gql_query::GqlQuery;
use self::ratelimiter::AnilistRateLimitingMiddleware;

const APILIST_API_URL: &str = "https://graphql.anilist.co/";

#[derive(thiserror::Error, Debug)]
pub enum AnilistError {
    #[error("failed to parse anilist response")]
    Parse(reqwest::Error),
    #[error("failed to send anilist request")]
    Request(reqwest_middleware::Error),
}

pub struct Anilist {
    client: ClientWithMiddleware,
}

impl Anilist {
    pub fn new() -> Self {
        let requests_per_minute = 90;
        let burst_size = 1;

        let rate_limiting_middleware =
            AnilistRateLimitingMiddleware::new(requests_per_minute, burst_size);

        let client = ClientBuilder::new(Client::new())
            .with(rate_limiting_middleware)
            .build();

        Self { client }
    }

    pub async fn fetch_animes(&self, ids: &[i32]) -> Result<Vec<AnilistApiAnime>, anyhow::Error> {
        let gql = GqlQuery::generate_gql_query(ids);
        let data: AnilistFetchAnimeResponse = self
            .client
            .post(APILIST_API_URL)
            .json(&gql)
            .send()
            .await
            .map_err(AnilistError::Request)?
            .json()
            .await
            .map_err(AnilistError::Parse)?;

        Ok(data.data.into_values().collect())
    }
}
