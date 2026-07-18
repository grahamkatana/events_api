use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,

    // Never send this back in a JSON response, even by accident.
    #[serde(skip_serializing)]
    pub password_hash: String,

    // Visible to the client — useful to show "please verify your email"
    // in a UI, so this one is NOT skip_serializing.
    pub email_verified_at: Option<DateTime<Utc>>,

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

// Used with the `Query` extractor to read `?token=...` from the URL.
#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
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