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
