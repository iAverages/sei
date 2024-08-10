use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use serde_json::json;

use crate::helpers::json_response;
use crate::models::user::{DBUser, SafeUser};

#[axum::debug_handler]
pub async fn get_user(Extension(user): Extension<DBUser>) -> impl IntoResponse {
    let safe_user: SafeUser = user.into();
    json_response!(StatusCode::OK, safe_user)
}
