use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub struct Event {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CreateEvent {
    pub name: String,
}

#[derive(Deserialize)]
pub struct UpdateEvent {
    pub name: String,
}