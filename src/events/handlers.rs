use super::models::{CreateEvent, Event, UpdateEvent};
use crate::auth::extractor::AuthUser;
use crate::common::state::SharedState;
use axum::{extract::{Path, State}, http::StatusCode, Json};

const EVENT_COLUMNS: &str = "id, name, details, event_type, location, created_at, user_id";

pub async fn list_events(
    State(state): State<SharedState>,
) -> Result<Json<Vec<Event>>, StatusCode> {
    let query = format!("SELECT {EVENT_COLUMNS} FROM events ORDER BY id");
    let events = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
        .fetch_all(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Result<Json<Event>, StatusCode> {
    let query = format!("SELECT {EVENT_COLUMNS} FROM events WHERE id = $1");
    let event = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
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
    let query = format!(
        "INSERT INTO events (name, details, event_type, location, user_id)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING {EVENT_COLUMNS}"
    );
    let event = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
        .bind(payload.name)
        .bind(payload.details)
        .bind(payload.event_type)
        .bind(payload.location)
        .bind(user.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}

async fn require_ownership(
    state: &SharedState,
    id: i32,
    user_id: i32,
) -> Result<(), StatusCode> {
    let query = format!("SELECT {EVENT_COLUMNS} FROM events WHERE id = $1");
    let existing = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

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

    let query = format!(
        "UPDATE events SET name = $1, details = $2, event_type = $3, location = $4
         WHERE id = $5
         RETURNING {EVENT_COLUMNS}"
    );
    let event = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
        .bind(payload.name)
        .bind(payload.details)
        .bind(payload.event_type)
        .bind(payload.location)
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