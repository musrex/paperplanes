use crate::{AppState, handlers::art::*};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/art", get(handler_art))
        .route("/fractal_art", get(handler_fractal))
}
