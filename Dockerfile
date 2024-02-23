FROM rust:1.76.0 AS builder

WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
CMD ["./zero2prod"]
