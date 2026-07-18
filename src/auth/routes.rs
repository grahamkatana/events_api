use super::handlers::{login, register};
use crate::common::state::SharedState;
use axum::{routing::post, Router};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .with_state(state)
}