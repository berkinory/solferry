FROM rust:1-slim AS build
RUN cargo install iroh-relay --version 1.4.1 --locked --features server --root /out

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /out/bin/iroh-relay /usr/local/bin/iroh-relay
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
USER 65534
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
