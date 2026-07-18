use super::models::{CreateEvent, Event, UpdateEvent};
use crate::auth::extractor::AuthUser;
use crate::common::state::SharedState;
use axum::{extract::{Path, State}, http::StatusCode, Json};

pub async fn list_events(
    State(state): State<SharedState>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT id, name, created_at, user_id FROM events ORDER BY id",
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Result<Json<Event>, StatusCode> {
    let event = sqlx::query_as::<_, Event>(
        "SELECT id, name, created_at, user_id FROM events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match event {
        Some(event) => Ok(Json(event)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_event(
    State(state): State<SharedState>,
    user: AuthUser,
    Json(payload): Json<CreateEvent>,
) -> Result<Json<Event>, StatusCode> {
    let event = sqlx::query_as::<_, Event>(
        "INSERT INTO events (name, user_id) VALUES ($1, $2)
         RETURNING id, name, created_at, user_id",
    )
    .bind(payload.name)
    .bind(user.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}

// Shared by update_event and delete_event: load the event, and make
// sure the requesting user is actually its owner before proceeding.
async fn require_ownership(
    state: &SharedState,
    id: i32,
    user_id: i32,
) -> Result<(), StatusCode> {
    let existing = sqlx::query_as::<_, Event>(
        "SELECT id, name, created_at, user_id FROM events WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // `existing.user_id` is `Option<i32>` — this is `None` for legacy
    // events created before we added ownership. `None != Some(x)` is
    // always true, so nobody can currently edit an ownerless event
    // through this check. That's a deliberate, safe default for now.
    if existing.user_id != Some(user_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(())
}

pub async fn update_event(
    State(state): State<SharedState>,
    user: AuthUser,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<Event>, StatusCode> {
    require_ownership(&state, id, user.user_id).await?;

    let event = sqlx::query_as::<_, Event>(
        "UPDATE events SET name = $1 WHERE id = $2
         RETURNING id, name, created_at, user_id",
    )
    .bind(payload.name)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}

pub async fn delete_event(
    State(state): State<SharedState>,
    user: AuthUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    require_ownership(&state, id, user.user_id).await?;

    sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}