FROM rust:1.85-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/clawtrade /app/clawtrade
COPY --from=builder /app/dashboard /app/dashboard
COPY --from=builder /app/config /app/config
ENV PORT=8080
EXPOSE 8080
CMD ["./clawtrade"]
