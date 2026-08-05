mod anilist;
mod auth;
mod consts;
mod database;
mod helpers;
mod importer;
mod mal;
mod middleware;
mod models;
mod routes;
use std::fmt::{self, Display, Formatter};

use axum::{
    extract::{FromRef, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
    Extension, Json, Router,
};
use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use helpers::json_response;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use tokio::sync::broadcast;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};

use crate::{auth::oauth::create_oauth_client, importer::Importer, middleware::auth_guard::guard};

use self::models::anime::FullAnime;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ImportEvent {
    user_id: String,
    anime: FullAnime,
    list_id: String, // TODO: not used yet
}

#[derive(Clone)]
pub struct AppState {
    key: Key,
    db: sqlx::Pool<sqlx::MySql>,
    reqwest: Client,
    importer: Importer,
    tx: broadcast::Sender<ImportEvent>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[axum::debug_handler]
async fn test(State(state): State<AppState>) -> impl IntoResponse {
    let _ = state.importer.add_items(vec![2025], None).await;
    (StatusCode::OK, "hi")
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting server...");

    // TODO: improve env validation handling
    let app_origin = std::env::var("APP_ORIGIN")
        .or_else(|_| std::env::var("API_URL"))
        .unwrap_or("http://localhost:3000".to_string());
    let mal_client_id = std::env::var("MAL_CLIENT_ID").expect("MAL_CLIENT_ID not set");
    let mal_client_secret = std::env::var("MAL_CLIENT_SECRET").expect("MAL_CLIENT_SECRET not set");

    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_credentials(true)
        .allow_headers(AllowHeaders::mirror_request())
        .allow_origin(AllowOrigin::exact(
            HeaderValue::from_str(app_origin.as_str()).unwrap(),
        ));

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let (tx, _rx) = broadcast::channel(256);
    let reqwest = Client::new();
    let mut importer = Importer::new(reqwest.clone(), db_pool.clone(), tx.clone());

    let state = AppState {
        key: Key::generate(),
        db: db_pool,
        reqwest,
        importer: importer.clone(),
        tx,
    };

    importer.start();

    let oauth_client = create_oauth_client(app_origin, mal_client_id.clone(), mal_client_secret);

    let protected_api = Router::new()
        .route("/test", get(test))
        .route("/auth/me", get(routes::user::get_user))
        .route("/user/import-status", get(routes::user::get_import_status))
        .route("/user/list", get(routes::user::get_list))
        .route("/user/list", post(routes::user::update_list_order))
        .route("/user/list/sse", get(routes::user::join_sse))
        .route("/lists", get(routes::list::get_lists))
        .route("/lists", post(routes::list::create_list))
        .route("/lists/:list_id", get(routes::list::get_list))
        .route("/lists/:list_id", patch(routes::list::update_list))
        .route("/lists/:list_id/entries", post(routes::list::add_entries))
        .route(
            "/lists/:list_id/entries/:anime_id",
            delete(routes::list::delete_entry),
        )
        .route("/lists/:list_id/order", put(routes::list::update_order))
        .route("/anime/search", get(routes::list::search_anime))
        .route_layer(from_fn_with_state(state.clone(), guard));

    let api = Router::new()
        .merge(protected_api)
        .route("/public/lists/:list_id", get(routes::list::get_public_list))
        .with_state(state.clone());

    let app = Router::new()
        .nest("/api/v1", api)
        .route(
            "/api/oauth/mal/redirect",
            get(routes::auth::handle_mal_redirect),
        )
        .route(
            "/api/oauth/mal/callback",
            get(routes::auth::handle_mal_callback),
        )
        .route("/api/health", get(|| async { StatusCode::NO_CONTENT }))
        .layer(Extension(oauth_client))
        .layer(cors)
        .with_state(state.clone());

    let address = std::env::var("BIND_ADDR").unwrap_or("0.0.0.0:3001".to_string());
    let listener = tokio::net::TcpListener::bind(&address).await.unwrap();
    tracing::info!("listening on {}", address);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// TODO: use this better for returning friendly errors to api response

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        json_response!(StatusCode::INTERNAL_SERVER_ERROR, {"message":"Internal Server Error"})
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
