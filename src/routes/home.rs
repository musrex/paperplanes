use crate::AppState;
use crate::handlers::auth::Backend;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, extract::State, response::Html, routing::get};
use axum_login::AuthSession;
use minijinja::context;
use serde::Serialize;
use sqlx::FromRow;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler_home))
        .route("/users", get(get_users))
        .route("/hello", get(json_handler))
}

use axum_extra::extract::cookie::{Cookie, CookieJar};

#[derive(Serialize)]
struct Message {
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

async fn json_handler() -> impl IntoResponse {
    let payload = serde_json::json!({
    "greeting": "Hello from Axum",
    "detail": "Te amo Cordelia",
    });
    Json(payload)
}

async fn set_user_message(
    State(state): State<Arc<AppState>>,
    message: Message,
) -> impl IntoResponse {
}

async fn get_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

async fn handler_home(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<Backend>,
) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("home.jinja").unwrap();
    let jar = CookieJar::new();
    // This needs to be refactored. I don't like the nested if statements.
    // I think good Rust is to use "?" and let the errors bubble up.
    let username = if let Some(cookie) = jar.get("user_id") {
        if let Ok(user_id) = cookie.value().parse::<i32>() {
            if let Ok(record) = sqlx::query!("SELECT username FROM users WHERE id = $1", user_id)
                .fetch_one(&state.db_pool)
                .await
            {
                Some(record.username)
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let rendered = template
        .render(context! {
            username => username,
        })
        .unwrap();

    Ok(Html(rendered))
}
