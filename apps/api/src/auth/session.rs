use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::Deserialize;
use serde::Serialize;
use time;

use crate::AppState;

use crate::models::user::SafeUser;

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
}

pub async fn create_session(
    state: AppState,
    user_id: String,
) -> Result<Cookie<'static>, anyhow::Error> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(30))
        .expect("valid timestamp");

    let mut token_str = [0u8; 32];
    StdRng::from_entropy().fill_bytes(&mut token_str);
    let token = hex::encode(token_str);

    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .expires(
            time::OffsetDateTime::from_unix_timestamp(expiration.timestamp())
                .expect("valid timestamp"),
        )
        .max_age(time::Duration::days(30))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .build();

    let res = sqlx::query!(
        "INSERT INTO sessions (user_id, id, expires_at) VALUES (?, ?, ?)",
        user_id,
        token,
        expiration
    )
    .execute(&state.db)
    .await;

    match res {
        Ok(_) => Ok(cookie.clone()),
        Err(err) => {
            tracing::error!("Error creating session: {:?}", err);
            Err(err.into())
        }
    }
}
