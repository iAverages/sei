use std::sync::Arc;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, QueryBuilder};

use crate::consts::MYSQL_PARAM_BIND_LIMIT;

pub struct DBAnimeUser {
    pub id: Option<i32>,
    // pub anime_id: Option<i32>,
    // pub status: AnimeWatchStatus,
    // pub watch_priority: i32,
    // pub created_at: NaiveDateTime,
    // pub updated_at: NaiveDateTime,
}

// TODO: readd
pub async fn link_user_to_anime(
    db: &Pool<MySql>,
    // items: Vec<(u32, Vec<AnimeUserEntry>)>,
) -> Result<(), anyhow::Error> {
    todo!();
    // if items.is_empty() {
    //     return Ok(());
    // }
    //
    // let mut query_builder = QueryBuilder::new(
    //     r#"
    //     INSERT INTO anime_users (user_id, anime_id, status, watch_priority)
    //     "#,
    // );
    //
    // let flat_entries: Vec<AnimeUserEntry> =
    //     items.into_iter().flat_map(|(_, strings)| strings).collect();
    //
    // if flat_entries.is_empty() {
    //     return Ok(());
    // }
    //
    // query_builder.push_values(flat_entries, |mut b, item| {
    //     let status_str: String = item.status.into();
    //     b.push_bind(item.user_id)
    //         .push_bind(item.anime_id)
    //         .push_bind(status_str)
    //         .push_bind(0);
    // });
    //
    // query_builder
    //     .push("ON DUPLICATE KEY UPDATE status = VALUES(status), updated_at = VALUES(updated_at)");
    //
    // let q = query_builder.build();
    //
    // q.execute(db).await.expect("Failed to insert anime_user");
    //
    // Ok(())
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

pub async fn get_animes_for_user(
    db: &Pool<MySql>,
    user_id: &str,
) -> Result<Vec<i32>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DBAnimeUser,
       "
WITH RECURSIVE RelatedItems AS (
    -- Anchor member: Get all items directly linked to the user
    SELECT
        ui.anime_id AS id
    FROM
        `anime_users` ui
    WHERE
        ui.user_id = ?

    UNION DISTINCT

    -- Recursive member: Find items related to the previously found items
    SELECT
        CASE
            WHEN ir.anime_id = ri.id THEN ir.relation_id
            ELSE ir.anime_id
        END AS id
    FROM
        `anime_relations` ir
    INNER JOIN
        RelatedItems ri ON ir.anime_id = ri.id OR ir.relation_id = ri.id
    WHERE
        (ir.anime_id = ri.id AND ir.relation_id IS NOT NULL) OR (ir.relation_id = ri.id AND ir.anime_id IS NOT NULL)
)
SELECT DISTINCT
    id
FROM
    RelatedItems;
",
        user_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows.iter().filter_map(|row| row.id).collect())
}

// TODO: fix naming of everything
#[derive(Serialize)]
pub struct DBAnimeUser2 {
    pub user_id: String,
    pub anime_id: i32,
    pub status: String,
    pub watch_priority: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
pub async fn get_user_list_entries(
    db: &Pool<MySql>,
    user_id: &str,
) -> Result<Vec<DBAnimeUser2>, anyhow::Error> {
    let rows = sqlx::query_as!(
        DBAnimeUser2,
        "SELECT * from anime_users WHERE user_id = ? AND status in (\"plan_to_watch\", \"watching\")",
        user_id
    )
    .fetch_all(db)
    .await?;

    Ok(rows)
}
