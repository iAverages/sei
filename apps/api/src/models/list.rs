use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, MySql, Pool, QueryBuilder, Transaction};

pub const DEFAULT_LIST_ID: &str = "default";
const MAX_ANIME_IDS: usize = 1000;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ListVisibility {
    Private,
    Unlisted,
    Public,
}

impl ListVisibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Private => "PRIVATE",
            Self::Unlisted => "UNLISTED",
            Self::Public => "PUBLIC",
        }
    }

    pub fn from_db(value: &str) -> Result<Self, anyhow::Error> {
        match value.to_ascii_uppercase().as_str() {
            "PRIVATE" => Ok(Self::Private),
            "UNLISTED" => Ok(Self::Unlisted),
            "PUBLIC" => Ok(Self::Public),
            _ => Err(anyhow::anyhow!("unknown list visibility: {value}")),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSummary {
    pub id: String,
    pub name: String,
    pub visibility: ListVisibility,
    pub is_default: bool,
}

impl ListSummary {
    pub fn default_list(visibility: ListVisibility) -> Self {
        Self {
            id: DEFAULT_LIST_ID.to_string(),
            name: "Default".to_string(),
            visibility,
            is_default: true,
        }
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAnime {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: String,
    pub status: String,
    pub picture: Option<String>,
    pub season: Option<String>,
    pub season_year: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ListDetail {
    pub list: ListSummary,
    pub anime: Vec<ListAnime>,
}

#[derive(FromRow)]
struct ListRow {
    id: String,
    name: String,
    visibility: String,
}

impl TryFrom<ListRow> for ListSummary {
    type Error = anyhow::Error;

    fn try_from(row: ListRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            name: row.name,
            visibility: ListVisibility::from_db(&row.visibility)?,
            is_default: false,
        })
    }
}

pub fn validate_name(name: String) -> Result<String, &'static str> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("List name cannot be empty");
    }
    if name.chars().count() > 100 {
        return Err("List name must be at most 100 characters");
    }
    Ok(name)
}

pub fn unique_anime_ids(ids: Vec<i32>) -> Result<Vec<i32>, &'static str> {
    if ids.len() > MAX_ANIME_IDS {
        return Err("A request can contain at most 1000 anime IDs");
    }

    let unique: HashSet<_> = ids.iter().copied().collect();
    if unique.len() != ids.len() {
        return Err("Anime IDs must be unique");
    }
    Ok(ids)
}

pub fn validate_exact_order(ids: &[i32], current: &[i32]) -> Result<(), &'static str> {
    if ids.len() != current.len() {
        return Err("IDs must exactly match the current list contents");
    }
    let requested: HashSet<_> = ids.iter().copied().collect();
    if requested.len() != ids.len() || !current.iter().all(|id| requested.contains(id)) {
        return Err("IDs must be unique and exactly match the current list contents");
    }
    Ok(())
}

