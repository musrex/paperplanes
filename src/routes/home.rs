use crate::AppState;
use crate::handlers::auth::Backend;
use axum::http::StatusCode;
use axum::{Router, extract::State, response::Html, routing::get};
use minijinja::context;
use std::sync::Arc;

use axum_login::{AuthSession, AuthUser};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(handler_home))
}

use axum_extra::extract::cookie::{Cookie, CookieJar};

async fn handler_home(
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession<Backend>,
) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("home.jinja").unwrap();

    let username = auth_session.user.as_ref().map(|user| user.username.clone());

    let rendered = template
        .render(context! {
            username => username,
        })
        .unwrap();

    Ok(Html(rendered))
}
