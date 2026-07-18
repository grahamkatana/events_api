use super::models::{Claims, LoginUser, RegisterUser, TokenResponse, User};
use crate::common::state::SharedState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;

pub async fn register(
    State(state): State<SharedState>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<User>, StatusCode> {
    // Generate a random "salt" — extra randomness mixed into the hash so
    // that two users with the same password don't produce the same hash.
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (email, password_hash) VALUES ($1, $2)
         RETURNING id, email, password_hash, created_at",
    )
    .bind(payload.email)
    .bind(password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        // A duplicate email violates our UNIQUE constraint from the
        // migration — Postgres reports this as a specific error code.
        // We detect it and return a proper 409 Conflict, instead of
        // a generic 500, so the client knows exactly what went wrong.
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(user))
}

pub async fn login(
    State(state): State<SharedState>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    // No such email? Treat it the same as "wrong password" below —
    // never let an attacker learn which emails are registered.
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let parsed_hash = PasswordHash::new(&user.password_hash)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user.id,
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TokenResponse { token }))
}