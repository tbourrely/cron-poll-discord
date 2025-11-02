FROM rust:1.89-slim-bullseye AS builder
RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/cron-poll-discord
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim AS base
RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev && rm -rf /var/lib/apt/lists/*

FROM base AS bot
COPY --from=builder /usr/local/cargo/bin/bot /usr/local/bin/bot
CMD ["bot"]

FROM base AS sender
COPY --from=builder /usr/local/cargo/bin/sender /usr/local/bin/sender
CMD ["sender"]

FROM base AS api
COPY --from=builder /usr/local/cargo/bin/api /usr/local/bin/api
CMD ["api"]
