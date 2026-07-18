use super::models::{Claims, LoginUser, RegisterUser, TokenResponse, User, VerifyEmailQuery};
use crate::common::state::SharedState;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use uuid::Uuid;

const USER_COLUMNS: &str = "id, email, password_hash, email_verified_at, created_at";

// Loaded from disk into the compiled binary at COMPILE TIME — not read
// from the filesystem while the server is running. If this file is
// missing when you build, compilation fails immediately, not at 2am
// when someone finally registers in production.
const VERIFY_EMAIL_TEMPLATE: &str = include_str!("../../templates/verify_email.html");

pub async fn register(
    State(state): State<SharedState>,
    Json(payload): Json<RegisterUser>,
) -> Result<Json<User>, StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();

    let verification_token = Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    let query = format!(
        "INSERT INTO users (email, password_hash, verification_token, verification_token_expires_at)
         VALUES ($1, $2, $3, $4)
         RETURNING {USER_COLUMNS}"
    );

    let user = sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(query))
        .bind(&payload.email)
        .bind(password_hash)
        .bind(&verification_token)
        .bind(expires_at)
        .fetch_one(&state.db)
        .await
        .map_err(|err| {
            if let sqlx::Error::Database(db_err) = &err {
                if db_err.is_unique_violation() {
                    return StatusCode::CONFLICT;
                }
            }
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let base_url = std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verify_link = format!("{base_url}/auth/verify-email?token={verification_token}");
    let html_body = VERIFY_EMAIL_TEMPLATE.replace("{{VERIFY_LINK}}", &verify_link);

    // We deliberately don't fail registration if the email fails to
    // send (e.g. MailHog is briefly down) — the user account still
    // exists, they'd just need to request a new verification link.
    // We just log it for now.
    if let Err(err) = state
        .mailer
        .send(&payload.email, "Confirm your email", html_body)
        .await
    {
        eprintln!("Failed to send verification email: {err}");
    }

    Ok(Json(user))
}

pub async fn login(
    State(state): State<SharedState>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<TokenResponse>, StatusCode> {
    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE email = $1");
    let user = sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(query))
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
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

pub async fn verify_email(
    State(state): State<SharedState>,
    Query(params): Query<VerifyEmailQuery>,
) -> Result<Html<&'static str>, StatusCode> {
    let result = sqlx::query(
        "UPDATE users
         SET email_verified_at = now(),
             verification_token = NULL,
             verification_token_expires_at = NULL
         WHERE verification_token = $1
           AND verification_token_expires_at > now()",
    )
    .bind(&params.token)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() > 0 {
        Ok(Html(
            "<h1>Email verified!</h1><p>You can close this tab and return to the app.</p>",
        ))
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}