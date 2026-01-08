use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PinnedQuery {
    pub pinned_query_id: i64,
    pub connection_id: i64,
    pub query_name: String,
    pub query_text: String,
    pub description: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}
