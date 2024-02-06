use axum::Json;
use axum::{http::StatusCode, response::IntoResponse, Extension};
use serde_json::json;

use crate::{helpers::json_response, types::CurrentUser};

#[axum::debug_handler]
pub async fn get_user(Extension(user): Extension<CurrentUser>) -> impl IntoResponse {
    json_response!(StatusCode::OK, { "user": user })
}
