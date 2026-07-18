use super::models::Claims;
use crate::common::state::SharedState;
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};

// A handler that takes `AuthUser` as a parameter can only be reached
// with a valid, unexpired JWT — Axum runs this extraction *before*
// the handler body, and rejects the request automatically if it fails.
pub struct AuthUser {
    pub user_id: i32,
}

impl FromRequestParts<SharedState> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthUser {
            user_id: token_data.claims.sub,
        })
    }
}

// Same idea as `AuthUser`, but additionally requires a verified email.
// Any handler that takes `VerifiedUser` instead of `AuthUser` gets
// this check for free — no separate "middleware function" needed.
pub struct VerifiedUser {
    pub user_id: i32,
}

impl FromRequestParts<SharedState> for VerifiedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &SharedState,
    ) -> Result<Self, Self::Rejection> {
        // Reuse AuthUser's logic first — this still requires a valid,
        // unexpired JWT. Only then do we check verification status.
        let AuthUser { user_id } = AuthUser::from_request_parts(parts, state).await?;

        let verified_at: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT email_verified_at FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_one(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if verified_at.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }

        Ok(VerifiedUser { user_id })
    }
}