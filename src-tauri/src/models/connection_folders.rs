use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConnectionFolder {
    pub folder_id: i64,
    pub user_id: i64,
    pub folder_name: String,
}
