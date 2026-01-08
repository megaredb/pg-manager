use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Diagram {
    pub diagram_id: i64,
    pub connection_id: i64,
    pub diagram_name: String,
    pub definition_json: String,
    pub created_at: Option<NaiveDateTime>,
}
