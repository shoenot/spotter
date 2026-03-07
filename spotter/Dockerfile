# --- Build stage ---
FROM rust:1.94-slim AS builder

# Install system deps needed to compile (openssl, pg client headers)
RUN apt-get update && apt-get install -y pkg-config libssl-dev libpq-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency manifests first and build a dummy binary.
# Docker caches each instruction as a layer. By building deps before copying
# your source, this layer is reused on rebuilds as long as Cargo.toml/lock
# haven't changed — so you don't redownload/recompile all deps every time.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm src/main.rs

# Now copy your actual source and the .sqlx offline query cache.
# SQLX_OFFLINE=true tells sqlx not to connect to a database at compile time,
# and instead use the cached query metadata in .sqlx/
COPY src ./src
COPY .sqlx ./.sqlx
COPY migrations ./migrations
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

# --- Runtime stage ---
FROM debian:bookworm-slim

# Install runtime dependencies (libpq for postgres, ca-certs for TLS to Spotify API)
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy only the compiled binary from the build stage
COPY --from=builder /app/target/release/spotter .

# Copy migrations so spotter can run them on startup
COPY --from=builder /app/migrations ./migrations

CMD ["./spotter"]
