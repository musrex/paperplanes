use crate::AppState;

use argon2::{
    Argon2,
    password_hash::{
        Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};

use axum::{
    Json,
    extract::{Form, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use axum_login::{AuthUser, AuthnBackend};

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub pw_hash: Vec<u8>,
}

#[derive(Serialize, FromRow, serde::Deserialize)]
pub struct NewUser {
    username: String,
    password: String,
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[derive(Clone)]
pub struct Backend {
    state: Arc<AppState>,
}

impl Backend {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[derive(Clone, Deserialize)]
pub struct Credentials {
    username: String,
    password: String,
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = std::convert::Infallible;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query!(
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            creds.username
        )
        .fetch_optional(&self.state.db_pool)
        .await
        .unwrap();

        let Some(row) = row else {
            return Ok(None);
        };

        // password verification
        let parsed_hash = PasswordHash::new(&row.password_hash).unwrap();
        if Argon2::default()
            .verify_password(creds.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            Ok(Some(User {
                id: row.id as i64,
                username: row.username,
                pw_hash: row.password_hash.into_bytes(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, id: &i64) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query!(
            "SELECT id, username, password_hash FROM users WHERE id = $1",
            id as &i64
        )
        .fetch_optional(&self.state.db_pool)
        .await
        .unwrap();

        Ok(row.map(|r| User {
            id: r.id as i64,
            username: r.username,
            pw_hash: r.password_hash.into_bytes(),
        }))
    }
}

pub fn hash_passwords(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let pw_bytes = password.as_bytes();
    let password_hash = argon2.hash_password(pw_bytes, &salt)?.to_string();

    Ok(password_hash)
}

type AuthSession = axum_login::AuthSession<Backend>;

// TODO:
// Refactor to handle JSON
pub async fn handle_login(
    State(_state): State<Arc<AppState>>,
    mut auth_session: AuthSession,
    Json(payload): Json<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(payload).await {
        Ok(Some(user)) => user,
        Ok(None) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    Redirect::to("/").into_response()
}

pub async fn handle_logout(
    State(_state): State<Arc<AppState>>,
    mut auth_session: AuthSession,
) -> impl IntoResponse {
    let _ = auth_session.logout().await;

    Redirect::to("/")
}

pub async fn handle_register(
    State(state): State<Arc<AppState>>,
    Form(form): Form<Credentials>,
) -> Redirect {
    let password_hash = hash_passwords(&form.password).unwrap();
    let user_id = Uuid::new_v4();

    sqlx::query!(
        "INSERT INTO users (uuid, username, password_hash)
         VALUES ($1, $2, $3)",
        user_id,
        form.username,
        password_hash
    )
    .execute(&state.db_pool)
    .await
    .unwrap();

    Redirect::to("/login")
}

// TODO:
// Better error handling for the password hashing function
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<NewUser>,
) -> impl IntoResponse {
    let uuid = Uuid::new_v4();
    let password = hash_passwords(&payload.password).unwrap();
    let user = sqlx::query!(
        r#"
          INSERT INTO users (uuid, username, password_hash)
          VALUES ($1, $2, $3)
        "#,
        uuid,
        payload.username,
        password,
    )
    .execute(&state.db_pool)
    .await;

    match user {
        Ok(info) if info.rows_affected() > 0 => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status" : "updated",
                "username": payload.username,
            })),
        )
            .into_response(),

        Ok(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error" : format!("Error creating user {}", payload.username)
            })),
        )
            .into_response(),

        Err(e) => {
            eprintln!("Database error while creating user: {e}.");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Database error while creating user: {e}."
                })),
            )
                .into_response()
        }
    }
}
