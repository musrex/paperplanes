use axum::{http::StatusCode, response::IntoResponse};

pub mod art;
pub mod auth;
pub mod user_functions;

pub async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404: nothing to see here")
}
