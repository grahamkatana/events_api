use super::handlers::{create_event, delete_event, get_event, list_events, update_event};
use crate::common::state::SharedState;
use axum::{routing::get, Router};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/events", get(list_events).post(create_event))
        .route(
            "/events/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .with_state(state)
}