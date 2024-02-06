use crate::{
    models::user::User,
    types::{CurrentUser, Session},
    AppState,
};

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

    tracing::debug!("Token: {:?}", token);

    if token.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let session = match sqlx::query_as!(Session, "SELECT * FROM sessions WHERE id = ?", token)
        .fetch_one(&state.db)
        .await
    {
        Ok(user) => user,
        Err(_) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let user = match sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", session.user_id,)
        .fetch_one(&state.db)
        .await
    {
        Ok(user) => user,
        Err(_) => {
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    request.extensions_mut().insert(CurrentUser {
        id: session.user_id,
        name: user.name,
        mal_id: user.mal_id,
        created_at: user.created_at,
        updated_at: user.updated_at,
    });

    Ok(next.run(request).await)
}
