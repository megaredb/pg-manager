use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QueryHistory {
    pub history_id: i64,
    pub connection_id: i64,
    pub query_text: String,
    pub status: String,
    pub execution_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub executed_at: Option<NaiveDateTime>,
}
