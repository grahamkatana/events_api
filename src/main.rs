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


use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, Events API!"
}