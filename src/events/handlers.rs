use super::models::{CreateEvent, Event, UpdateEvent};
use crate::auth::extractor::{AuthUser, VerifiedUser};
use crate::common::state::SharedState;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

const EVENT_COLUMNS: &str =
    "id, name, details, event_type, location, cover_image_url, created_at, user_id";

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
    user: VerifiedUser,
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

pub async fn upload_cover(
    State(state): State<SharedState>,
    user: AuthUser,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<Event>, StatusCode> {
    require_ownership(&state, id, user.user_id).await?;

    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type = String::from("application/octet-stream");
    let mut extension = String::from("bin");

    // A multipart request is made of one or more named "fields" —
    // we loop through them looking for the one named "file". Real
    // forms could have other fields alongside it (e.g. a caption).
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if field.name() == Some("file") {
            content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            if let Some(name) = field.file_name() {
                if let Some(ext) = name.rsplit('.').next() {
                    extension = ext.to_string();
                }
            }

            let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            file_bytes = Some(data.to_vec());
        }
    }

    let bytes = file_bytes.ok_or(StatusCode::BAD_REQUEST)?;

    // Reject anything that isn't declared as an image. Not bulletproof
    // (a client could lie about content-type) but a reasonable first
    // line of defense; a stricter version would also inspect the raw
    // bytes for a real image "magic number" before trusting this.
    if !content_type.starts_with("image/") {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let key = format!("events/{id}/{}.{extension}", Uuid::new_v4());

    let url = state
        .storage
        .upload(&key, bytes, &content_type)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let query = format!(
        "UPDATE events SET cover_image_url = $1 WHERE id = $2 RETURNING {EVENT_COLUMNS}"
    );
    let event = sqlx::query_as::<_, Event>(sqlx::AssertSqlSafe(query))
        .bind(url)
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(event))
}