use crate::AppState;
use axum::http::StatusCode;
use axum::{Router, extract::State, response::Html, routing::get};
use minijinja::context;
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/login", get(handler_login))
}

async fn handler_login(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("login.jinja").unwrap();

    let rendered = template.render(()).unwrap();

    Ok(Html(rendered))
}
