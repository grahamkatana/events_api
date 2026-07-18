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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_type_serializes_to_snake_case() {
        // Locks in the exact JSON string clients will see — if someone
        // changes `rename_all` later, this test fails loudly instead
        // of silently breaking every client parsing "in_person".
        let json = serde_json::to_string(&EventType::InPerson).unwrap();
        assert_eq!(json, "\"in_person\"");
    }

    #[test]
    fn every_event_type_round_trips_through_json() {
        for variant in [EventType::Virtual, EventType::InPerson, EventType::Hybrid] {
            let json = serde_json::to_string(&variant).unwrap();
            let parsed: EventType = serde_json::from_str(&json).unwrap();
            // Comparing re-serialized JSON rather than the enum values
            // directly, since EventType doesn't derive PartialEq.
            assert_eq!(json, serde_json::to_string(&parsed).unwrap());
        }
    }
}