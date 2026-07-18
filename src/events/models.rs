use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    Virtual,
    InPerson,
    Hybrid,
}

#[derive(Serialize, Clone, sqlx::FromRow)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
    pub created_at: DateTime<Utc>,

    #[serde(skip_serializing)]
    pub user_id: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateEvent {
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateEvent {
    pub name: String,
    pub details: Option<String>,
    pub event_type: EventType,
    pub location: Option<String>,
}