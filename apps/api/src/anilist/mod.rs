pub mod api_types;
pub mod gql_query;
mod ratelimiter;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::vec;

use reqwest::{retry, Client, Response};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

use self::api_types::{AnilistApiAnime, AnilistFetchAnimeErrorResponse, AnilistFetchAnimeResponse};
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

        // TODO: check this works correctly
        let rate_limiting_middleware =
            AnilistRateLimitingMiddleware::new(requests_per_minute, burst_size);

        let client = ClientBuilder::new(Client::new())
            .with(rate_limiting_middleware)
            .build();

        Self { client }
    }

    // TODO: handle case where anilist does not have anime given a malid
    // they are dumb and error every requested anime instead of just the one that errored
    pub fn fetch_animes(
        self: Arc<Self>,
        ids: &[i32],
        retry: bool,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<AnilistApiAnime>, anyhow::Error>> + Send>> {
        let ids = ids.to_vec();
        let this = self.clone();
        Box::pin(async move {
            let gql = GqlQuery::generate_gql_query(&ids);
            // let data: AnilistFetchAnimeResponse =
            let response = this.client.post(APILIST_API_URL).json(&gql).send().await?;
            let text = response.text().await?;
            let success_result: Result<AnilistFetchAnimeResponse, _> = serde_json::from_str(&text);

            Ok(match success_result {
                Ok(response) => response.data.into_values().collect::<Vec<_>>(),
                Err(_) => {
                    if retry {
                        let errored_ids = this.find_failed_ids(&ids, text, gql.query);
                        let retry_ids = ids
                            .into_iter()
                            .filter(|id| {
                                errored_ids
                                    .iter()
                                    .find(|error_id| *error_id == id)
                                    .is_none()
                            })
                            .collect::<Vec<_>>();
                        // TODO: fix unwrap
                        this.fetch_animes(&retry_ids, false).await.unwrap()
                    } else {
                        vec![]
                    }
                }
            })
        })
    }

    fn find_failed_ids(&self, ids: &[i32], body_response: String, gql_query: String) -> Vec<i32> {
        let mut failed_ids = vec![];
        let error_result: Result<AnilistFetchAnimeErrorResponse, _> =
            serde_json::from_str(&body_response);
        if let Ok(json) = error_result {
            // only handle first for now, never seen it return multiple errors
            // for the requests we do
            let error = json.errors.first();
            if let Some(error) = error {
                if error.status == 404 {
                    // same here, never seen it return multiple
                    let loc = error.locations.first();
                    if let Some(loc) = loc {
                        let total = gql_query.lines().count();
                        let lpq = total / ids.len();
                        let error_index = (loc.line - 1) as usize / lpq;
                        let errored_item = ids.get(error_index).expect("No item for error given");
                        tracing::info!(anime_id = errored_item, "anime was not found on anilist",);
                        failed_ids.push(errored_item.clone());
                    }
                }
            }
        }

        failed_ids
    }
}
