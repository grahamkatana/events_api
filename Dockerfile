# ---- Build stage ----
FROM rust:1.93-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY templates ./templates

RUN cargo build --release

# ---- Runtime stage ----
FROM debian:bookworm-slim

# ca-certificates is required for our outbound HTTPS calls (reqwest to
# Digital Samba/AWS SDK to MinIO/lettre to SMTP-over-TLS) to be able to
# verify TLS certificates. Easy to forget, and the failure it causes
# otherwise (cert verification errors) is a genuinely confusing one to
# debug the first time you hit it.
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/events_api ./events_api
COPY templates ./templates
COPY migrations ./migrations

EXPOSE 3000

CMD ["./events_api"]