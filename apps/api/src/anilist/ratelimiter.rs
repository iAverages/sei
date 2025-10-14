use async_rate_limiter::RateLimiter;
use async_trait::async_trait;
use axum::http::Extensions;
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::{Middleware, Next, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct AnilistRateLimitingMiddleware {
    rate_limiter: Arc<RateLimiter>,
}

impl AnilistRateLimitingMiddleware {
    pub fn new(requests_per_minute: u64, burst_size: u64) -> Self {
        let rate_limiter = Arc::new(RateLimiter::new(requests_per_minute as usize));
        rate_limiter.burst(burst_size as usize);
        AnilistRateLimitingMiddleware { rate_limiter }
    }
}

#[async_trait]
impl Middleware for AnilistRateLimitingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        loop {
            self.rate_limiter.acquire().await;

            let response = next
                .clone()
                .run(req.try_clone().unwrap(), extensions)
                .await?;

            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                if let Some(retry_after_header) = response.headers().get("Retry-After") {
                    if let Ok(s) = retry_after_header.to_str() {
                        if let Ok(seconds) = s.parse::<u64>() {
                            tracing::debug!("anilist client, retry-after {} seconds", seconds);
                            sleep(Duration::from_secs(seconds)).await;
                        }
                    }
                }
                continue;
            }
            return Ok(response);
        }
    }
}
