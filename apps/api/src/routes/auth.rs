use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::{
    cookie::{Cookie, PrivateCookieJar},
    CookieJar,
};
use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthorizationCode, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, TokenResponse,
};
use serde::Deserialize;

use crate::AppState;
use crate::{
    auth::session::create_session,
    models::user::{create_user, find_user_mal_id, get_mal_user, CreateUser},
};

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
            // Ensure the user has the latest token
            sqlx::query!(
                "UPDATE users SET mal_access_token = ? WHERE id = ?",
                token,
                user.id
            )
            .execute(&state.db)
            .await
            .expect("Failed to update user token");

            // Fetch user again with updated token
            // TODO: Update the token in the user object instead of fetching it again
            find_user_mal_id(state.clone(), mal_user_id)
                .await
                .expect("Failed to find user")
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

    let cookie = create_session(state.clone(), user.id).await.unwrap();

    let updated_jar = jar.add(cookie);

    // let reqwest = state.reqwest.clone();
    // let mal_user_list = get_mal_user_list(reqwest, user).await;
    //
    // if let Ok(mal) = mal_user_list {
    //     let ids = mal.data.iter().map(|item| item.node.id).collect::<Vec<_>>();
    //     importer::import_anime_from_ids(state, ids);
    // }

    (updated_jar, Redirect::temporary("http://localhost:3000"))
}
