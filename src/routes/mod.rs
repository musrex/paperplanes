pub mod art;
pub mod auth;
pub mod home;

use crate::AppState;
use axum::Router;
use std::sync::Arc;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(home::routes())
        .merge(art::routes())
        .merge(auth::routes())
        .with_state(app_state)
}
