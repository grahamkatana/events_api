use crate::events::models::Event;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub events: Vec<Event>,
    pub next_id: u32,
}

pub type SharedState = Arc<Mutex<AppState>>;