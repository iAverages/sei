use sqlx::{MySql, Pool, QueryBuilder};

pub async fn create_anime_relation(
    db: &Pool<MySql>,
    items: Vec<(u32, u32, String)>,
) -> Result<(), anyhow::Error> {
    if items.is_empty() {
        return Ok(());
    }
    let mut query_builder = QueryBuilder::new(
        r#"
        INSERT INTO anime_relations (anime_id, relation_id, relation)
        "#,
    );

    query_builder.push_values(items.iter(), |mut b, item| {
        b.push_bind(item.0)
            .push_bind(item.1)
            .push_bind(item.2.clone());
    });

    query_builder.push("ON DUPLICATE KEY UPDATE  relation = VALUES(relation)");

    let q = query_builder.build();

    q.execute(db).await.expect("Failed to insert anime");

    tracing::info!("Inserted {} anime relations", items.len());

    Ok(())
}
