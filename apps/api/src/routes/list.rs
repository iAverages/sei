use std::collections::HashSet;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{Duration as ChronoDuration, Utc};
use serde::Deserialize;
use serde_json::json;

use crate::{
    models::{
        list::{self, ListDetail, ListSummary, ListVisibility, DEFAULT_LIST_ID},
        user::{update_list_entries_mal, DBUser},
    },
    AppState,
};

type ApiResult<T> = Result<T, ListApiError>;

pub(crate) struct ListApiError {
    status: StatusCode,
    message: &'static str,
}

impl ListApiError {
    fn bad_request(message: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message,
        }
    }

    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "List not found",
        }
    }

    fn internal(context: &'static str, error: impl std::fmt::Display) -> Self {
        tracing::error!(error = %error, "{context}");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Internal server error",
        }
    }
}

impl IntoResponse for ListApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "message": self.message }))).into_response()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateListRequest {
    name: String,
    visibility: ListVisibility,
    anime_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct UpdateListRequest {
    name: String,
    visibility: ListVisibility,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeIdsRequest {
    anime_ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct OrderRequest {
    ids: Vec<i32>,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
}

pub async fn get_lists(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
) -> ApiResult<Json<Vec<ListSummary>>> {
    list::get_owned_lists(&state.db, &user.id)
        .await
        .map(Json)
        .map_err(|error| ListApiError::internal("failed to get lists", error))
}

pub async fn get_list(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Path(list_id): Path<String>,
) -> ApiResult<Json<ListDetail>> {
    if list_id == DEFAULT_LIST_ID {
        let five_minutes_ago = Utc::now().naive_utc() - ChronoDuration::minutes(5);
        if user.list_last_update < five_minutes_ago {
            let state = state.clone();
            let user = user.clone();
            tokio::spawn(async move {
                update_list_entries_mal(state, user).await;
            });
        }

        let anime = list::get_default_anime(&state.db, &user.id)
            .await
            .map_err(|error| ListApiError::internal("failed to get default list", error))?;
        let summary = list::get_default_summary(&state.db, &user.id)
            .await
            .map_err(|error| {
                ListApiError::internal("failed to get default list visibility", error)
            })?
            .ok_or_else(ListApiError::not_found)?;
        return Ok(Json(ListDetail {
            list: summary,
            anime,
        }));
    }

    let summary = require_owned_list(&state, &list_id, &user.id).await?;
    let anime = list::get_custom_anime(&state.db, &list_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get custom list entries", error))?;
    Ok(Json(ListDetail {
        list: summary,
        anime,
    }))
}

pub async fn get_public_list(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<ListDetail>> {
    if let Some((summary, owner_id)) = list::get_public_default_summary(&state.db, &slug)
        .await
        .map_err(|error| ListApiError::internal("failed to get public default list", error))?
    {
        let anime = list::get_default_anime(&state.db, &owner_id)
            .await
            .map_err(|error| {
                ListApiError::internal("failed to get public default list entries", error)
            })?;
        return Ok(Json(ListDetail {
            list: summary,
            anime,
        }));
    }

    let summary = list::get_public_summary(&state.db, &slug)
        .await
        .map_err(|error| ListApiError::internal("failed to get public list", error))?
        .ok_or_else(ListApiError::not_found)?;
    let anime = list::get_custom_anime(&state.db, &summary.id)
        .await
        .map_err(|error| ListApiError::internal("failed to get public list entries", error))?;
    Ok(Json(ListDetail {
        list: summary,
        anime,
    }))
}

pub async fn create_list(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Json(request): Json<CreateListRequest>,
) -> ApiResult<(StatusCode, Json<ListDetail>)> {
    let name = list::validate_name(request.name).map_err(ListApiError::bad_request)?;
    let ids = list::unique_anime_ids(request.anime_ids).map_err(ListApiError::bad_request)?;
    let anime = list::require_hydrated_anime(&state.db, &ids)
        .await
        .map_err(|error| ListApiError::internal("failed to validate list anime", error))?;
    if anime.len() != ids.len() {
        return Err(ListApiError::bad_request(
            "All anime IDs must refer to hydrated anime",
        ));
    }

    let id = cuid::cuid2();
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|error| ListApiError::internal("failed to start list creation", error))?;
    let slug = list::generate_slug(&mut transaction, &name, None)
        .await
        .map_err(|error| ListApiError::internal("failed to generate list slug", error))?;
    sqlx::query(
        "INSERT INTO anime_lists (id, owner_id, name, slug, visibility) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&user.id)
    .bind(&name)
    .bind(&slug)
    .bind(request.visibility.as_str())
    .execute(&mut *transaction)
    .await
    .map_err(|error| ListApiError::internal("failed to create list", error))?;
    list::insert_entries(&mut transaction, &id, &ids, 0)
        .await
        .map_err(|error| ListApiError::internal("failed to create list entries", error))?;
    transaction
        .commit()
        .await
        .map_err(|error| ListApiError::internal("failed to commit list creation", error))?;

    Ok((
        StatusCode::CREATED,
        Json(ListDetail {
            list: ListSummary {
                id,
                name,
                slug,
                visibility: request.visibility,
                is_default: false,
            },
            anime: list::order_anime(anime, &ids),
        }),
    ))
}

pub async fn update_list(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Path(list_id): Path<String>,
    Json(request): Json<UpdateListRequest>,
) -> ApiResult<Json<ListDetail>> {
    if list_id == DEFAULT_LIST_ID {
        sqlx::query(
            "UPDATE users SET default_list_visibility = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(request.visibility.as_str())
        .bind(&user.id)
        .execute(&state.db)
        .await
        .map_err(|error| ListApiError::internal("failed to update default list", error))?;
        let anime = list::get_default_anime(&state.db, &user.id)
            .await
            .map_err(|error| ListApiError::internal("failed to get updated default list", error))?;
        return Ok(Json(ListDetail {
            list: ListSummary::default_list(request.visibility, user.name),
            anime,
        }));
    }

    let name = list::validate_name(request.name).map_err(ListApiError::bad_request)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|error| ListApiError::internal("failed to start list update", error))?;
    owned_list_in_transaction(&mut transaction, &list_id, &user.id).await?;
    let slug = list::generate_slug(&mut transaction, &name, Some(&list_id))
        .await
        .map_err(|error| ListApiError::internal("failed to generate list slug", error))?;
    sqlx::query(
        "UPDATE anime_lists SET name = ?, slug = ?, visibility = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND owner_id = ?",
    )
    .bind(&name)
    .bind(&slug)
    .bind(request.visibility.as_str())
    .bind(&list_id)
    .bind(&user.id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ListApiError::internal("failed to update list", error))?;
    transaction
        .commit()
        .await
        .map_err(|error| ListApiError::internal("failed to commit list update", error))?;
    let anime = list::get_custom_anime(&state.db, &list_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get updated list", error))?;
    Ok(Json(ListDetail {
        list: ListSummary {
            id: list_id,
            name,
            slug,
            visibility: request.visibility,
            is_default: false,
        },
        anime,
    }))
}

pub async fn add_entries(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Path(list_id): Path<String>,
    Json(request): Json<AnimeIdsRequest>,
) -> ApiResult<Json<ListDetail>> {
    reject_default(&list_id)?;
    let ids = list::unique_anime_ids(request.anime_ids).map_err(ListApiError::bad_request)?;
    let anime = list::require_hydrated_anime(&state.db, &ids)
        .await
        .map_err(|error| ListApiError::internal("failed to validate list entries", error))?;
    if anime.len() != ids.len() {
        return Err(ListApiError::bad_request(
            "All anime IDs must refer to hydrated anime",
        ));
    }

    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|error| ListApiError::internal("failed to start entry addition", error))?;
    let summary = owned_list_in_transaction(&mut transaction, &list_id, &user.id).await?;
    let current = list::list_entry_ids(&mut transaction, &list_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get existing entries", error))?;
    let existing: HashSet<_> = current.iter().copied().collect();
    let new_ids: Vec<_> = ids
        .into_iter()
        .filter(|id| !existing.contains(id))
        .collect();
    list::insert_entries(&mut transaction, &list_id, &new_ids, current.len() as i32)
        .await
        .map_err(|error| ListApiError::internal("failed to add list entries", error))?;
    transaction
        .commit()
        .await
        .map_err(|error| ListApiError::internal("failed to commit entry addition", error))?;

    let anime = list::get_custom_anime(&state.db, &list_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get updated list", error))?;
    Ok(Json(ListDetail {
        list: summary,
        anime,
    }))
}

pub async fn delete_entry(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Path((list_id, anime_id)): Path<(String, i32)>,
) -> ApiResult<StatusCode> {
    reject_default(&list_id)?;
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|error| ListApiError::internal("failed to start entry deletion", error))?;
    owned_list_in_transaction(&mut transaction, &list_id, &user.id).await?;
    sqlx::query("DELETE FROM anime_list_entries WHERE list_id = ? AND anime_id = ?")
        .bind(&list_id)
        .bind(anime_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| ListApiError::internal("failed to delete list entry", error))?;
    let ids = list::list_entry_ids(&mut transaction, &list_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get remaining entries", error))?;
    list::set_custom_order(&mut transaction, &list_id, &ids)
        .await
        .map_err(|error| ListApiError::internal("failed to normalize list positions", error))?;
    transaction
        .commit()
        .await
        .map_err(|error| ListApiError::internal("failed to commit entry deletion", error))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn update_order(
    State(state): State<AppState>,
    Extension(user): Extension<DBUser>,
    Path(list_id): Path<String>,
    Json(request): Json<OrderRequest>,
) -> ApiResult<StatusCode> {
    let mut transaction = state
        .db
        .begin()
        .await
        .map_err(|error| ListApiError::internal("failed to start list reorder", error))?;

    if list_id == DEFAULT_LIST_ID {
        let current = list::get_default_anime_for_update(&mut transaction, &user.id)
            .await
            .map_err(|error| ListApiError::internal("failed to get default list order", error))?;
        let current_ids: Vec<_> = current.into_iter().map(|anime| anime.id).collect();
        list::validate_exact_order(&request.ids, &current_ids)
            .map_err(ListApiError::bad_request)?;
        list::set_default_order(&mut transaction, &user.id, &request.ids)
            .await
            .map_err(|error| ListApiError::internal("failed to reorder default list", error))?;
    } else {
        owned_list_in_transaction(&mut transaction, &list_id, &user.id).await?;
        let current = list::list_entry_ids(&mut transaction, &list_id)
            .await
            .map_err(|error| ListApiError::internal("failed to get list order", error))?;
        list::validate_exact_order(&request.ids, &current).map_err(ListApiError::bad_request)?;
        list::set_custom_order(&mut transaction, &list_id, &request.ids)
            .await
            .map_err(|error| ListApiError::internal("failed to reorder custom list", error))?;
    }

    transaction
        .commit()
        .await
        .map_err(|error| ListApiError::internal("failed to commit list reorder", error))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_anime(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> ApiResult<Json<Vec<list::ListAnime>>> {
    let query = query.q.trim();
    if query.chars().count() < 2 {
        return Ok(Json(Vec::new()));
    }
    if query.chars().count() > 100 {
        return Err(ListApiError::bad_request(
            "Search queries must be at most 100 characters",
        ));
    }
    list::search_hydrated_anime(&state.db, query)
        .await
        .map(Json)
        .map_err(|error| ListApiError::internal("failed to search anime", error))
}

fn reject_default(list_id: &str) -> ApiResult<()> {
    if list_id == DEFAULT_LIST_ID {
        Err(ListApiError::bad_request(
            "The default list cannot be modified this way",
        ))
    } else {
        Ok(())
    }
}

async fn require_owned_list(
    state: &AppState,
    list_id: &str,
    owner_id: &str,
) -> ApiResult<ListSummary> {
    list::get_owned_summary(&state.db, list_id, owner_id)
        .await
        .map_err(|error| ListApiError::internal("failed to get owned list", error))?
        .ok_or_else(ListApiError::not_found)
}

async fn owned_list_in_transaction(
    transaction: &mut sqlx::Transaction<'_, sqlx::MySql>,
    list_id: &str,
    owner_id: &str,
) -> ApiResult<ListSummary> {
    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT id, name, slug, visibility FROM anime_lists WHERE id = ? AND owner_id = ? FOR UPDATE",
    )
    .bind(list_id)
    .bind(owner_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(|error| ListApiError::internal("failed to lock owned list", error))?;
    let (id, name, slug, visibility) = row.ok_or_else(ListApiError::not_found)?;
    let visibility = match visibility.to_ascii_uppercase().as_str() {
        "PRIVATE" => ListVisibility::Private,
        "UNLISTED" => ListVisibility::Unlisted,
        "PUBLIC" => ListVisibility::Public,
        _ => {
            return Err(ListApiError::internal(
                "list has invalid visibility",
                visibility,
            ))
        }
    };
    Ok(ListSummary {
        id,
        name,
        slug,
        visibility,
        is_default: false,
    })
}
