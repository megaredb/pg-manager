use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AppUserLog {
    pub log_id: i64,
    pub user_id: i64,
    pub action_type: String,
    pub details: Option<String>,
    pub timestamp: Option<NaiveDateTime>,
}
