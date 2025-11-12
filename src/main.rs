mod db;
mod handlers;
mod routes;

use crate::db::init_db;
use crate::handlers::{auth::Backend, handler_404};

use anyhow::Result;
use axum_login::{
    AuthManagerLayerBuilder,
    tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer},
};
use dotenvy::dotenv;
use minijinja::{Environment, path_loader};

use time::Duration;
use tokio::{signal, task::AbortHandle};
//use tower_sessions::cookie::Key;
use tower_sessions_sqlx_store::PostgresStore;
//use minijinja_autoreload::AutoReloader;
use std::{env, sync::Arc};

#[derive(Clone)]
pub struct AppState {
    db_pool: db::DbPool,
    pub templates: Environment<'static>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // setting up our db connections
    let database_url = env::var("DATABASE_URL")?;

    let db_pool = init_db(&database_url).await?;

    // session layer
    let session_store = PostgresStore::new(db_pool.clone());
    session_store
        .migrate()
        .await
        .expect("Something went wrong...");

    // removing expired sessions
    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    //let key = Key::generate();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    // init template engine and add templates
    let mut templates = Environment::new();
    templates.set_loader(path_loader("templates"));

    // pass env to handlers via state
    let app_state = Arc::new(AppState { templates, db_pool });

    let backend = Backend::new(app_state.clone());

    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

    // define routes
    let app = routes::create_router(app_state)
        .layer(auth_layer)
        .fallback(handler_404);
    //.layer(login_manager);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await?;

    deletion_task.await??;

    Ok(())
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to instal Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}
