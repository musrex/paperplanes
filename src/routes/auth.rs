use crate::{AppState, handlers::auth::*};

use axum::{Router, routing::get};

use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(show_login).post(handle_login))
        .route("/register", get(show_register).post(handle_register))
}
