use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Clone, sqlx::Type, ToSchema)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Virtual,
    InPerson,
    Hybrid,
}

#[derive(Serialize, Clone, sqlx::FromRow, ToSchema)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
    pub cover_image_url: Option<String>,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing)]
    #[schema(ignore)]
    pub user_id: Option<i32>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateEvent {
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateEvent {
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
}