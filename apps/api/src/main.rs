mod helpers;
mod models;
mod routes;

use std::net::SocketAddr;

use axum::{extract::FromRef, http::Method, routing::get, Extension, Router};
use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use reqwest::Client;
use sqlx::mysql::MySqlPoolOptions;
use tower_http::cors::{Any, CorsLayer};

fn create_oauth_client(api_url: String, client_id: String, client_secret: String) -> BasicClient {
    let redirect_url = api_url + "/oauth/mal/callback";
    let auth_url = AuthUrl::new("https://myanimelist.net/v1/oauth2/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://myanimelist.net/v1/oauth2/token".to_string())
        .expect("Invalid token endpoint URL");

    BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL"))
}

#[derive(Clone)]
pub struct AppState {
    key: Key,
    db: sqlx::Pool<sqlx::MySql>,
    reqwest: Client,
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
        .allow_origin(Any);

    let api_url = std::env::var("APP_URL").unwrap_or("http://localhost:3001".to_string());
    let mal_client_id = std::env::var("MAL_CLIENT_ID").expect("MAL_CLIENT_ID not set");
    let mal_client_secret = std::env::var("MAL_CLIENT_SECRET").expect("MAL_CLIENT_SECRET not set");

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    let reqwest = Client::new();

    let state = AppState {
        key: Key::generate(),
        db: db_pool,
        reqwest,
    };

    let oauth_client =
        create_oauth_client(api_url.clone(), mal_client_id.clone(), mal_client_secret);

    let app = Router::new()
        .route("/api/auth/me", get(routes::user::get_user))
        .route(
            "/oauth/mal/redirect",
            get(routes::oauth::handle_mal_redirect),
        )
        .route(
            "/oauth/mal/callback",
            get(routes::oauth::handle_mal_callback),
        )
        .layer(Extension(oauth_client))
        .layer(cors)
        .with_state(state);

    let address = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    tracing::info!("listening on {}", address);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
