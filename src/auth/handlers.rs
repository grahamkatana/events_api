use super::models::{
    Claims, LoginUser, MessageResponse, RegisterUser, ResendVerification, TokenResponse, User,
    VerifyEmailQuery,
};
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
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use uuid::Uuid;

const USER_COLUMNS: &str =
    "id, email, password_hash, email_verified_at, last_verification_sent_at, created_at";

const VERIFY_EMAIL_TEMPLATE: &str = include_str!("../../templates/verify_email.html");

// How long a user must wait between requesting new verification emails.
const RESEND_COOLDOWN: Duration = Duration::seconds(60);

// Shared by `register` and `resend_verification` — builds the email
// and sends it via whatever `EmailSender` is configured.
async fn send_verification_email(state: &SharedState, to: &str, token: &str) {
    let base_url =
        std::env::var("APP_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());
    let verify_link = format!("{base_url}/auth/verify-email?token={token}");
    let html_body = VERIFY_EMAIL_TEMPLATE.replace("{{VERIFY_LINK}}", &verify_link);

    if let Err(err) = state.mailer.send(to, "Confirm your email", html_body).await {
        eprintln!("Failed to send verification email: {err}");
    }
}

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
    let now = Utc::now();
    let expires_at = now + Duration::hours(24);

    let query = format!(
        "INSERT INTO users
            (email, password_hash, verification_token, verification_token_expires_at, last_verification_sent_at)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING {USER_COLUMNS}"
    );

    let user = sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(query))
        .bind(&payload.email)
        .bind(password_hash)
        .bind(&verification_token)
        .bind(expires_at)
        .bind(now)
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

    send_verification_email(&state, &payload.email, &verification_token).await;

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

pub async fn resend_verification(
    State(state): State<SharedState>,
    Json(payload): Json<ResendVerification>,
) -> Json<MessageResponse> {
    // Always return this exact message, no matter what happens below —
    // whether the email doesn't exist, is already verified, or is on
    // cooldown, an attacker gets the same response either way. This is
    // the same "don't leak account existence" principle from login.
    let generic = Json(MessageResponse {
        message: "If that email exists and isn't verified yet, a new verification link has been sent."
            .to_string(),
    });

    let query = format!("SELECT {USER_COLUMNS} FROM users WHERE email = $1");
    let user = match sqlx::query_as::<_, User>(sqlx::AssertSqlSafe(query))
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
    {
        Ok(Some(user)) => user,
        _ => return generic, // not found, or a DB error — respond identically
    };

    if user.email_verified_at.is_some() {
        return generic;
    }

    let now = Utc::now();
    if let Some(last_sent) = user.last_verification_sent_at {
        if now - last_sent < RESEND_COOLDOWN {
            // Still within cooldown — silently do nothing, but the
            // response looks exactly the same to the caller.
            return generic;
        }
    }

    let verification_token = Uuid::new_v4().to_string();
    let expires_at: DateTime<Utc> = now + Duration::hours(24);

    let update_result = sqlx::query(
        "UPDATE users
         SET verification_token = $1,
             verification_token_expires_at = $2,
             last_verification_sent_at = $3
         WHERE id = $4",
    )
    .bind(&verification_token)
    .bind(expires_at)
    .bind(now)
    .bind(user.id)
    .execute(&state.db)
    .await;

    if update_result.is_ok() {
        send_verification_email(&state, &payload.email, &verification_token).await;
    }

    generic
}