pub async fn get_owned_lists(
    db: &Pool<MySql>,
    owner_id: &str,
) -> Result<Vec<ListSummary>, anyhow::Error> {
    let default_list = get_default_summary(db, owner_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("default list owner not found"))?;
    let rows = sqlx::query_as::<_, ListRow>(
        "SELECT id, name, visibility FROM anime_lists WHERE owner_id = ? ORDER BY created_at, id",
    )
    .bind(owner_id)
    .fetch_all(db)
    .await?;

    let mut lists = Vec::with_capacity(rows.len() + 1);
    lists.push(default_list);
    lists.extend(
        rows.into_iter()
            .map(ListSummary::try_from)
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(lists)
}

pub async fn get_default_summary(
    db: &Pool<MySql>,
    owner_id: &str,
) -> Result<Option<ListSummary>, anyhow::Error> {
    let visibility: Option<String> =
        sqlx::query_scalar("SELECT default_list_visibility FROM users WHERE id = ?")
            .bind(owner_id)
            .fetch_optional(db)
            .await?;
    visibility
        .map(|visibility| ListVisibility::from_db(&visibility).map(ListSummary::default_list))
        .transpose()
}

pub async fn get_public_default_summary(
    db: &Pool<MySql>,
    owner_id: &str,
) -> Result<Option<ListSummary>, anyhow::Error> {
    let visibility: Option<String> = sqlx::query_scalar(
        "SELECT default_list_visibility FROM users WHERE id = ? AND default_list_visibility IN ('PUBLIC', 'UNLISTED')",
    )
    .bind(owner_id)
    .fetch_optional(db)
    .await?;
    visibility
        .map(|visibility| ListVisibility::from_db(&visibility).map(ListSummary::default_list))
        .transpose()
}

pub async fn get_owned_summary(
    db: &Pool<MySql>,
    list_id: &str,
    owner_id: &str,
) -> Result<Option<ListSummary>, anyhow::Error> {
    let row = sqlx::query_as::<_, ListRow>(
        "SELECT id, name, visibility FROM anime_lists WHERE id = ? AND owner_id = ?",
    )
    .bind(list_id)
    .bind(owner_id)
    .fetch_optional(db)
    .await?;
    row.map(ListSummary::try_from).transpose()
}

pub async fn get_public_summary(
    db: &Pool<MySql>,
    list_id: &str,
) -> Result<Option<ListSummary>, anyhow::Error> {
    let row = sqlx::query_as::<_, ListRow>(
        "SELECT id, name, visibility FROM anime_lists WHERE id = ? AND visibility IN ('PUBLIC', 'UNLISTED')",
    )
    .bind(list_id)
    .fetch_optional(db)
    .await?;
    row.map(ListSummary::try_from).transpose()
}

pub async fn get_default_anime(
    db: &Pool<MySql>,
    owner_id: &str,
) -> Result<Vec<ListAnime>, anyhow::Error> {
    sqlx::query_as::<_, ListAnime>(
        r#"
        SELECT a.id, a.english_title, a.romaji_title, a.status, a.picture, a.season, a.season_year
        FROM anime_users au
        JOIN animes a ON a.id = au.anime_id
        WHERE au.user_id = ?
          AND UPPER(au.status) IN ('PLAN_TO_WATCH', 'WATCHING', 'ON_HOLD')
          AND a.romaji_title IS NOT NULL
          AND a.status IS NOT NULL
          AND UPPER(a.status) NOT IN ('NOT_YET_RELEASED', 'CANCELLED')
        ORDER BY CASE WHEN au.watch_priority = 0 THEN 1 ELSE 0 END, au.watch_priority, a.id
        "#,
    )
    .bind(owner_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn get_default_anime_for_update(
    transaction: &mut Transaction<'_, MySql>,
    owner_id: &str,
) -> Result<Vec<ListAnime>, anyhow::Error> {
    sqlx::query_as::<_, ListAnime>(
        r#"
        SELECT a.id, a.english_title, a.romaji_title, a.status, a.picture, a.season, a.season_year
        FROM anime_users au
        JOIN animes a ON a.id = au.anime_id
        WHERE au.user_id = ?
          AND UPPER(au.status) IN ('PLAN_TO_WATCH', 'WATCHING', 'ON_HOLD')
          AND a.romaji_title IS NOT NULL
          AND a.status IS NOT NULL
          AND UPPER(a.status) NOT IN ('NOT_YET_RELEASED', 'CANCELLED')
        ORDER BY CASE WHEN au.watch_priority = 0 THEN 1 ELSE 0 END, au.watch_priority, a.id
        FOR UPDATE
        "#,
    )
    .bind(owner_id)
    .fetch_all(&mut **transaction)
    .await
    .map_err(Into::into)
}

pub async fn get_custom_anime(
    db: &Pool<MySql>,
    list_id: &str,
) -> Result<Vec<ListAnime>, anyhow::Error> {
    sqlx::query_as::<_, ListAnime>(
        r#"
        SELECT a.id, a.english_title, a.romaji_title, a.status, a.picture, a.season, a.season_year
        FROM anime_list_entries entry
        JOIN animes a ON a.id = entry.anime_id
        WHERE entry.list_id = ?
        ORDER BY entry.position, a.id
        "#,
    )
    .bind(list_id)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}

pub async fn require_hydrated_anime(
    db: &Pool<MySql>,
    ids: &[i32],
) -> Result<Vec<ListAnime>, anyhow::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::new(
        "SELECT id, english_title, romaji_title, status, picture, season, season_year FROM animes WHERE romaji_title IS NOT NULL AND status IS NOT NULL AND id IN (",
    );
    let mut separated = query.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");
    query
        .build_query_as::<ListAnime>()
        .fetch_all(db)
        .await
        .map_err(Into::into)
}

pub fn order_anime(anime: Vec<ListAnime>, ids: &[i32]) -> Vec<ListAnime> {
    let mut by_id: HashMap<_, _> = anime.into_iter().map(|anime| (anime.id, anime)).collect();
    ids.iter().filter_map(|id| by_id.remove(id)).collect()
}

pub async fn insert_entries(
    transaction: &mut Transaction<'_, MySql>,
    list_id: &str,
    ids: &[i32],
    starting_position: i32,
) -> Result<(), anyhow::Error> {
    if ids.is_empty() {
        return Ok(());
    }

    let mut query =
        QueryBuilder::new("INSERT INTO anime_list_entries (list_id, anime_id, position) ");
    query.push_values(ids.iter().enumerate(), |mut row, (index, anime_id)| {
        row.push_bind(list_id)
            .push_bind(anime_id)
            .push_bind(starting_position + index as i32);
    });
    query.build().execute(&mut **transaction).await?;
    Ok(())
}

pub async fn list_entry_ids(
    transaction: &mut Transaction<'_, MySql>,
    list_id: &str,
) -> Result<Vec<i32>, anyhow::Error> {
    let rows: Vec<(i32,)> = sqlx::query_as(
        "SELECT anime_id FROM anime_list_entries WHERE list_id = ? ORDER BY position, anime_id",
    )
    .bind(list_id)
    .fetch_all(&mut **transaction)
    .await?;
    Ok(rows.into_iter().map(|row| row.0).collect())
}

pub async fn set_custom_order(
    transaction: &mut Transaction<'_, MySql>,
    list_id: &str,
    ids: &[i32],
) -> Result<(), anyhow::Error> {
    for (position, anime_id) in ids.iter().enumerate() {
        sqlx::query(
            "UPDATE anime_list_entries SET position = ?, updated_at = CURRENT_TIMESTAMP WHERE list_id = ? AND anime_id = ?",
        )
        .bind(position as i32)
        .bind(list_id)
        .bind(anime_id)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

pub async fn set_default_order(
    transaction: &mut Transaction<'_, MySql>,
    owner_id: &str,
    ids: &[i32],
) -> Result<(), anyhow::Error> {
    for (position, anime_id) in ids.iter().enumerate() {
        sqlx::query(
            "UPDATE anime_users SET watch_priority = ?, updated_at = CURRENT_TIMESTAMP WHERE user_id = ? AND anime_id = ?",
        )
        .bind(position as i32 + 1)
        .bind(owner_id)
        .bind(anime_id)
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}

pub async fn search_hydrated_anime(
    db: &Pool<MySql>,
    query: &str,
) -> Result<Vec<ListAnime>, anyhow::Error> {
    let query = query
        .replace('!', "!!")
        .replace('%', "!%")
        .replace('_', "!_");
    let pattern = format!("%{query}%");
    sqlx::query_as::<_, ListAnime>(
        r#"
        SELECT id, english_title, romaji_title, status, picture, season, season_year
        FROM animes
        WHERE romaji_title IS NOT NULL
          AND status IS NOT NULL
          AND (romaji_title LIKE ? ESCAPE '!' OR english_title LIKE ? ESCAPE '!')
        ORDER BY romaji_title, id
        LIMIT 30
        "#,
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(db)
    .await
    .map_err(Into::into)
}
