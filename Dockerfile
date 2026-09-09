FROM rust:1.98.1-alpine3.24 AS builder

WORKDIR /build
RUN apk add --no-cache musl-dev

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm src/main.rs

COPY src src/
RUN cargo build --release

FROM alpine:3.24 AS alpine

WORKDIR /app
COPY --from=builder /build/target/release/firebird_exporter /app/firebird_exporter

ENTRYPOINT ["/app/firebird_exporter"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD /app/firebird_exporter healthcheck

FROM scratch AS minimal

COPY --from=builder /build/target/release/firebird_exporter /firebird_exporter

ENTRYPOINT ["/firebird_exporter"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD ["/firebird_exporter","healthcheck"]