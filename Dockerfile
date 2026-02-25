FROM rust:1.88 AS builder
WORKDIR /usr/src/blog
COPY . .
RUN cargo install --locked --path .

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/blog /usr/local/bin/blog
WORKDIR /usr/local/bin
CMD ["blog"]