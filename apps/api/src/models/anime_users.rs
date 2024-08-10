use sqlx::{MySql, Pool, QueryBuilder};

use crate::importer::AnimeUserEntry;

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

    query_builder.push_values(flat_entries, |mut b, item| {
        let status_str: String = item.status.into();
        b.push_bind(item.user_id)
            .push_bind(item.anime_id)
            .push_bind(status_str)
            .push_bind(0);
    });

    query_builder.push("ON DUPLICATE KEY UPDATE status = VALUES(status), watch_priority = VALUES(watch_priority), updated_at = VALUES(updated_at)");

    let q = query_builder.build();

    q.execute(db).await.expect("Failed to insert anime");

    Ok(())
}
