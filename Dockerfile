FROM rust:1.65-bullseye AS chef
RUN cargo install cargo-chef
WORKDIR /app

ARG ETHERSCAN_API_KEY

ARG SLACK_WEBHOOK_URL

ARG HORIZON_STELLAR_NODE

ARG REALIO_STELLAR_NODE

ENV ETHERSCAN_API_KEY=$ETHERSCAN_API_KEY

ENV SLACK_WEBHOOK_URL=$SLACK_WEBHOOK_URL

ENV HORIZON_STELLAR_NODE=$HORIZON_STELLAR_NODE

ENV REALIO_STELLAR_NODE=$REALIO_STELLAR_NODE

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release


FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates

ARG ETHERSCAN_API_KEY

ARG SLACK_WEBHOOK_URL

ARG HORIZON_STELLAR_NODE

ARG REALIO_STELLAR_NODE

ENV ETHERSCAN_API_KEY=$ETHERSCAN_API_KEY

ENV SLACK_WEBHOOK_URL=$SLACK_WEBHOOK_URL

ENV HORIZON_STELLAR_NODE=$HORIZON_STELLAR_NODE

ENV REALIO_STELLAR_NODE=$REALIO_STELLAR_NODE
WORKDIR /app
EXPOSE 3000
COPY --from=builder /app/target/release/notifications /usr/local/bin

ENTRYPOINT [ "/usr/local/bin/notifications" ]