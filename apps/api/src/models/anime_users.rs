use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::consts::MYSQL_PARAM_BIND_LIMIT;
use crate::importer::{AnimeUserEntry, AnimeWatchStatus};

pub struct DBAnimeUser {
    pub user_id: String,
    pub anime_id: i32,
    pub status: AnimeWatchStatus,
    pub watch_priority: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub async fn link_user_to_anime(
    db: &Pool<MySql>,
    items: Vec<(u32, Vec<AnimeUserEntry>)>,
) -> Result<(), anyhow::Error> {
    if items.is_empty() {
        return Ok(());
    }

    let mut query_builder = QueryBuilder::new(
        r#"
        INSERT INTO anime_users (user_id, anime_id, status, watch_priority)
        "#,
    );

    let flat_entries: Vec<AnimeUserEntry> =
        items.into_iter().flat_map(|(_, strings)| strings).collect();

    if flat_entries.is_empty() {
        return Ok(());
    }

    query_builder.push_values(flat_entries, |mut b, item| {
        let status_str: String = item.status.into();
        b.push_bind(item.user_id)
            .push_bind(item.anime_id)
            .push_bind(status_str)
            .push_bind(0);
    });

    query_builder
        .push("ON DUPLICATE KEY UPDATE status = VALUES(status), updated_at = VALUES(updated_at)");

    let q = query_builder.build();

    q.execute(db).await.expect("Failed to insert anime_user");

    Ok(())
}

#[derive(Deserialize)]
pub struct WatchPriorityUpdate {
    pub ids: Vec<i32>,
}

pub async fn update_watch_priority(db: &Pool<MySql>, user_id: String, data: WatchPriorityUpdate) {
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        r#"
        INSERT INTO anime_users (anime_id, user_id, watch_priority) 
        "#,
    );

    let mut index = 1;
    let user_id = Arc::new(user_id);

    let groups = data.ids.chunks(MYSQL_PARAM_BIND_LIMIT / 3);

    for group in groups {
        query_builder.push_values(group.iter(), |mut b, id| {
            b.push_bind(id).push_bind(user_id.as_str()).push_bind(index);
            index += 1;
        });

        let q = query_builder
            .push(
                r#"
                ON DUPLICATE KEY UPDATE watch_priority = VALUES(watch_priority)
                "#,
            )
            .build();

        q.execute(db).await.expect("Failed to update anime_user");
    }
}

pub async fn get_user_entrys(
    db: &Pool<MySql>,
    user_id: String,
) -> Result<Vec<DBAnimeUser>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DBAnimeUser,
        "SELECT * from anime_users WHERE user_id = ? AND status in (\"plan_to_watch\", \"watching\")",
        user_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}
