use super::models::{CreateEvent, Event, UpdateEvent};
use crate::common::state::SharedState;
use axum::{extract::{Path, State}, http::StatusCode, Json};

pub async fn list_events(State(state): State<SharedState>) -> Json<Vec<Event>> {
    let state = state.lock().unwrap();
    Json(state.events.clone())
}

pub async fn get_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
) -> Result<Json<Event>, StatusCode> {
    let state = state.lock().unwrap();
    match state.events.iter().find(|e| e.id == id) {
        Some(event) => Ok(Json(event.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_event(
    State(state): State<SharedState>,
    Json(payload): Json<CreateEvent>,
) -> Json<Event> {
    let mut state = state.lock().unwrap();
    let new_event = Event {
        id: state.next_id,
        name: payload.name,
    };
    state.events.push(new_event.clone());
    state.next_id += 1;
    Json(new_event)
}

pub async fn update_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<Event>, StatusCode> {
    let mut state = state.lock().unwrap();
    match state.events.iter_mut().find(|e| e.id == id) {
        Some(event) => {
            event.name = payload.name;
            Ok(Json(event.clone()))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
) -> StatusCode {
    let mut state = state.lock().unwrap();
    let original_len = state.events.len();
    state.events.retain(|e| e.id != id);

    if state.events.len() < original_len {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}