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
struct User {
    id: i32,
    username: String,
    pw_hash: Vec<u8>,
}

impl AuthUser for User {
    type Id = i32;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.pw_hash
    }
}

#[derive(Clone)]
struct Backend {
    state: Arc<AppState>,
}

impl Backend {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[derive(Clone)]
struct Credentials {
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
                id: row.id,
                username: row.username,
                pw_hash: row.password_hash.into_bytes(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, id: &i32) -> Result<Option<Self::User>, Self::Error> {
        let row = sqlx::query!(
            "SELECT id, username, password_hash FROM users WHERE id = $1",
            id
        )
        .fetch_optional(&self.state.db_pool)
        .await
        .unwrap();

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            pw_hash: r.password_hash.into_bytes(),
        }))
    }
}

#[derive(Deserialize)]
pub struct AuthForm {
    pub username: String,
    pub password: String,
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

pub async fn handle_login(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<AuthForm>,
) -> impl IntoResponse {
    let user = sqlx::query!(
        "SELECT id, password_hash FROM users WHERE username = $1",
        form.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .expect("DB Error - Username not found.");

    if let Some(user) = user {
        let parsed_hash = PasswordHash::new(&user.password_hash).unwrap();
        if Argon2::default()
            .verify_password(form.password.as_bytes(), &parsed_hash)
            .is_ok()
        {
            let cookie = Cookie::build(("user_id", user.id.to_string()))
                .path("/")
                .http_only(true);

            return (jar.add(cookie), Redirect::to("/")).into_response();
        }
    }

    Redirect::to("/login").into_response()
}

pub async fn show_register(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    let template = state.templates.get_template("register.jinja").unwrap();

    let rendered = template.render(()).unwrap();

    Ok(Html(rendered))
}

pub async fn handle_register(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AuthForm>,
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
