mod anilist;
mod auth;
mod helpers;
mod importer;
mod mal;
mod middleware;
mod models;
mod routes;
use std::{
    fmt::{self, Display, Formatter},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use axum::{
    extract::{FromRef, State},
    http::{HeaderValue, Method, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Extension, Json, Router,
};
use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use helpers::json_response;
use reqwest::Client;
use serde_json::json;
use sqlx::mysql::MySqlPoolOptions;
use tokio::{sync::Mutex, time};
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tower_http::services::ServeDir;

use crate::middleware::auth_guard::guard;

use self::{auth::oauth::create_oauth_client, importer::Importer, models::user::DBUser};

#[axum::debug_handler]
async fn debug_route(State(state): State<AppState>) -> impl IntoResponse {
    let importer = state.importer.lock().await;
    json_response!(StatusCode::OK, {
        "queue": importer.stats()
    })
}

#[axum::debug_handler]
async fn test_handler(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> impl IntoResponse {
    let user_id = user.id;
    let mut importer = state.importer.lock().await;
    // importer.add(2024, user_id.clone());
    // importer.add(36098, user_id.clone());
    // importer.add(59226, user_id.clone());
    // importer.add(59027, user_id);

    json_response!(StatusCode::OK, {
        "queue": importer.stats()
    })
}

#[derive(Clone)]
pub struct AppState {
    key: Key,
    db: sqlx::Pool<sqlx::MySql>,
    reqwest: Client,
    importer: Arc<Mutex<Importer>>,
}

impl FromRef<AppState> for sqlx::Pool<sqlx::MySql> {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Client {
    fn from_ref(state: &AppState) -> Self {
        state.reqwest.clone()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    tracing::info!("Starting server...");

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_credentials(true)
        .allow_headers(AllowHeaders::mirror_request())
        .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "http://localhost:3000",
        )));

    let api_url = std::env::var("API_URL").unwrap_or("http://localhost:3001".to_string());
    let mal_client_id = std::env::var("MAL_CLIENT_ID").expect("MAL_CLIENT_ID not set");
    let mal_client_secret = std::env::var("MAL_CLIENT_SECRET").expect("MAL_CLIENT_SECRET not set");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let reqwest = Client::new();
    let importer = Arc::new(Mutex::new(Importer::new(reqwest.clone(), db_pool.clone())));

    let state = AppState {
        key: Key::generate(),
        db: db_pool,
        reqwest,
        importer: importer.clone(),
    };

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(2000));

        loop {
            interval.tick().await;
            let mut importer_ref = importer.lock().await;
            importer_ref.process().await;
        }
    });

    let oauth_client =
        create_oauth_client(api_url.clone(), mal_client_id.clone(), mal_client_secret);

    let app = Router::new()
        .nest_service("/", ServeDir::new("public"))
        .nest(
            "/api/v1",
            Router::new()
                .route("/test", get(test_handler))
                .route("/debug", get(debug_route))
                .route("/auth/me", get(routes::user::get_user))
                // .route("/anime", get(routes::anime::get_anime_list))
                // .route("/order", post(routes::anime::update_list_order))
                .route_layer(from_fn_with_state(state.clone(), guard))
                // .route("/anime/:id", get(routes::anime::get_anime))
                // .route(
                //     "/anime/:id/relations",
                //     get(routes::anime::get_anime_relations),
                // )
                // .route(
                //     "/anime/:id/import",
                //     get(routes::anime::get_anime_force_import),
                // )
                .with_state(state.clone()),
        )
        .route(
            "/oauth/mal/redirect",
            get(routes::auth::handle_mal_redirect),
        )
        .route(
            "/oauth/mal/callback",
            get(routes::auth::handle_mal_callback),
        )
        .layer(Extension(oauth_client))
        .layer(cors)
        .with_state(state.clone());

    let address = SocketAddr::from(([0, 0, 0, 0], 3001));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on {}", address);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

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
