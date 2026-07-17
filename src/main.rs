// #[derive(Debug)]
// struct Event {
//     name: String,
//     location: String,
//     attendees: u32,
// }

// fn find_event<'a>(events: &'a [Event], name: &str) -> Option<&'a Event> {
//     for event in events {
//         if event.name == name {
//             return Some(event);
//         }
//     }
//     None

// }

// fn main() {
//     let name = "Events API";
//     let version = 1;

//     println!("Welcome to {}, version {}", name, version);
//     let result = add(5, 10);
//     println!("The sum of 5 and 10 is {}", result);
//     let event = Event {
//         name: String::from("Rust Conference"),
//         location: String::from("New York"),
//         attendees: 500,
//     };
//     println!("Event: {}, Location: {}, Attendees: {}", event.name, event.location, event.attendees);
//     println!("{:?}", event);

//     let events = vec![
//         Event {
//             name: String::from("Rust Conference"),
//             location: String::from("New York"),
//             attendees: 500,
//         },
//         Event {
//             name: String::from("Web Development Meetup"),
//             location: String::from("San Francisco"),
//             attendees: 200,
//         },
//     ];
//     println!("{:?}", events);
//     let found_event = find_event(&events, "Rust Conference");
//     println!("Found event: {:?}", found_event);
// }

// fn add(a: i32, b: i32)->i32 {
//     return a + b;
// }


// async fn greet() -> String {
//     String::from("Hello from an async function!")
// }

// #[tokio::main]
// async fn main() {
//     let message = greet().await;
//     println!("{}", message);
// }


// use axum::{
//     extract::{Path, State},
//     routing::get, 
//     Router
// };

// use std::sync::{Arc, Mutex};

// struct Event {
//     id: u32,
//     name: String,
// }

// // The shared state our whole app can access: a list of events,
// // safely shared across many requests at once.
// type SharedState = Arc<Mutex<Vec<Event>>>;

// #[tokio::main]
// async fn main() {
//      let events: Vec<Event> = vec![
//         Event { id: 1, name: String::from("Rust Meetup") },
//         Event { id: 2, name: String::from("Jazz Night") },
//     ];
//     let state: SharedState = Arc::new(Mutex::new(events));
//     let app = Router::new()
//         .route("/", get(root))
//         .route("/events", get(list_events))
//         .route("/events/{id}", get(get_event))
//         .with_state(state);

//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
//         .await
//         .unwrap();

//     println!("Listening on http://0.0.0.0:3000");

//     axum::serve(listener, app).await.unwrap();
// }

// async fn root() -> &'static str {
//     "Hello, Events API!"
// }

// async fn list_events(State(state): State<SharedState>) -> String {
//     let events = state.lock().unwrap();
//     let mut output = String::new();
//     for event in events.iter() {
//         output.push_str(&format!("{}: {}\n", event.id, event.name));
//     }
//     output
// }

// async fn get_event(
//     State(state): State<SharedState>,
//     Path(id): Path<u32>,
// ) -> String {
//     let events = state.lock().unwrap();
//     match events.iter().find(|e| e.id == id) {
//         Some(event) => format!("{}: {}", event.id, event.name),
//         None => String::from("Event not found"),
//     }
// }

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Serialize, Clone)]
struct Event {
    id: u32,
    name: String,
}

#[derive(Deserialize)]
struct CreateEvent {
    name: String,
}

#[derive(Deserialize)]
struct UpdateEvent {
    name: String,
}

struct AppState {
    events: Vec<Event>,
    next_id: u32,
}

type SharedState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() {
    let state = AppState {
        events: vec![
            Event { id: 1, name: String::from("Rust Meetup") },
            Event { id: 2, name: String::from("Jazz Night") },
        ],
        next_id: 3,
    };

    let shared: SharedState = Arc::new(Mutex::new(state));

    let app = Router::new()
        .route("/events", get(list_events).post(create_event))
        .route(
            "/events/{id}",
            get(get_event).put(update_event).delete(delete_event),
        )
        .with_state(shared);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn list_events(State(state): State<SharedState>) -> Json<Vec<Event>> {
    let state = state.lock().unwrap();
    Json(state.events.clone())
}

async fn get_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
) -> Result<Json<Event>, axum::http::StatusCode> {
    let state = state.lock().unwrap();
    match state.events.iter().find(|e| e.id == id) {
        Some(event) => Ok(Json(event.clone())),
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

async fn create_event(
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

async fn update_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
    Json(payload): Json<UpdateEvent>,
) -> Result<Json<Event>, axum::http::StatusCode> {
    let mut state = state.lock().unwrap();
    match state.events.iter_mut().find(|e| e.id == id) {
        Some(event) => {
            event.name = payload.name;
            Ok(Json(event.clone()))
        }
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

async fn delete_event(
    State(state): State<SharedState>,
    Path(id): Path<u32>,
) -> axum::http::StatusCode {
    let mut state = state.lock().unwrap();
    let original_len = state.events.len();
    state.events.retain(|e| e.id != id);

    if state.events.len() < original_len {
        axum::http::StatusCode::NO_CONTENT
    } else {
        axum::http::StatusCode::NOT_FOUND
    }
}