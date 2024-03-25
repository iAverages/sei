use std::time::Duration;

use sqlx::{MySql, Pool, QueryBuilder};
use tokio::time::timeout;

use crate::{
    anime::{
        self, get_error_ids, get_local_anime_datas, AniListAnimeItem, AnilistResponse,
        AnimeTableRow, CoverImage, RelationsEdge, Title, MAX_ANILIST_PER_QUERY,
    },
    AppState, ImportQueueItem,
};

#[derive(Debug, Clone)]
pub struct InsertAnime {
    pub status: String,
    pub title: Title,
    pub id_mal: i32,
    pub cover_image: CoverImage,
    pub season: Option<String>,
    pub season_year: Option<i32>,
}

async fn insert_animes(db: &Pool<MySql>, animes: Vec<InsertAnime>) -> Result<(), sqlx::Error> {
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

async fn queue_related_animes(state: AppState, animes: Vec<AniListAnimeItem>) {
    for anime in animes {
        tracing::info!("Queueing related animes for {}", anime.id_mal.unwrap());

        let valid_relations = anime
            .relations
            .as_ref()
            .map(|r| {
                r.edges
                    .iter()
                    .filter(|r| r.relation_type == "PREQUEL" || r.relation_type == "SEQUEL")
                    .filter(|r| r.node.id_mal.is_some())
                    .collect::<Vec<&RelationsEdge>>()
            })
            .unwrap_or(vec![]);

        let local_relation_animes = get_local_anime_datas(
            state.db.clone(),
            valid_relations
                .iter()
                .map(|r| r.node.id_mal.unwrap())
                .collect::<Vec<i32>>(),
        )
        .await
        .unwrap();

        let local_relations = local_relation_animes
            .iter()
            .filter(|a| {
                let now = chrono::Utc::now().naive_utc();
                let updated_at = a.updated_at;
                let diff = now - updated_at;

                if diff.num_hours() >= 1 {
                    return false;
                }
                true
            })
            .collect::<Vec<&AnimeTableRow>>();

        tracing::info!(
            "Found {} related animes ({}) that need (re)importing",
            local_relations.len(),
            anime.id_mal.unwrap()
        );

        if valid_relations.is_empty() {
            let local_anime =
                anime::get_local_anime_data(state.db.clone(), anime.id_mal.unwrap()).await;
            if local_anime.is_err() {
                tracing::error!(
                    "Anime {} not found in local database",
                    anime.id_mal.unwrap()
                );
            }
            let a = local_anime.unwrap();
            let now = chrono::Utc::now().naive_utc();
            let updated_at = a.updated_at;
            let diff = now - updated_at;

            if diff.num_hours() >= 1 {
                tracing::info!("Building series relations");
                build_series_group(state.clone(), anime).await;
                return;
            }
            tracing::info!("No related animes to queue for {}", anime.id_mal.unwrap());
            continue;
        }

        for relation in valid_relations {
            let local_anime = local_relations
                .iter()
                .find(|a| a.id == relation.node.id_mal.unwrap());

            if local_anime.is_none() {
                state.import_queue.push(ImportQueueItem::Relationship {
                    anime_id: relation.node.id_mal.unwrap(),
                    related_anime_id: anime.id_mal.unwrap(),
                    related_anime_type: relation.relation_type.clone(),
                    times_in_queue: 0,
                });
            }
        }
    }
    // let anime_relations = animes
    //     .iter()
    //     .flat_map(|a| {
    //         a.relations
    //             .as_ref()
    //             .map(|r| {
    //                 let related = r
    //                     .edges
    //                     .iter()
    //                     .filter(|r| r.relation_type == "PREQUEL" || r.relation_type == "SEQUEL")
    //                     .filter(|r| r.node.id_mal.is_some())
    //                     .inspect(|r| {
    //                         if a.id_mal.unwrap() == 31964 {
    //                             tracing::info!(
    //                                 "Found related anime: {} {}",
    //                                 r.node.id_mal.unwrap(),
    //                                 r.relation_type
    //                             );
    //                         }
    //                     })
    //                     .collect::<Vec<&RelationsEdge>>();

    //                 related
    //                     .iter()
    //                     // Fuck you - https://avrg.dev/sei-gae-pout
    //                     .filter(|r| r.node.id_mal.unwrap() != 125359)
    //                     .map(|r| AnimeRelationInsertItem {
    //                         anime_id: a.id_mal.unwrap(),
    //                         related_anime_id: r.node.id_mal.unwrap(),
    //                         relation: r.relation_type.to_owned(),
    //                     })
    //                     .collect::<Vec<AnimeRelationInsertItem>>()
    //             })
    //             .unwrap()
    //     })
    //     .collect::<Vec<AnimeRelationInsertItem>>();

    // tracing::info!("Found {} related animes", anime_relations.len());
    // let local_anime_data = get_local_anime_datas(
    //     state.db,
    //     anime_relations
    //         .iter()
    //         .map(|r| r.related_anime_id)
    //         .collect::<Vec<i32>>(),
    // )
    // .await;

    // let local_anime_ids = local_anime_data
    //     .unwrap()
    //     .iter()
    //     // filter out all animes where the updated_at is less than 1 hour
    //     .filter(|a| {
    //         let now = chrono::Utc::now().naive_utc();
    //         let updated_at = a.updated_at;
    //         let diff = now - updated_at;

    //         if diff.num_hours() >= 1 {
    //             tracing::info!("Anime {} wil get reimported", a.id);
    //             return true;
    //         }
    //         false
    //     })
    //     .map(|a| a.id)
    //     .collect::<Vec<i32>>();

    // let anime_relations = anime_relations
    //     .iter()
    //     // .filter(|r| !local_anime_ids.contains(&r.related_anime_id))
    //     .collect::<Vec<&AnimeRelationInsertItem>>();

    // if anime_relations.is_empty() {
    //     tracing::info!("No related animes to queue");
    //     return;
    // }

    // tracing::info!("Queueing {} related animes", anime_relations.len());
    // anime_relations.iter().for_each(|relation| {
    //     if local_anime_ids.contains(&relation.related_anime_id) {
    //         tracing::info!(
    //             "Anime {} is already in the database, skipping...",
    //             relation.related_anime_id
    //         );
    //         return;
    //     }
    //     state.import_queue.push(ImportQueueItem::Relationship {
    //         times_in_queue: 0,
    //         anime_id: relation.anime_id,
    //         related_anime_id: relation.related_anime_id,
    //         related_anime_type: relation.relation.clone(),
    //     });
    // });
}

async fn add_anime_to_series(
    state: AppState,
    series_id: i32,
    anime_id: i32,
    series_order: i32,
) -> bool {
    tracing::info!("Adding {} to series {}", series_id, anime_id,);
    let relation_isert_result = sqlx::query!(
        r#"
                INSERT INTO anime_series (series_id, anime_id, series_order)
                VALUES (?, ?, ?)
                ON DUPLICATE KEY UPDATE series_id = ?, anime_id = ?, series_order = ?
                "#,
        series_id,
        anime_id,
        series_order,
        series_id,
        anime_id,
        series_order,
    )
    .execute(&state.db)
    .await;

    if let Err(e) = relation_isert_result {
        tracing::error!("Failed to insert relation link: {}", e);
        return false;
    }

    true
}

async fn add_anime_relation(
    state: AppState,
    anime_id: i32,
    related_anime_id: i32,
    relation: String,
) -> bool {
    // tracing::info!(
    //     "Adding relation between anime {} and {}",
    //     anime_id,
    //     related_anime_id
    // );
    let relation_isert_result = sqlx::query!(
        r#"
        INSERT INTO anime_relations (anime_id, relation_id, relation)
        VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE anime_id = ?, relation_id = ?, relation = ?
        "#,
        related_anime_id,
        anime_id,
        relation,
        related_anime_id,
        anime_id,
        relation,
    )
    .execute(&state.db)
    .await;

    if let Err(e) = relation_isert_result {
        tracing::error!("Failed to insert relation link: {}", e);
        return false;
    }

    true
}

async fn add_user_anime(
    state: AppState,
    anime_id: i32,
    user_id: String,
    anime_watch_status: String,
) -> bool {
    // tracing::info!(
    //     "Adding user anime between anime {} and user {}",
    //     anime_id,
    //     user_id
    // );
    let relation_isert_result = sqlx::query!(
        r#"
                INSERT INTO anime_users (anime_id, user_id, status)
                VALUES (?, ?, ?)
                ON DUPLICATE KEY UPDATE anime_id = ?, user_id = ?, status = ?
                "#,
        anime_id,
        user_id,
        anime_watch_status,
        anime_id,
        user_id,
        anime_watch_status,
    )
    .execute(&state.db)
    .await;

    if let Err(e) = relation_isert_result {
        tracing::error!("Failed to insert anime_user: {}", e);
        return false;
    }

    true
}

async fn get_queue_items(state: AppState, max: usize) -> Vec<ImportQueueItem> {
    let mut process_items = vec![];
    for _ in 0..max {
        let item = timeout(Duration::from_millis(500), state.import_queue.pop()).await;

        if let Ok(item) = item {
            let times_in_queue = match &item {
                ImportQueueItem::Anime { times_in_queue, .. } => times_in_queue,
                ImportQueueItem::Relationship { times_in_queue, .. } => times_in_queue,
                ImportQueueItem::UserAnime { times_in_queue, .. } => times_in_queue,
            };
            let anime_id = match &item {
                ImportQueueItem::Anime { anime_id, .. } => anime_id,
                ImportQueueItem::Relationship { anime_id, .. } => anime_id,
                ImportQueueItem::UserAnime { anime_id, .. } => anime_id,
            };

            if times_in_queue > &5 {
                tracing::warn!(
                    "Anime {} has been in queue for too long, skipping...",
                    anime_id
                );
                continue;
            }

            process_items.push(item);
        } else {
            break;
        }
    }

    process_items
}

fn get_id_for_import(item: &ImportQueueItem) -> i32 {
    match item {
        ImportQueueItem::Anime { anime_id, .. } => *anime_id,
        ImportQueueItem::Relationship { anime_id, .. } => *anime_id,
        ImportQueueItem::UserAnime { anime_id, .. } => *anime_id,
    }
}

fn get_ids_for_import(items: Vec<ImportQueueItem>) -> Vec<i32> {
    items.iter().map(get_id_for_import).collect()
}

fn get_anime_from_anilist_result(result: AnilistResponse, i: usize) -> Option<AniListAnimeItem> {
    match i {
        0 => result.data.anime1,
        1 => result.data.anime2,
        2 => result.data.anime3,
        3 => result.data.anime4,
        4 => result.data.anime5,
        5 => result.data.anime6,
        6 => result.data.anime7,
        7 => result.data.anime8,
        8 => result.data.anime9,
        9 => result.data.anime10,
        10 => result.data.anime11,
        11 => result.data.anime12,
        12 => result.data.anime13,
        13 => result.data.anime14,
        14 => result.data.anime15,
        15 => result.data.anime16,
        16 => result.data.anime17,
        17 => result.data.anime18,
        18 => result.data.anime19,
        19 => result.data.anime20,
        20 => result.data.anime21,
        21 => result.data.anime22,
        22 => result.data.anime23,
        23 => result.data.anime24,
        24 => result.data.anime25,
        25 => result.data.anime26,
        26 => result.data.anime27,
        27 => result.data.anime28,
        28 => result.data.anime29,
        29 => result.data.anime30,
        30 => result.data.anime31,
        31 => result.data.anime32,
        32 => result.data.anime33,
        33 => result.data.anime34,
        34 => result.data.anime35,
        _ => None,
    }
}

async fn get_ids_for_update(db: sqlx::Pool<MySql>, items: Vec<i32>) -> Vec<i32> {
    let local_data = anime::get_local_anime_datas(db, items)
        .await
        .unwrap_or(vec![]);

    local_data
        .iter()
        .filter(|d| {
            let now = chrono::Utc::now().naive_utc();
            let updated_at = d.updated_at;
            let diff = now - updated_at;

            if diff.num_hours() > 1 {
                tracing::info!(
                    "Anime {} was last updated more than 1 hour ago, reimporting",
                    d.id
                );
                return true;
            }

            false
        })
        .map(|d| d.id)
        .collect()
}

struct AnimeIds {
    id: i32,
}

struct AnimeRelationInsertItem {
    anime_id: i32,
    related_anime_id: i32,
    relation: String,
}

#[derive(Debug)]
struct SeriesRelations {
    anime_id: Option<i32>,
    relation_id: Option<i32>,
    relation: Option<String>,
}

async fn build_series_group(state: AppState, anime: AniListAnimeItem) {
    let series_id = anime.id_mal.unwrap();

    let query = sqlx::query_as!(
        SeriesRelations,
        r#"
        WITH RECURSIVE related AS (
            SELECT anime_id, relation_id, relation
            FROM anime_relations
            WHERE anime_id = ?
        
            UNION ALL
        
            SELECT t.anime_id, t.relation_id, t.relation
            FROM anime_relations t
            JOIN related r ON t.anime_id = r.relation_id
        )
    
        SELECT * FROM related;
    "#,
        series_id,
    );

    let data = query.fetch_all(&state.db).await;

    if let Err(e) = data {
        tracing::error!("Failed to get series relations: {}", e);
        return;
    }

    let data = data.unwrap();

    println!("{:?}", data);
}

pub fn import_queue_worker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Starting import queue worker...");

        loop {
            tracing::info!("Items in queue: {}", state.import_queue.len());
            let queue_items = get_queue_items(state.clone(), MAX_ANILIST_PER_QUERY).await;

            if queue_items.is_empty() {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                continue;
            }

            let import_ids = get_ids_for_import(queue_items.clone());

            if import_ids.is_empty() {
                continue;
            }

            tracing::info!(
                "Fetching anime data for {}",
                import_ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );

            let anilist_result =
                anime::get_animes_from_anilist(state.reqwest.clone(), import_ids.clone()).await;

            if anilist_result.retry_after != -1 {
                tracing::warn!(
                    "Rate limit hit, sleeping for {}",
                    anilist_result.retry_after
                );

                for id in import_ids.iter() {
                    let item = queue_items
                        .iter()
                        .find(|item| get_id_for_import(item) == *id);
                    state.import_queue.push(item.unwrap().clone());
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(
                    anilist_result.retry_after as u64,
                ))
                .await;
            }

            let anilist_response = match anilist_result.response {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to get anime: {}, readding to queue", e);

                    import_ids.iter().for_each(|id| {
                        let item = queue_items.iter().find(|item| match item {
                            ImportQueueItem::Anime { anime_id, .. } => *anime_id == *id,
                            ImportQueueItem::Relationship { anime_id, .. } => *anime_id == *id,
                            ImportQueueItem::UserAnime { anime_id, .. } => *anime_id == *id,
                        });
                        match item {
                            Some(ImportQueueItem::Anime { times_in_queue, .. }) => {
                                state.import_queue.push(ImportQueueItem::Anime {
                                    anime_id: *id,
                                    times_in_queue: times_in_queue + 1,
                                })
                            }
                            Some(ImportQueueItem::Relationship {
                                times_in_queue,
                                anime_id,
                                related_anime_id,
                                related_anime_type,
                            }) => state.import_queue.push(ImportQueueItem::Relationship {
                                anime_id: *anime_id,
                                related_anime_id: *related_anime_id,
                                related_anime_type: related_anime_type.clone(),
                                times_in_queue: times_in_queue + 1,
                            }),
                            _ => {}
                        };
                    });
                    continue;
                }
            };

            let mut animes = vec![];
            let error_ids = get_error_ids(anilist_response.clone(), import_ids.clone());
            if !error_ids.is_empty() {
                tracing::warn!(
                    "Failed to get anime data for: {}",
                    error_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                );
                let requeue_ids = queue_items
                    .iter()
                    .filter_map(|item| {
                        let id = match item {
                            ImportQueueItem::Anime { anime_id, .. } => anime_id,
                            ImportQueueItem::Relationship { anime_id, .. } => anime_id,
                            ImportQueueItem::UserAnime { anime_id, .. } => anime_id,
                        };

                        if !error_ids.contains(id) {
                            return Some(item.clone());
                        }
                        None
                    })
                    .collect::<Vec<ImportQueueItem>>();

                for item in requeue_ids {
                    state.import_queue.push(item);
                }
            } else {
                for (index, id) in import_ids.iter().enumerate() {
                    let anime_data = get_anime_from_anilist_result(anilist_response.clone(), index);

                    if anime_data.is_none() {
                        tracing::warn!("Anime {} not found in response, readding to queue...", id);

                        let times_in_queue = match queue_items
                            .iter()
                            .find(|item| get_id_for_import(item) == *id)
                        {
                            Some(ImportQueueItem::Anime { times_in_queue, .. }) => {
                                times_in_queue + 1
                            }
                            Some(ImportQueueItem::Relationship { times_in_queue, .. }) => {
                                times_in_queue + 1
                            }
                            Some(ImportQueueItem::UserAnime { times_in_queue, .. }) => {
                                times_in_queue + 1
                            }
                            _ => 0,
                        };

                        state.import_queue.push(ImportQueueItem::Anime {
                            anime_id: *id,
                            times_in_queue,
                        });

                        continue;
                    }

                    let anime_data = anime_data.unwrap();
                    animes.push(anime_data);
                }
            }

            if animes.is_empty() {
                continue;
            }

            let insert_result = insert_animes(
                &state.db,
                animes
                    .iter()
                    .map(|a| InsertAnime {
                        cover_image: a.cover_image.clone(),
                        id_mal: a.id_mal.unwrap(),
                        status: a.status.clone(),
                        title: a.title.clone(),
                        season: a.season.clone(),
                        season_year: a.season_year,
                    })
                    .collect(),
            )
            .await;

            if let Err(e) = insert_result {
                tracing::error!("Failed to insert anime: {}", e);
                import_ids.iter().for_each(|id| {
                    state.import_queue.push(ImportQueueItem::Anime {
                        anime_id: *id,
                        times_in_queue: 0,
                    })
                });
                continue;
            }

            let user_anime_inserts = import_ids
                .iter()
                .filter_map(|id| {
                    queue_items.iter().find(|item| match item {
                        ImportQueueItem::UserAnime { anime_id, .. } => anime_id == id,
                        _ => false,
                    })
                })
                .collect::<Vec<&ImportQueueItem>>();

            // TODO: Make both of these into bulk inserts
            for user_anime in user_anime_inserts {
                let anime_id = match user_anime {
                    ImportQueueItem::UserAnime { anime_id, .. } => *anime_id,
                    _ => continue,
                };

                let user_id = match user_anime {
                    ImportQueueItem::UserAnime { user_id, .. } => user_id.clone(),
                    _ => continue,
                };

                let anime_watch_status = match user_anime {
                    ImportQueueItem::UserAnime {
                        anime_watch_status, ..
                    } => anime_watch_status.clone(),
                    _ => continue,
                };

                let user_anime_insert_result = add_user_anime(
                    state.clone(),
                    anime_id,
                    user_id.clone(),
                    anime_watch_status.clone(),
                )
                .await;

                if !user_anime_insert_result {
                    tracing::error!(
                        "Failed to insert user anime {} for user {}",
                        anime_id,
                        user_id
                    );
                    continue;
                }
            }

            queue_related_animes(state.clone(), animes).await;

            let relation_inserts = import_ids
                .iter()
                .filter_map(|id| {
                    queue_items.iter().find(|item| match item {
                        ImportQueueItem::Relationship { anime_id, .. } => anime_id == id,
                        _ => false,
                    })
                })
                .collect::<Vec<&ImportQueueItem>>();

            for relation in relation_inserts {
                let anime_id = match relation {
                    ImportQueueItem::Relationship { anime_id, .. } => *anime_id,
                    _ => continue,
                };

                let related_anime_id = match relation {
                    ImportQueueItem::Relationship {
                        related_anime_id, ..
                    } => *related_anime_id,
                    _ => continue,
                };

                let related_anime_type = match relation {
                    ImportQueueItem::Relationship {
                        related_anime_type, ..
                    } => related_anime_type.clone(),
                    _ => continue,
                };

                let relation_insert_result = add_anime_relation(
                    state.clone(),
                    anime_id,
                    related_anime_id,
                    related_anime_type,
                )
                .await;

                if !relation_insert_result {
                    tracing::error!(
                        "Failed to anime {} to series {}",
                        anime_id,
                        related_anime_id
                    );
                    continue;
                }
            }
        }
    });
}
