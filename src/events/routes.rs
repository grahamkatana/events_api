use super::handlers::{create_event, delete_event, get_event, list_events, update_event, upload_cover};
use super::ws::ws_handler;
use crate::common::state::SharedState;
use axum::{routing::get, Router};

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/events", get(list_events).post(create_event))
        .route(
            "/events/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/events/{id}/cover", axum::routing::post(upload_cover))
        .route("/ws", get(ws_handler))
        .with_state(state)
}