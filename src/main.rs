mod db;
mod handlers;
mod routes;

use crate::db::init_db;
use crate::handlers::handler_404;

use anyhow::{Context, Result};
use dotenvy::dotenv;
use minijinja::Environment;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    db_pool: db::DbPool,
    pub templates: Environment<'static>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let db_pool = init_db(&database_url).await?;

    // init template engine and add templates
    let mut templates = Environment::new();
    templates
        .add_template("home", include_str!("../templates/home.jinja"))
        .unwrap();

    // pass env to handlers via state
    let app_state = Arc::new(AppState { templates, db_pool });

    // define routes
    let app = routes::create_router(app_state).fallback(handler_404);
    //        .route("/art", get(handler_art))
    //        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await?;

    Ok(())
}
