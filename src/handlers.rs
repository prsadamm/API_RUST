use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use crate::{
    auth::{AdminUser, AuthUser},
    error::AppError,
    jwt::create_token,
    models::{UserPublic, UserRow},
    password::{hash_password, verify_password},
    state::AppState,
};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

fn validate_register(email: &str, password: &str) -> Result<(), AppError> {
    let valid_email = email.len() <= 254
        && email.contains('@')
        && !email.starts_with('@')
        && !email.ends_with('@');

    let valid_password = (8..=128).contains(&password.len());

    if valid_email && valid_password {
        Ok(())
    } else {
        Err(AppError::BadRequest)
    }
}

fn dummy_hash() -> &'static str {
    static DUMMY: OnceLock<String> = OnceLock::new();
    DUMMY.get_or_init(|| {
        hash_password("mot-de-passe-factice").expect("génération du hash factice")
    })
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserPublic>), AppError> {
    let email = normalize_email(&body.email);
    validate_register(&email, &body.password)?;

    let password = body.password;
    let hash = tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|_| AppError::Internal)??;

    let result = sqlx::query_as::<_, UserPublic>(
        "INSERT INTO users (email, password_hash)
         VALUES ($1, $2)
         RETURNING id, email, role",
    )
    .bind(&email)
    .bind(&hash)
    .fetch_one(&state.db)
    .await;

    match result {
        Ok(user) => Ok((StatusCode::CREATED, Json(user))),

        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(AppError::EmailTaken),

        Err(e) => Err(e.into()),
    }
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AppError> {
    let email = normalize_email(&body.email);

    let user = sqlx::query_as::<_, UserRow>(
        "SELECT id, password_hash, role FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.db)
    .await?;

    let (hash_to_check, user) = match user {
        Some(u) => (u.password_hash.clone(), Some(u)),
        None => (dummy_hash().to_string(), None),
    };

    let password = body.password;
    let password_ok =
        tokio::task::spawn_blocking(move || verify_password(&password, &hash_to_check))
            .await
            .map_err(|_| AppError::Internal)?;

    let user = match (user, password_ok) {
        (Some(u), true) => u,
        _ => return Err(AppError::InvalidCredentials),
    };

    let token = create_token(user.id, &user.role, &state.jwt_secret)?;

    Ok(Json(TokenResponse { token }))
}

pub async fn me(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<UserPublic>, AppError> {
        let user = sqlx::query_as::<_, UserPublic>(
        "SELECT id, email, role FROM users WHERE id = $1",
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(Json(user))
}

pub async fn list_users(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<UserPublic>>, AppError> {
    let users = sqlx::query_as::<_, UserPublic>("SELECT id, email, role FROM users")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(users))
}