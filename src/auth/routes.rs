use super::handlers::{login, register, verify_email};
use crate::common::state::SharedState;
use axum::{
    routing::{get, post},
    Router,
};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/verify-email", get(verify_email))
        .with_state(state)
}