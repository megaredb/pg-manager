use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub tag_id: i64,
    pub user_id: i64,
    pub tag_name: String,
    pub color_hex: Option<String>,
}
