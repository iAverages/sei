use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use sea_query::{Expr, Iden};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, sqlx::Type, Deserialize, Serialize)]
#[sqlx(rename_all = "PascalCase")]
pub enum AnimeJobQueueStatus {
    Pending,
    InProgress,
    Failed,
    Complete,
}

impl Display for AnimeJobQueueStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for AnimeJobQueueStatus {
    type Err = AnimeJobQueueStatusError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(AnimeJobQueueStatus::Pending),
            "InProgress" => Ok(AnimeJobQueueStatus::InProgress),
            "Failed" => Ok(AnimeJobQueueStatus::Failed),
            "Complete" => Ok(AnimeJobQueueStatus::Complete),
            _ => Err(AnimeJobQueueStatusError::ParseError),
        }
    }
}

impl From<AnimeJobQueueStatus> for Expr {
    fn from(val: AnimeJobQueueStatus) -> Self {
        Expr::Value(val.to_string().into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AnimeJobQueueStatusError {
    #[error("failed to parse anome job queue status")]
    ParseError,
}

// TODO: how to I remove this? I only want sqlx to be able to convert
// automatically from string to the enum
impl From<String> for AnimeJobQueueStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Pending" => AnimeJobQueueStatus::Pending,
            "InProgress" => AnimeJobQueueStatus::InProgress,
            "Failed" => AnimeJobQueueStatus::Failed,
            "Complete" => AnimeJobQueueStatus::Complete,
            _ => panic!("failed to convert string to AnimeJobQueueStatus"),
        }
    }
}
// impl TryFrom<String> for AnimeJobQueueStatus {
//     type Error = AnimeJobQueueStatusError;
//
//     fn try_from(value: String) -> Result<Self, Self::Error> {
//         let converted = match value.as_str() {
//             "Pending" => AnimeJobQueueStatus::Pending,
//             "InProgress" => AnimeJobQueueStatus::InProgress,
//             "Failed" => AnimeJobQueueStatus::Failed,
//             "Complete" => AnimeJobQueueStatus::Complete,
//             _ => return Err(AnimeJobQueueStatusError::ParseError),
//         };
//         Ok(converted)
//     }
// }

impl From<AnimeJobQueueStatus> for String {
    fn from(val: AnimeJobQueueStatus) -> Self {
        let value = match val {
            AnimeJobQueueStatus::Pending => "Pending",
            AnimeJobQueueStatus::InProgress => "InProgress",
            AnimeJobQueueStatus::Failed => "Failed",
            AnimeJobQueueStatus::Complete => "Complete",
        };

        value.to_string()
    }
}

#[derive(Iden)]
pub enum AnimeJobQueue {
    Table,
    Id,
    AnimeId,
    Status,
    CreatedAt,
    CompleteAt,
    TriggeredById,
}

#[derive(FromRow, Debug)]
pub struct DBAnimeJobQueue {
    pub id: String,
    pub anime_id: i32,
    pub status: String,
    pub triggered_by_id: Option<String>,
}
