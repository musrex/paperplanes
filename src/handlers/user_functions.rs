use crate::AppState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Serialize)]
struct ProfileMessage {
    id: i32,
    content: String,
}

#[derive(Serialize, FromRow)]
struct User {
    id: i32,
    username: String,
}

#[derive(Serialize)]
struct Users {
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
