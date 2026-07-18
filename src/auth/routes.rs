use super::handlers::{login, register, resend_verification, verify_email};
use crate::common::state::SharedState;
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

pub fn build_router(state: SharedState) -> Router {
    // Allows 2 requests/second per client IP, with bursts up to 5 —
    // generous for normal use, but shuts down anyone hammering
    // register/login/resend in a loop.
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .expect("valid rate limiter config"),
    );

    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/verify-email", get(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .layer(GovernorLayer::new(governor_conf))
        .with_state(state)
}