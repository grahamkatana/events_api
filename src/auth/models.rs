use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub email: String,

    // Never send this back in a JSON response, even by accident.
    #[serde(skip_serializing)]
    pub password_hash: String,

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

// `sub` ("subject") and `exp` ("expiration") are standard JWT claim
// names — using them means any JWT-aware tool can read this token
// even outside our own app.
#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}