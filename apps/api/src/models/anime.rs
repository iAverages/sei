use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::prelude::FromRow;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::anilist::api_types::{CoverImage, Title};

#[derive(Debug, Clone)]
pub struct InsertAnime {
    pub status: String,
    pub title: Title,
    pub id_mal: u32,
    pub cover_image: CoverImage,
    pub season: Option<String>,
    pub season_year: Option<u32>,
}

pub async fn insert_animes(db: &Pool<MySql>, animes: Vec<InsertAnime>) -> Result<(), sqlx::Error> {
    if animes.is_empty() {
        return Ok(());
    }
    let mut query_builder = QueryBuilder::new(
        r#"
        INSERT INTO animes (id, romaji_title,  status, picture, season, season_year, updated_at)
        "#,
    );

    query_builder.push_values(animes.iter(), |mut b, anime| {
        b.push_bind(anime.id_mal)
            .push_bind(anime.title.romaji.clone())
            .push_bind(anime.status.clone())
            .push_bind(anime.cover_image.large.clone())
            .push_bind(anime.season.clone())
            .push_bind(anime.season_year)
            .push_bind(chrono::Utc::now());
    });

    query_builder.push("ON DUPLICATE KEY UPDATE romaji_title = VALUES(romaji_title), status = VALUES(status), picture = VALUES(picture), season = VALUES(season), season_year = VALUES(season_year), updated_at = VALUES(updated_at)");

    let q = query_builder.build();

    q.execute(db).await.expect("Failed to insert anime");

    tracing::info!("Inserted {} animes", animes.len());

    Ok(())
}

#[derive(FromRow, Serialize)]
pub struct DBAnime {
    pub id: i32,
    pub english_title: Option<String>,
    pub romaji_title: String,
    pub status: String,
    pub picture: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub season: Option<String>,
    pub season_year: Option<i32>,
}

pub async fn get_released_animes_by_id(
    db: &Pool<MySql>,
    ids: Vec<i32>,
) -> Result<Vec<DBAnime>, anyhow::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            animes
        WHERE
            status = "FINISHED" AND
            id IN ( 
        "#,
    );

    for (i, id) in ids.iter().enumerate() {
        query_builder.push_bind(id);
        if i < ids.len() - 1 {
            query_builder.push(", ");
        }
    }

    query_builder.push(")");

    let query = query_builder.build_query_as::<DBAnime>();

    let animes = query.fetch_all(db).await?;

    Ok(animes)
}
