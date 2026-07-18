use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,

    #[serde(skip_serializing)]
    pub password_hash: String,

    pub email_verified_at: Option<DateTime<Utc>>,

    // Internal bookkeeping for the resend cooldown — not the client's business.
    #[serde(skip_serializing)]
    pub last_verification_sent_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ResendVerification {
    pub email: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}