FROM rust:1.88-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY cli ./cli
COPY server ./server
COPY crates ./crates
COPY config ./config

RUN cargo build --release -p dweb-cloud-server -p dweb-cloud-cli

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app

COPY --from=builder /app/target/release/dweb-cloud-server /usr/local/bin/dweb-cloud-server
COPY --from=builder /app/target/release/dweb-cloud-cli /usr/local/bin/dweb-cloud-cli
COPY --from=builder /app/config ./config

ENV DWEB_CLOUD_HTTP=0.0.0.0:9080
ENV DWEB_CLOUD_DATA_DIR=/var/lib/dweb-cloud
ENV DWEB_CLOUD_APP_CONFIG=/app/config/apps.json

EXPOSE 9080
VOLUME ["/var/lib/dweb-cloud"]
CMD ["/usr/local/bin/dweb-cloud-server"]
