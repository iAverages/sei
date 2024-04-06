use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::AppState;

pub async fn create_user(app_state: AppState, user: CreateUser) -> User {
    let id = cuid::cuid2();
    let res = sqlx::query!(
        "INSERT INTO users
        (id,name,picture, mal_id, mal_access_token, mal_refresh_token)
        VALUES (?,?,?,?,?,?)",
        id,
        user.name,
        user.picture,
        user.mal_id,
        user.mal_access_token,
        user.mal_refresh_token
    )
    .execute(&app_state.db)
    .await
    .expect("Failed to create user");

    let id = res.last_insert_id();

    sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&app_state.db)
        .await
        .expect("Failed to find user")
}

pub async fn find_user_mal_id(state: AppState, mal_id: i32) -> Option<User> {
    let user = sqlx::query_as!(User, "SELECT *  FROM users WHERE mal_id = ?", mal_id)
        .fetch_one(&state.db)
        .await;

    match user {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}

pub async fn get_mal_user(state: AppState, token: String, mal_id: i32) -> MalUser {
    let mut search_id = "@me".to_string();

    if mal_id != 0 {
        search_id = mal_id.to_string();
    }

    state
        .reqwest
        .get(format!(
            "https://api.myanimelist.net/v2/users/{}",
            search_id
        ))
        .bearer_auth(token)
        .send()
        .await
        .expect("Failed to get MAL user")
        .json::<MalUser>()
        .await
        .expect("Failed to parse MAL user")
}

pub struct CreateUser {
    pub name: String,
    pub picture: String,
    pub mal_id: i32,
    pub mal_access_token: String,
    pub mal_refresh_token: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub picture: String,
    pub mal_id: i32,
    pub mal_access_token: String,
    pub mal_refresh_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

#[derive(Deserialize)]
pub struct MalUser {
    pub id: i32,
    pub name: String,
    pub picture: String,
}
