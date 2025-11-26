use crate::AppState;
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, serde::Deserialize)]
pub struct ProfileMessage {
    id: i32,
    content: String,
}

#[derive(Serialize, FromRow, serde::Deserialize)]
pub struct User {
    id: i32,
    username: String,
}

#[derive(Serialize, FromRow, serde::Deserialize)]
pub struct NewUser {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct Users {
    users: Vec<User>,
}

pub async fn json_handler() -> impl IntoResponse {
    let payload = serde_json::json!({
    "greeting": "Hello from Axum",
    "detail": "Te amo Cordelia",
    });
    Json(payload)
}

#[axum::debug_handler]
pub async fn set_user_message(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ProfileMessage>,
) -> impl IntoResponse {
    let query = sqlx::query!(
        r#"
        UPDATE users 
        SET profile_text = $1 
        WHERE id = $2 
        "#,
        payload.content,
        payload.id,
    )
    .execute(&state.db_pool)
    .await;

    match query {
        Ok(info) if info.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status"  : "updated",
                "user_id" : payload.id
            })),
        )
            .into_response(),

        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error" : format!("User with id {} not found", payload.id),
            })),
        )
            .into_response(),

        Err(e) => {
            eprintln!("Database error when updating profile: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error" : "Database error when updating profile: {e}"
                })),
            )
                .into_response()
        }
    }
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let rows: Result<Vec<User>, sqlx::Error> = sqlx::query_as!(
        User,
        r#"
        SELECT id, username FROM users
        "#,
    )
    .fetch_all(&state.db_pool)
    .await;

    match rows {
        Ok(users) => {
            let payload = Users { users };
            Json(payload).into_response()
        }
        Err(e) => {
            eprintln!("Database error while fetching users: {e}");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to fetch users."
                })),
            )
                .into_response()
        }
    }
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewUser>,
) -> impl IntoResponse {
    let password = payload.password;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    let uuid = Uuid::new_v4();
    let user = sqlx::query!(
        r#"
          INSERT INTO users (uuid, username, password_hash)
          VALUES ($1, $2, $3)
        "#,
        uuid,
        payload.username,
        password_hash.to_string(),
    )
    .execute(&state.db_pool)
    .await;

    match user {
        Ok(info) if info.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status" : "updated",
                "username": payload.username,
            })),
        )
            .into_response(),

        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error" : format!("Error creating user {}", payload.username)
            })),
        )
            .into_response(),

        Err(e) => {
            eprintln!("Database error while creating user: {e}.");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database error while creating user: {e}."
                })),
            )
                .into_response()
        }
    }
}
