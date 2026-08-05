use chrono::{NaiveDateTime, Utc};
use sea_query::{MysqlQueryBuilder, OnConflict, Query};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};

use crate::auth::session::Session;
use crate::database::models::AnimeUsers;
use crate::mal::get_mal_user_list;
use crate::AppState;

pub async fn create_user(app_state: AppState, user: CreateUser) -> DBUser {
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

    sqlx::query_as!(DBUser, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&app_state.db)
        .await
        .expect("Failed to find user")
}

pub async fn find_user_mal_id(state: AppState, mal_id: i32) -> Option<DBUser> {
    let user = sqlx::query_as!(DBUser, "SELECT *  FROM users WHERE mal_id = ?", mal_id)
        .fetch_one(&state.db)
        .await;

    match user {
        Ok(user) => Some(user),
        Err(_) => None,
    }
}

pub async fn get_user_by_session(state: AppState, session_id: String) -> Option<DBUser> {
    let session = sqlx::query_as!(Session, "SELECT * FROM sessions WHERE id = ?", session_id)
        .fetch_one(&state.db)
        .await
        .ok()?;

    sqlx::query_as!(DBUser, "SELECT * FROM users WHERE id = ?", session.user_id)
        .fetch_one(&state.db)
        .await
        .ok()
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
pub struct DBUser {
    pub id: String,
    pub name: String,
    pub picture: String,
    pub mal_id: i32,
    pub mal_access_token: String,
    pub mal_refresh_token: String,
    pub list_last_update: NaiveDateTime,
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

#[derive(Deserialize, Serialize, Clone)]
pub struct SafeUser {
    pub id: String,
    pub name: String,
    pub picture: String,
    pub mal_id: i32,
    pub created_at: NaiveDateTime,
}

impl From<DBUser> for SafeUser {
    fn from(user: DBUser) -> Self {
        SafeUser {
            created_at: user.created_at,
            mal_id: user.mal_id,
            picture: user.picture,
            id: user.id,
            name: user.name,
        }
    }
}

pub async fn is_user_importing(db: &Pool<MySql>, user_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_file!("database/queries/is_user_importing.sql", user_id)
        .fetch_one(db)
        .await?;

    Ok(result.importing_count > 0)
}

pub struct AnimeListEntry {
    anime_id: i32,
    status: String, // TODO: replace with enum for values
    watch_priority: i32,
}

pub async fn add_to_list(db: &Pool<MySql>, user_id: &str, add_entries: Vec<AnimeListEntry>) {
    let (sql, values) = Query::insert()
        .into_table(AnimeUsers::Table)
        .columns([
            AnimeUsers::UserId,
            AnimeUsers::AnimeId,
            AnimeUsers::Status,
            AnimeUsers::WatchPriority,
            AnimeUsers::UpdatedAt,
        ])
        .values_from_panic(add_entries.iter().map(|entry| {
            [
                user_id.into(),
                entry.anime_id.into(),
                entry.status.clone().into(),
                entry.watch_priority.into(),
                Utc::now().into(),
            ]
        }))
        .on_conflict(
            OnConflict::columns([AnimeUsers::UserId, AnimeUsers::AnimeId])
                .update_column(AnimeUsers::Status)
                .update_column(AnimeUsers::UpdatedAt)
                .to_owned(),
        )
        .build_sqlx(MysqlQueryBuilder);

    match sqlx::query_with(&sql, values).execute(db).await {
        Ok(_) => {
            tracing::info!("added animes to user list");
            if cfg!(debug_assertions) {
                add_entries.iter().for_each(|entry| {
                    tracing::trace!(anime = entry.anime_id, "added anime to user list");
                });
            }
        }
        Err(_) => {
            tracing::info!("added animes to user list");
        }
    };
}

pub async fn update_list_entries_mal(state: AppState, user: DBUser) {
    let user_id = user.id.clone();
    let response = get_mal_user_list(state.reqwest, user).await;
    if let Err(error) = response {
        tracing::error!(
            error = error.to_string(),
            "failed to fetch mal list for user"
        );
        return;
    }

    let animes = response.unwrap();
    let _ = state
        .importer
        .add_items(
            animes.data.iter().map(|anime| anime.node.id).collect(),
            Some(&user_id),
        )
        .await;

    add_to_list(
        &state.db,
        &user_id,
        animes
            .data
            .into_iter()
            .map(|anime| AnimeListEntry {
                anime_id: anime.node.id,
                status: anime.list_status.status,
                watch_priority: 0,
            })
            .collect(),
    )
    .await;

    if let Err(error) =
        sqlx::query("UPDATE users SET list_last_update = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&user_id)
            .execute(&state.db)
            .await
    {
        tracing::error!(
            error = error.to_string(),
            "failed to update MAL list sync time"
        );
    }
}
