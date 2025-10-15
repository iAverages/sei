use sea_query::{Expr, ExprTrait, MysqlQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::database::models::Animes;

#[derive(FromRow, Serialize, Clone, Debug, Deserialize)]
pub struct FullAnime {
    pub id: i32,
    pub romaji_title: String,
    pub status: String, // TODO: make to correct enum type
    pub picture: Option<String>,
    pub season: Option<String>,
    pub season_year: Option<i32>,
}

pub async fn get_released_animes_by_id(
    db: &Pool<MySql>,
    ids: &[i32],
) -> Result<Vec<FullAnime>, anyhow::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let mut query = Query::select();
    query
        .from(Animes::Table)
        .columns([
            Animes::Id,
            Animes::RomajiTitle,
            Animes::Status,
            Animes::Picture,
            Animes::Season,
            Animes::SeasonYear,
        ])
        .and_where(Expr::col(Animes::Id).is_in(ids.to_vec()))
        .and_where(Expr::col(Animes::RomajiTitle).is_not_null())
        .and_where(Expr::col(Animes::Status).is_not_null());

    let (sql, values) = query.build_sqlx(MysqlQueryBuilder);

    let animes: Vec<FullAnime> = sqlx::query_as_with(&sql, values)
        .fetch_all(db)
        .await
        .unwrap();

    Ok(animes)
}

#[derive(Serialize, FromRow)]
pub struct DBAnimeRelation {
    anime_id: i32,
    relation_id: i32,
}

pub async fn get_anime_relations(
    db: &Pool<MySql>,
    ids: &[i32],
) -> Result<Vec<DBAnimeRelation>, anyhow::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let mut query_builder: QueryBuilder<MySql> = QueryBuilder::new(
        r#"
        SELECT
            *
        FROM
            anime_relations
        WHERE
            anime_id IN ( 
        "#,
    );

    for (i, id) in ids.iter().enumerate() {
        query_builder.push_bind(id);
        if i < ids.len() - 1 {
            query_builder.push(", ");
        }
    }

    query_builder.push(")");

    let query = query_builder.build_query_as::<DBAnimeRelation>();

    let animes = query.fetch_all(db).await?;

    Ok(animes)
}
