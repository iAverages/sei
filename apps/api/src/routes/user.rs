use axum::{http::StatusCode, response::IntoResponse};

use crate::helpers::json_response;
use axum::Json;
use serde_json::json;

#[axum::debug_handler]
pub async fn get_user() -> impl IntoResponse {
    // (StatusCode::OK, "Hello, World!").into_response()
    json_response!(StatusCode::OK, {
        "id": 1,
        "name": "John Doe",
        "email": ""
    })
}
