use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Connection {
    pub connection_id: i64,
    pub folder_id: Option<i64>,
    pub user_id: i64,
    pub connection_name: String,
    pub host: String,
    pub port: Option<i64>, // INTEGER in SQLite is i64 usually, nullable
    pub db_name: String,
    pub db_user: String,
    pub db_password_encrypted: String,
    pub ssl_mode: Option<String>,
}
