use crate::commands::app_user_logs::log_action_internal;
use crate::models::connections::Connection;
use crate::password::decrypt_data;

use chrono::Utc;
use serde_json::Value;
use sqlx::{Column, PgPool, Row, SqlitePool};
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Schema {
    pub schema_name: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Table {
    pub table_name: String,
    pub table_schema: String,
}
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct View {
    pub view_name: String,
    pub view_schema: String,
}
#[derive(serde::Serialize, Debug, Clone, sqlx::FromRow)]
pub struct ForeignKeyRelation {
    pub constraint_name: String,
    pub schema_name: String,
    pub table_name: String,
    pub column_name: String,
    pub foreign_schema_name: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}
#[derive(serde::Serialize, Debug, Clone, sqlx::FromRow)]
pub struct ColumnDef {
    pub table_name: String,
    pub column_name: String,
    pub data_type: String,
}

fn create_pg_connection_string(connection: &Connection) -> Result<String, String> {
    let password = decrypt_data(&connection.db_password_encrypted)
        .map_err(|_| "Could not decrypt connection password".to_string())?;

    let port = connection.port.unwrap_or(5432);
    let ssl_mode = connection.ssl_mode.as_deref().unwrap_or("prefer");

    Ok(format!(
        "postgresql://{}:{}@{}:{}/{}?sslmode={}",
        connection.db_user, password, connection.host, port, connection.db_name, ssl_mode
    ))
}

#[tauri::command]
pub async fn test_connection(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<bool, String> {
    let connection =
        sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id = ?")
            .bind(connection_id)
            .fetch_one(&*pool)
            .await
            .map_err(|e| e.to_string())?;
    let conn_str = create_pg_connection_string(&connection)?;
    match PgPool::connect(&conn_str).await {
        Ok(pg_pool) => {
            pg_pool.close().await;
            Ok(true)
        }
        Err(e) => Err(format!("Failed to connect: {e}")),
    }
}
#[tauri::command]
pub async fn get_schemas(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
) -> Result<Vec<Schema>, String> {
    let connection =
        sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id = ?")
            .bind(connection_id)
            .fetch_one(&*pool)
            .await
            .map_err(|e| e.to_string())?;
    let conn_str = create_pg_connection_string(&connection)?;
    let pg_pool = PgPool::connect(&conn_str)
        .await
        .map_err(|e| e.to_string())?;
    let schemas = sqlx::query_as::<_, Schema>(
        r#"
    SELECT SCHEMA_NAME
    FROM information_schema.schemata
    WHERE SCHEMA_NAME NOT IN ('pg_catalog',
                              'information_schema',
                              'pg_toast')
    ORDER BY SCHEMA_NAME
    "#,
    )
    .fetch_all(&pg_pool)
    .await
    .map_err(|e| e.to_string())?;
    pg_pool.close().await;
    Ok(schemas)
}

#[derive(serde::Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
    pub execution_time_ms: u64,
    pub row_count: u64,
}

#[tauri::command]
pub async fn execute_query(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    query_text: String,
) -> Result<QueryResult, String> {
    let connection =
        sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id = ?")
            .bind(connection_id)
            .fetch_one(&*pool)
            .await
            .map_err(|e| format!("Failed to fetch connection: {e}"))?;

    let conn_str = create_pg_connection_string(&connection)?;
    let pg_pool = PgPool::connect(&conn_str)
        .await
        .map_err(|e| e.to_string())?;

    let start = std::time::Instant::now();

    let result = sqlx::query(&query_text).fetch_all(&pg_pool).await;

    let duration = start.elapsed().as_millis() as u64;
    let executed_at = Utc::now().naive_utc();

    match result {
        Ok(pg_rows) => {
            let mut columns = Vec::new();
            let mut rows = Vec::new();
            let row_count = pg_rows.len() as u64;

            if let Some(first_row) = pg_rows.first() {
                for col in first_row.columns() {
                    columns.push(col.name().to_string());
                }
            }

            for row in pg_rows {
                let mut row_data = Vec::new();
                for (i, _) in row.columns().iter().enumerate() {
                    let value: Value = if let Ok(v) = row.try_get::<i32, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<String, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<bool, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<f64, _>(i) {
                        serde_json::json!(v)
                    } else if let Ok(v) = row.try_get::<i64, _>(i) {
                        serde_json::json!(v)
                    } else {
                        serde_json::json!(null)
                    };
                    row_data.push(value);
                }
                rows.push(row_data);
            }

            let _ = sqlx::query(
                "INSERT INTO query_history (connection_id, query_text, status, execution_time_ms, executed_at) VALUES (?, ?, 'success', ?, ?)"
            )
            .bind(connection_id)
            .bind(&query_text)
            .bind(duration as i64)
            .bind(executed_at)
            .execute(&*pool)
            .await;

            let short_query = if query_text.len() > 50 {
                format!("{}...", &query_text[0..47])
            } else {
                query_text.clone()
            };
            let _ = log_action_internal(
                &pool,
                connection.user_id,
                "EXECUTE_QUERY",
                Some(&short_query),
            )
            .await;

            pg_pool.close().await;

            Ok(QueryResult {
                columns,
                rows,
                execution_time_ms: duration,
                row_count,
            })
        }
        Err(e) => {
            let error_msg = e.to_string();

            let _ = sqlx::query(
                "INSERT INTO query_history (connection_id, query_text, status, execution_time_ms, error_message, executed_at) VALUES (?, ?, 'error', ?, ?, ?)"
            )
            .bind(connection_id)
            .bind(&query_text)
            .bind(duration as i64)
            .bind(&error_msg)
            .bind(executed_at)
            .execute(&*pool)
            .await;

            let _ = log_action_internal(&pool, connection.user_id, "QUERY_ERROR", Some(&error_msg))
                .await;

            pg_pool.close().await;
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn get_tables(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    schema_name: String,
) -> Result<Vec<Table>, String> {
    let c = sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id=?")
        .bind(connection_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let s = create_pg_connection_string(&c)?;
    let p = PgPool::connect(&s).await.map_err(|e| e.to_string())?;
    let r = sqlx::query_as::<_, Table>(
        r#"
    SELECT TABLE_NAME,
       table_schema
    FROM information_schema.tables
    WHERE table_schema=$1
    AND table_type='BASE TABLE'
    ORDER BY TABLE_NAME
    "#,
    )
    .bind(schema_name)
    .fetch_all(&p)
    .await
    .map_err(|e| e.to_string())?;
    p.close().await;
    Ok(r)
}
#[tauri::command]
pub async fn get_views(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    schema_name: String,
) -> Result<Vec<View>, String> {
    let c = sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id=?")
        .bind(connection_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let s = create_pg_connection_string(&c)?;
    let p = PgPool::connect(&s).await.map_err(|e| e.to_string())?;
    let r = sqlx::query_as::<_, View>(
        r#"
    SELECT TABLE_NAME AS view_name,
                     table_schema AS view_schema
    FROM information_schema.views
    WHERE table_schema=$1
    ORDER BY TABLE_NAME"#,
    )
    .bind(schema_name)
    .fetch_all(&p)
    .await
    .map_err(|e| e.to_string())?;
    p.close().await;
    Ok(r)
}
#[tauri::command]
pub async fn get_foreign_keys(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    schema_name: String,
) -> Result<Vec<ForeignKeyRelation>, String> {
    let c = sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id=?")
        .bind(connection_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let s = create_pg_connection_string(&c)?;
    let p = PgPool::connect(&s).await.map_err(|e| e.to_string())?;
    let r = sqlx::query_as::<_, ForeignKeyRelation>(
        r#"
    SELECT tc.constraint_name,
       tc.table_schema AS SCHEMA_NAME,
       tc.table_name,
       kcu.column_name,
       ccu.table_schema AS foreign_schema_name,
       ccu.table_name AS foreign_table_name,
       ccu.column_name AS foreign_column_name
    FROM information_schema.table_constraints AS tc
    JOIN information_schema.key_column_usage AS kcu ON tc.constraint_name = kcu.constraint_name
    AND tc.table_schema = kcu.table_schema
    JOIN information_schema.constraint_column_usage AS ccu ON ccu.constraint_name = tc.constraint_name
    AND ccu.table_schema = tc.table_schema
    WHERE tc.constraint_type = 'FOREIGN KEY'
    AND tc.table_schema = $1
    "#,
    )
    .bind(schema_name)
    .fetch_all(&p)
    .await
    .map_err(|e| e.to_string())?;
    p.close().await;
    Ok(r)
}
#[tauri::command]
pub async fn get_schema_columns(
    pool: State<'_, SqlitePool>,
    connection_id: i64,
    schema_name: String,
) -> Result<Vec<ColumnDef>, String> {
    let c = sqlx::query_as::<_, Connection>("SELECT * FROM connections WHERE connection_id=?")
        .bind(connection_id)
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    let s = create_pg_connection_string(&c)?;
    let p = PgPool::connect(&s).await.map_err(|e| e.to_string())?;
    let r = sqlx::query_as::<_, ColumnDef>(
        r#"SELECT 
        table_name, 
        column_name, 
        data_type 
        FROM information_schema.columns 
        WHERE table_schema = $1 
        ORDER BY table_name, ordinal_position"#,
    )
    .bind(schema_name)
    .fetch_all(&p)
    .await
    .map_err(|e| e.to_string())?;
    p.close().await;
    Ok(r)
}
