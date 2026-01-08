use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConnectionTag {
    pub tag_id: i64,
    pub connection_id: i64,
}
