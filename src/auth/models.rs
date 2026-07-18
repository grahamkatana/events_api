use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Clone, sqlx::FromRow, ToSchema)]
pub struct User {
    pub id: i32,
    pub email: String,

    #[serde(skip_serializing)]
    #[schema(ignore)]
    pub password_hash: String,

    pub email_verified_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing)]
    #[schema(ignore)]
    pub last_verification_sent_at: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ResendVerification {
    pub email: String,
}

#[derive(Serialize, ToSchema)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[derive(Serialize, ToSchema)]
pub struct TokenResponse {
    pub token: String,
}