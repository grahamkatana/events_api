# Events API

A REST API for managing events, built with Rust, Axum, and PostgreSQL.

## Stack

- **Axum** — HTTP framework
- **Tokio** — async runtime
- **sqlx** — Postgres client with compile-time query checking
- **PostgreSQL** — primary database
- **MinIO** — S3-compatible object storage for event cover images
- **MailHog** — local SMTP catcher for development email
- **Digital Samba** — video conferencing API for virtual/hybrid event meeting links
- **utoipa** + Swagger UI — OpenAPI docs

## Features

- Event CRUD (create, list, get, update, delete)
- Paginated event listing
- JWT-based authentication (register, login)
- Email verification with resend + rate limiting
- Ownership enforcement (only an event's creator can modify or delete it)
- Cover image upload via MinIO
- Automatic video meeting link generation for virtual/hybrid events (Digital Samba)
- Live updates over WebSocket when events change
- Request logging (console + rotating daily log files)
- OpenAPI documentation via Swagger UI

## Project structure

```
src/
├── lib.rs              # builds the app; used by main.rs and by tests
├── main.rs             # process entrypoint: logging setup, starts the server
├── common/             # shared infrastructure (DB state, email, storage, errors, docs)
└── <feature>/           # one folder per feature (events, auth), each with:
    ├── mod.rs
    ├── models.rs        # request/response/DB types
    ├── handlers.rs       # route logic
    └── routes.rs          # path -> handler wiring
```

Each feature is self-contained. Code shared across features lives in `common/`.

## Prerequisites

- Rust (stable, installed via [rustup](https://rustup.rs))
- Docker Desktop
- [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli):
  ```
  cargo install sqlx-cli --no-default-features --features rustls,postgres
  ```

## Setup

1. Copy `.env.example` to `.env` (or use the provided `.env`) and adjust values if needed.

2. Start supporting services:
   ```
   docker compose up -d
   ```
   This starts Postgres, Adminer, MailHog, and MinIO. See the table below for ports.

3. Run migrations:
   ```
   sqlx migrate run
   ```

4. Start the API:
   ```
   cargo run
   ```

The API listens on `http://localhost:3000`.

## Services and ports

| Service | Purpose | URL |
|---|---|---|
| API | the application itself | http://localhost:3000 |
| Swagger UI | interactive API docs | http://localhost:3000/swagger-ui |
| Postgres | database | localhost:5455 |
| Adminer | database UI | http://localhost:8081 |
| MailHog | dev email inbox | http://localhost:8026 |
| MinIO API | object storage | http://localhost:9100 |
| MinIO console | storage UI | http://localhost:9101 |

## Testing

Unit tests run against pure logic (password hashing, serialization) and need no external services:

```
cargo test --lib
```

Integration tests exercise the full HTTP router against a real database. Create a separate test database first:

```sql
CREATE DATABASE events_db_test;
```

Run migrations against it, then run the full test suite:

```
sqlx migrate run --database-url postgres://events_user:events_password@localhost:5455/events_db_test
cargo test
```

## Pagination

`GET /events` accepts `page` and `per_page` query parameters (defaults: page 1, 20 per page; `per_page` is capped at 100). Response shape:

```json
{
  "data": [ ... ],
  "page": 1,
  "per_page": 20,
  "total": 47,
  "total_pages": 3
}
```

## Video meetings

When creating an event with `"event_type": "virtual"` or `"hybrid"`, the API automatically creates a video conferencing room via Digital Samba and returns its URL as `meeting_url` on the created event. `in_person` events always have `meeting_url: null`. If room creation fails (e.g. the video provider is down), the event is still created — `meeting_url` is simply left `null` and the failure is logged.

## WebSocket

Connect to `ws://localhost:3000/ws` to receive live notifications whenever an event is created, updated, or deleted. Messages are JSON, tagged by `type`:

```json
{"type":"created","event":{...}}
{"type":"updated","event":{...}}
{"type":"deleted","id":5}
```

## Authentication

1. `POST /auth/register` — creates a user, sends a verification email
2. Confirm via the link in the email (check MailHog if running locally)
3. `POST /auth/login` — returns a JWT
4. Send the token as `Authorization: Bearer <token>` on protected routes

Creating an event requires a verified email. Updating or deleting an event requires being its owner.

## Environment variables

| Variable | Purpose |
|---|---|
| `DATABASE_URL` | Postgres connection string |
| `TEST_DATABASE_URL` | Postgres connection string for integration tests |
| `JWT_SECRET` | signing key for JWTs |
| `SMTP_HOST`, `SMTP_PORT`, `SMTP_FROM` | outgoing email configuration |
| `APP_BASE_URL` | used to build links in emails |
| `MINIO_ENDPOINT`, `MINIO_ACCESS_KEY`, `MINIO_SECRET_KEY`, `MINIO_BUCKET`, `MINIO_PUBLIC_URL` | object storage configuration |
| `DIGITAL_SAMBA_API_KEY`, `DIGITAL_SAMBA_TEAM_NAME` | video conferencing (get these free at digitalsamba.com, no card required) |

## Logging

Logs are written to both the console and `storage/logs/` (one file per day). Control verbosity with the `RUST_LOG` environment variable, e.g.:

```
RUST_LOG=events_api=debug cargo run
```