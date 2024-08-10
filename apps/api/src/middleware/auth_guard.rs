use crate::models::user::{get_user_by_session, SafeUser};
use crate::AppState;

use axum::extract::{Request, State};
use axum_extra::extract::CookieJar;

use axum::{http::StatusCode, middleware::Next, response::Response};

pub async fn guard(
    State(state): State<AppState>,
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = jar.get("token").map(|cookie| cookie.value());

    if token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = get_user_by_session(state, token.unwrap().to_string()).await;

    if user.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = user.unwrap();

    request.extensions_mut().insert(SafeUser {
        id: user.id,
        name: user.name,
        picture: user.picture,
        mal_id: user.mal_id,
        created_at: user.created_at,
    });

    Ok(next.run(request).await)
}
