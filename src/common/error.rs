use axum::http::StatusCode;
use std::fmt::Display;

// Used in place of `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)`
// throughout the app — logs the REAL error (visible in your terminal)
// before converting it to the generic status code the client sees.
// Clients should never see raw database/internal error details (that
// can leak schema info or internals), but WE need to see them.
pub fn log_error<E: Display>(err: E) -> StatusCode {
    tracing::error!("{err}");
    StatusCode::INTERNAL_SERVER_ERROR
}