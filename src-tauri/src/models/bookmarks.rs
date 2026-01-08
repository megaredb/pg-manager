use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Bookmark {
    pub bookmark_id: i64,
    pub connection_id: i64,
    pub schema_name: String,
    pub object_name: String,
    pub object_type: String,
}
