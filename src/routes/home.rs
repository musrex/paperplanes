use crate::AppState;
use crate::handlers::auth::Backend;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, extract::State, response::Html, routing::get};
use minijinja::context;
use serde::Serialize;
use std::sync::Arc;

use axum_login::{AuthSession, AuthUser};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler_home))
        .route("/hello", get(json_handler))
}

use axum_extra::extract::cookie::{Cookie, CookieJar};

#[derive(Serialize)]
struct Message {
    greeting: &'static str,
    detail: &'static str,
}

async fn json_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let payload = Message {
        greeting: "Hello from Axum",
        detail: "Te amo Cordelia",
    };
    Json(payload)
}

async fn get_user(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let users = sqlx::query!("SELECT * FROM users")
        .fetch_all(&state.db_pool)
        .await;
    let payload = Message { users: users };
    Json(payload)
}

async fn handler_home(
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<Backend>,
) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("home.jinja").unwrap();

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
