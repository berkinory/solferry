FROM rust:1-alpine AS build
RUN apk add --no-cache musl-dev
RUN cargo install iroh-relay --version 1.4.1 --locked --features server --root /out

FROM alpine:3.22
RUN apk add --no-cache ca-certificates
COPY --from=build /out/bin/iroh-relay /usr/local/bin/iroh-relay
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
COPY token.sh /usr/local/bin/relay-token
RUN mkdir /data && chown 65534 /data
USER 65534
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
