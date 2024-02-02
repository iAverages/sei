use axum::{
    extract::{Query, State},
    response::Redirect,
    Extension, Json,
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, TokenResponse,
};
use serde::{Deserialize, Serialize};

use crate::models::user::{create_user, find_user_mal_id, get_mal_user, CreateUser, User};
use crate::AppState;

#[derive(Deserialize)]
pub struct MalRedirectQuery {
    code: String,
}

#[axum::debug_handler]
pub async fn handle_mal_redirect(
    State(state): State<AppState>,
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
    jar: PrivateCookieJar,
    Extension(oauth_client): Extension<BasicClient>,
) -> Json<User> {
    let csrf_token = jar
        .get("mal_csrf_token")
        .expect("No CSRF token in cookie jar");
    let pkce_verifier_str = jar
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
        Some(user) => user,
        None => {
            create_user(
                state,
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

    Json(user)
}
