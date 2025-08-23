use crate::AppState;

use argon2::{
    Argon2,
    password_hash::{
        Error, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng,
    },
};

use axum::{
    extract::{Form, FromRequestParts, State},
    http::{StatusCode, request::Parts},
    response::{Html, IntoResponse, Redirect},
};

use axum_login::{AuthUser, AuthnBackend, UserId};

use axum_extra::extract::cookie::{Cookie, CookieJar};

//use minijinja::{State, context};
use serde::Deserialize;
use sqlx;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub pw_hash: Vec<u8>,
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

pub async fn show_login(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("login.jinja").unwrap();

    let rendered = template.render(()).unwrap();

    Ok(Html(rendered))
}

type AuthSession = axum_login::AuthSession<Backend>;

pub async fn handle_login(
    State(_state): State<Arc<AppState>>,
    mut auth_session: AuthSession,
    Form(form): Form<Credentials>,
) -> impl IntoResponse {
    let user = match auth_session.authenticate(form.clone()).await {
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

pub async fn show_register(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("register.jinja").unwrap();

    let rendered = template.render(()).unwrap();

    Ok(Html(rendered))
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
