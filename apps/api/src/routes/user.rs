use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use serde_json::json;

use crate::helpers::json_response;
use crate::models::user::SafeUser;

#[axum::debug_handler]
pub async fn get_user(Extension(user): Extension<SafeUser>) -> impl IntoResponse {
    json_response!(StatusCode::OK, user)
}
