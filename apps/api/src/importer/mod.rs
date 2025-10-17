use std::sync::Arc;
use std::time::Duration;
use std::vec;

use chrono::Utc;
use cuid::cuid2;
use reqwest::Client;
use sea_query::Condition;
use sea_query::Expr;
use sea_query::ExprTrait;
use sea_query::OnConflict;
use sqlx::{MySql, Pool};
use thiserror::Error;
use tokio::sync::broadcast;
use tokio::time;

use sea_query::{MysqlQueryBuilder, Query};
use sea_query_sqlx::SqlxBinder;

use crate::anilist::api_types::AnilistApiAnime;
use crate::anilist::gql_query::MAX_ANILIST_PER_QUERY;
use crate::anilist::Anilist;
use crate::consts::MYSQL_PARAM_BIND_LIMIT;
use crate::database::anime_job_queue::AnimeJobQueue;
use crate::database::anime_job_queue::AnimeJobQueueStatus;
use crate::database::anime_job_queue::DBAnimeJobQueue;
use crate::database::models::Animes;
use crate::models::anime::FullAnime;
use crate::ImportEvent;

pub struct ImporterStats {
    in_queue: i32,
}

#[derive(Error, Debug)]
pub enum ImporterError {
    #[error("failed to add items to queue")]
    AddToQueue,
    #[error("failed to add update items status")]
    UpdateStatus,
    #[error("failed to store anime data")]
    StoreData,
}

#[derive(Clone)]
pub struct Importer {
    inner: Arc<ImporterInner>,
}

impl Importer {
    pub fn new(_: Client, db: Pool<MySql>, tx: broadcast::Sender<ImportEvent>) -> Self {
        Importer {
            inner: Arc::new(ImporterInner::new(db, tx)),
        }
    }

    pub fn start(&mut self) {
        tracing::info!("starting job process queue");
        let inner = self.inner.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(2000));

            loop {
                tracing::trace!("job process queue tick");
                interval.tick().await;
                inner.process().await;
            }
        });
    }

    pub fn stats(&self) -> ImporterStats {
        todo!()
    }

    // TODO: should we requeue if the status is error but is within last 6 hours?
    async fn valid_to_queue(&self, items: &[i32]) -> Vec<(i32, bool)> {
        let mut query = Query::select();
        query
            .from(AnimeJobQueue::Table)
            .columns([
                AnimeJobQueue::Id,
                AnimeJobQueue::AnimeId,
                AnimeJobQueue::Status,
                AnimeJobQueue::TriggeredById,
            ])
            .cond_where(
                Condition::all()
                    .add(
                        Condition::any()
                            .add(Expr::col(AnimeJobQueue::Status).is_in([
                                AnimeJobQueueStatus::Pending,
                                AnimeJobQueueStatus::InProgress,
                            ]))
                            .add(
                                Expr::col(AnimeJobQueue::CompleteAt)
                                    .gte(Expr::cust("NOW() - INTERVAL 6 HOUR")),
                            ),
                    )
                    .add(Expr::col(AnimeJobQueue::AnimeId).is_in(items.to_vec())),
            )
            .group_by_col(AnimeJobQueue::AnimeId);

        let (sql, values) = query.build_sqlx(MysqlQueryBuilder);

        let result: Result<Vec<DBAnimeJobQueue>, _> = sqlx::query_as_with(&sql, values)
            .fetch_all(&self.inner.db)
            .await;

        if let Err(error) = result {
            tracing::error!(
                error = error.to_string(),
                "an error occured while fetching anime job queue"
            );
            return items.iter().map(|id| (*id, false)).collect();
        }

        let recently_updated = result.unwrap();

        let mut should_queue = Vec::new();
        for id in items {
            let updated_recently = recently_updated.iter().find(|item| item.anime_id == *id);
            if updated_recently.is_none() {
                tracing::trace!(id = id, "going to requeue item");
            }
            should_queue.push((*id, updated_recently.is_none()));
        }

        should_queue
    }

    async fn add_empty_anime(&self, ids: &[i32]) -> Result<(), anyhow::Error> {
        let (sql, values) = Query::insert()
            .into_table(Animes::Table)
            .columns([Animes::Id])
            .values_from_panic(ids.iter().map(|id| [(*id).into()]))
            .on_conflict(
                OnConflict::column(Animes::Id)
                    .do_nothing_on([Animes::Id])
                    .to_owned(),
            )
            .build_sqlx(MysqlQueryBuilder);
        println!("{:?}", sql);

        sqlx::query_with(&sql, values)
            .execute(&self.inner.db)
            .await
            .inspect_err(|error| {
                tracing::error!(error = error.to_string(), "failed to add anime to database");
            })
            .map_err(|_| ImporterError::AddToQueue)?;

        Ok(())
    }

    pub async fn add_items(
        &self,
        items: Vec<i32>,
        triggered_by_id: Option<&str>,
    ) -> Result<(), ImporterError> {
        let ids_to_queue = self
            .valid_to_queue(&items)
            .await
            .into_iter()
            .filter(|(_, should_queue)| *should_queue)
            .map(|(id, _)| id)
            .collect::<Vec<i32>>();
        tracing::info!(
            amount = ids_to_queue.len(),
            skipped = items.len() - ids_to_queue.len(),
            "adding items to queue"
        );

        if ids_to_queue.is_empty() {
            tracing::info!("no provided items should be added to the queue");
            return Ok(());
        }

        // TODO: handle error
        let res = self.add_empty_anime(&ids_to_queue).await;
        if let Err(error) = res {
            tracing::info!("failed to add empty anime to database: {}", error);
        }

        let mut query = Query::insert();
        query.into_table(AnimeJobQueue::Table);

        let mut columns = vec![
            AnimeJobQueue::Id,
            AnimeJobQueue::AnimeId,
            AnimeJobQueue::Status,
        ];
        if triggered_by_id.is_some() {
            columns.push(AnimeJobQueue::TriggeredById);
        }
        query.columns(columns);

        query.values_from_panic(ids_to_queue.into_iter().map(|anime_id| {
            let id = cuid2();
            let mut values = vec![
                id.into(),
                anime_id.into(),
                AnimeJobQueueStatus::Pending.into(),
            ];
            if let Some(triggered_by_id_val) = triggered_by_id {
                values.push(triggered_by_id_val.into());
            }
            values
        }));

        let (sql, values) = query.build_sqlx(MysqlQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.inner.db)
            .await
            .inspect_err(|error| {
                tracing::error!(error = error.to_string(), "failed to add items to queue");
            })
            .map_err(|_| ImporterError::AddToQueue)?;

        Ok(())
    }
}

