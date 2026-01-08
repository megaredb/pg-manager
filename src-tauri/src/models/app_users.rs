use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AppUser {
    pub user_id: i64,
    pub username: String,
    pub password_hash: String,
    pub role: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}
