use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::{
    cookie::{Cookie, PrivateCookieJar, SameSite},
    CookieJar,
};
use chrono::Utc;
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, TokenResponse,
};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::Deserialize;
use time;

use crate::{anime::get_mal_user_list, AppState};
use crate::{
    importer,
    models::user::{create_user, find_user_mal_id, get_mal_user, CreateUser, User},
};

pub async fn create_session(state: AppState, user: User) -> Result<Cookie<'static>, anyhow::Error> {
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
        user.id,
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

#[derive(Deserialize)]
pub struct MalRedirectQuery {
    code: String,
}

#[axum::debug_handler]
pub async fn handle_mal_redirect(
    State(_): State<AppState>,
    jar: PrivateCookieJar,
    Extension(oauth_client): Extension<BasicClient>,
) -> Result<(PrivateCookieJar, Redirect), Redirect> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_plain();

    let (auth_url, csrf_token) = oauth_client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(pkce_challenge)
        .url();

    let updated_jar = jar
        .add(Cookie::new("mal_csrf_token", csrf_token.secret().clone()))
        .add(Cookie::new(
            "mal_pkce_verifier",
            pkce_verifier.secret().to_string(),
        ));

    Ok((updated_jar, Redirect::temporary(auth_url.as_str())))
}

#[axum::debug_handler]
pub async fn handle_mal_callback(
    Query(query): Query<MalRedirectQuery>,
    State(state): State<AppState>,
    private_jar: PrivateCookieJar,
    jar: CookieJar,
    Extension(oauth_client): Extension<BasicClient>,
) -> impl IntoResponse {
    let csrf_token = private_jar
        .get("mal_csrf_token")
        .expect("No CSRF token in cookie jar");
    let pkce_verifier_str = private_jar
        .get("mal_pkce_verifier")
        .expect("No PKCE verifier in cookie jar");

    let pkce_verifier = PkceCodeVerifier::new(pkce_verifier_str.value().to_string());

    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .unwrap();

    let token = token_result.access_token().secret().to_string();
    let mal_user = get_mal_user(state.clone(), token.clone(), 0).await;
    let mal_user_id = mal_user.id;

    let user = find_user_mal_id(state.clone(), mal_user_id).await;

    let user = match user {
        Some(user) => {
            sqlx::query!(
                "UPDATE users SET mal_access_token = ? WHERE id = ?",
                token,
                user.id
            )
            .execute(&state.db)
            .await
            .expect("Failed to update user");

            user
        }
        None => {
            create_user(
                state.clone(),
                CreateUser {
                    name: mal_user.name,
                    picture: mal_user.picture,
                    mal_id: mal_user.id,
                    mal_access_token: token.clone(),
                    mal_refresh_token: "".to_string(),
                },
            )
            .await
        }
    };

    let cookie = create_session(state.clone(), user.clone()).await.unwrap();

    let updated_jar = jar.add(cookie);

    let reqwest = state.reqwest.clone();
    let mal_user_list = get_mal_user_list(reqwest, user).await;

    if let Ok(mal) = mal_user_list {
        let ids = mal.data.iter().map(|item| item.node.id).collect::<Vec<_>>();
        importer::import_anime_from_ids(state, ids);
    }

    (updated_jar, Redirect::temporary("http://localhost:3000"))
}