struct ImporterInner {
    db: Pool<MySql>,
    anilist: Arc<Anilist>,
    tx: broadcast::Sender<ImportEvent>,
}

impl ImporterInner {
    pub fn new(db: Pool<MySql>, tx: broadcast::Sender<ImportEvent>) -> Self {
        let anilist = Arc::new(Anilist::new());
        ImporterInner { db, anilist, tx }
    }

    async fn update_item_status(
        &self,
        items: &[String],
        status: AnimeJobQueueStatus,
    ) -> Result<(), ImporterError> {
        // -1 due to the param in values (status)
        for chunk in items.chunks(MYSQL_PARAM_BIND_LIMIT - 1) {
            let mut query = Query::update();

            let mut values = vec![(AnimeJobQueue::Status, status.clone().into())];
            // if job is complete or failed, set completed at to the current time
            if status == AnimeJobQueueStatus::Complete || status == AnimeJobQueueStatus::Failed {
                values.push((AnimeJobQueue::CompleteAt, Utc::now().into()));
            }

            query
                .table(AnimeJobQueue::Table)
                .values(values)
                .and_where(Expr::col(AnimeJobQueue::Id).is_in(chunk.to_vec()));

            let (sql, values) = query.build_sqlx(MysqlQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.db)
                .await
                .map_err(|_| ImporterError::UpdateStatus)?;
        }

        Ok(())
    }

