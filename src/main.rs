mod common;
mod events;

use common::state::{AppState, SharedState};
use events::models::Event;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let initial_state = AppState {
        events: vec![
            Event { id: 1, name: String::from("Rust Meetup") },
            Event { id: 2, name: String::from("Jazz Night") },
        ],
        next_id: 3,
    };

    let shared: SharedState = Arc::new(Mutex::new(initial_state));

    let app = events::routes::build_router(shared);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}