use crate::AppState;
use axum::http::StatusCode;
use axum::{Router, extract::State, response::Html, routing::get};
use minijinja::context;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler_home))
}

use axum_extra::extract::cookie::{Cookie, CookieJar};

async fn handler_home(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("home.jinja").unwrap();

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
