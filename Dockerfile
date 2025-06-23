FROM lukemathwalker/cargo-chef:latest-rust-1.87-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json --bin server --no-default-features

# Build application
COPY . .
RUN cargo build --release --bin server --no-default-features

# Run binary
FROM debian:bookworm-slim AS runtime

WORKDIR /app
COPY --from=builder /app/target/release/server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/server", "-p", "9000"]
