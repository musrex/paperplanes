use crate::{AppState, handlers::user_functions::*};
use axum::http::StatusCode;
use axum::{
    Router,
    extract::State,
    response::Html,
    routing::{get, patch},
};
use minijinja::context;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handler_home))
        .route("/users", get(get_users))
        .route("/hello", get(json_handler))
        .route("/users/message", patch(set_user_message))
}

use axum_extra::extract::cookie::CookieJar;

async fn handler_home(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
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