    // TODO: handle case where job gets stuck in an "in progress" state
    // should probably just check at startup? mark as errored for now since
    // only one instance of the backend will be running
    async fn fetch_next(&self, amount: u32) -> Vec<DBAnimeJobQueue> {
        let result = sqlx::query_file!(
            "database/queries/update_queue_items_status.sql",
            AnimeJobQueueStatus::InProgress,
            AnimeJobQueueStatus::Pending,
            amount
        )
        .execute(&self.db)
        .await;

        if let Err(error) = result {
            tracing::error!(
                error = error.to_string(),
                "failed to mark jobs as in progress"
            );
            return vec![];
        }

        // TODO: move these queries into a model struct
        let result = sqlx::query_file_as!(
            DBAnimeJobQueue,
            "database/queries/fetch_next_queue_items.sql",
            amount
        )
        .fetch_all(&self.db)
        .await;

        if let Ok(data) = result {
            return data;
        }

        let error = result.err().unwrap();
        tracing::error!(
            error = error.to_string(),
            "failed to fetch next queue items"
        );
        // if we failed to fetch the next items, ensure we mark these as failed
        let result = sqlx::query_file!(
            "database/queries/update_queue_items_status.sql",
            AnimeJobQueueStatus::Failed,
            AnimeJobQueueStatus::InProgress,
            amount
        )
        .execute(&self.db)
        .await;

        if let Err(error) = result {
            tracing::error!(error = error.to_string(), "failed to mark jobs as failed");
        }

        vec![]
    }

    async fn set_anime_data(
        &self,
        anime_data: Vec<(AnilistApiAnime, Option<String>)>,
    ) -> Result<(), ImporterError> {
        tracing::info!(total = anime_data.len(), "settings data for animes");
        let columns = [
            Animes::Id,
            Animes::Status,
            Animes::RomajiTitle,
            Animes::Picture,
            Animes::Season,
            Animes::SeasonYear,
        ];

        for chunk in anime_data.chunks(MYSQL_PARAM_BIND_LIMIT / columns.len()) {
            let mut query = Query::insert();
            query.into_table(Animes::Table).columns(columns.clone());

            for (anime, _) in chunk {
                query.values_panic([
                    anime.id_mal.into(),
                    anime.status.clone().into(),
                    anime.title.romaji.clone().into(),
                    anime.cover_image.large.clone().into(),
                    anime.season.clone().into(),
                    anime.season_year.into(),
                ]);
            }

            query.on_conflict(
                OnConflict::column(Animes::Id)
                    .update_column(Animes::RomajiTitle)
                    .update_column(Animes::Status)
                    .update_column(Animes::Picture)
                    .update_column(Animes::UpdatedAt)
                    .update_column(Animes::Season)
                    .update_column(Animes::SeasonYear)
                    .to_owned(),
            );

            let (sql, values) = query.build_sqlx(MysqlQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.db)
                .await
                .map_err(|_| ImporterError::StoreData)?;

            chunk.iter().for_each(|(anime, triggered_by_id)| {
                let Some(triggered_by_id) = triggered_by_id else {
                    return;
                };
                let res = self.tx.send(ImportEvent {
                    user_id: triggered_by_id.clone(),
                    anime: anime.clone().into(),
                    list_id: "default".to_string(),
                });

                tracing::info!(anime = anime.id_mal, "send event over sse");
                if let Err(error) = res {
                    tracing::error!("error while sending sse event: {}", error);
                }
            });
        }

        Ok(())
    }

    async fn process(&self) {
        let items = self.fetch_next(MAX_ANILIST_PER_QUERY as u32).await;
        if items.is_empty() {
            tracing::trace!("no jobs to process");
            return;
        }
        let anime_ids = items.iter().map(|item| item.anime_id).collect::<Vec<i32>>();
        let job_ids = items
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<String>>();

        tracing::info!(jobs_amount = items.len(), "found jobs to process");

        let response = self.anilist.clone().fetch_animes(&anime_ids, true).await;

        if let Err(error) = response {
            tracing::error!(error = error.to_string(), "failed to process queue");
            let _ = self
                .update_item_status(&job_ids, AnimeJobQueueStatus::Failed)
                .await;
            return;
        }

        let anime_data = response.unwrap();

        let anime_data_with_trigged_by = anime_data
            .into_iter()
            .map(|anime| {
                let event = items
                    .iter()
                    .find(|event| event.anime_id == anime.id_mal as i32);
                let triggered_by_id = match event {
                    Some(event) => event.triggered_by_id.clone(),
                    None => None,
                };
                (anime, triggered_by_id)
            })
            .collect::<Vec<_>>();
        let _ = self.set_anime_data(anime_data_with_trigged_by).await;
        let _ = self
            .update_item_status(&job_ids, AnimeJobQueueStatus::Complete)
            .await;
    }
}
