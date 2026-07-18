use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // brings `.oneshot(...)` into scope

// Integration tests live in `tests/` (not inside `src/`) and can ONLY
// see our crate's `pub` API — `events_api::build_app` — exactly the
// way an outside user of our library would. This is a genuine,
// separate compiled crate, one per file in this folder.

async fn test_app() -> axum::Router {
    dotenvy::dotenv().ok();

    // A SEPARATE database from your real dev data — see the .env
    // instructions. Running tests should never touch data you care
    // about, and definitely shouldn't fail just because your dev
    // database has leftover rows from manual testing.
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set in .env to run integration tests");

    events_api::build_app(&database_url, "test-jwt-secret".to_string()).await
}

#[tokio::test]
async fn listing_events_returns_200_with_json_array() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn creating_event_without_a_token_is_unauthorized() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/events")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"name":"Test Event","details":null,"event_type":"virtual","location":null}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn getting_a_nonexistent_event_returns_404() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/events/999999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}