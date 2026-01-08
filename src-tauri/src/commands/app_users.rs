use crate::{
    commands::app_user_logs::log_action_internal,
    models::app_users::AppUser,
    password::{hash_password, verify_password},
};
use sqlx::SqlitePool;
use tauri::State;

#[derive(serde::Deserialize)]
pub struct CreateAppUserRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateAppUserRequest {
    pub user_id: i64,
    pub username: String,
    pub password: Option<String>,
    pub role: Option<String>,
}

#[tauri::command]
pub async fn create_app_user(
    pool: State<'_, SqlitePool>,
    request: CreateAppUserRequest,
) -> Result<i64, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO app_users (username, password_hash, role) VALUES (?, ?, ?) RETURNING user_id",
    )
    .bind(&request.username)
    .bind(hash_password(&request.password))
    .bind(request.role)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    let _ = log_action_internal(
        &pool,
        id,
        "REGISTER",
        Some(&format!("Username: {}", request.username)),
    )
    .await;

    Ok(id)
}

#[tauri::command]
pub async fn update_app_user(
    pool: State<'_, SqlitePool>,
    request: UpdateAppUserRequest,
) -> Result<(), String> {
    if let Some(new_password) = request.password {
        let new_hash = hash_password(&new_password);
        sqlx::query(
            "UPDATE app_users SET username = ?, role = ?, password_hash = ? WHERE user_id = ?",
        )
        .bind(request.username)
        .bind(request.role)
        .bind(new_hash)
        .bind(request.user_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        sqlx::query("UPDATE app_users SET username = ?, role = ? WHERE user_id = ?")
            .bind(request.username)
            .bind(request.role)
            .bind(request.user_id)
            .execute(&*pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_app_user(pool: State<'_, SqlitePool>, user_id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM app_users WHERE user_id = ?")
        .bind(user_id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_app_users(
    pool: State<'_, SqlitePool>,
    username_search: Option<String>,
) -> Result<Vec<AppUser>, String> {
    let mut sql = String::from("SELECT * FROM app_users");

    let has_search = username_search
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if has_search {
        sql.push_str(" ORDER BY (LOWER(username) LIKE LOWER(?)) DESC, username ASC");
    } else {
        sql.push_str(" ORDER BY username ASC");
    }

    let mut query = sqlx::query_as::<_, AppUser>(&sql);

    if has_search {
        if let Some(search) = &username_search {
            let search_pattern = format!("%{search}%");
            query = query.bind(search_pattern);
        }
    }

    query.fetch_all(&*pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn verify_user_credentials(
    pool: State<'_, SqlitePool>,
    username: String,
    password: String,
) -> Result<Option<AppUser>, String> {
    let result = sqlx::query_as::<_, AppUser>(
        "SELECT user_id, username, password_hash, role, created_at FROM app_users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        eprintln!("Database error during user retrieval: {e}");
        e.to_string()
    })?;

    if let Some(user) = result {
        if verify_password(&password, &user.password_hash) {
            let _ = log_action_internal(&pool, user.user_id, "LOGIN", Some("Success")).await;

            Ok(Some(user))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}
