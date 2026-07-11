FROM rust:1.96-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release && cp target/release/minimamemosa /minimamemosa

FROM alpine:3.20
RUN apk add --no-cache ca-certificates sqlite-libs su-exec
RUN adduser -D -h /app minimamemosa
RUN mkdir -p /app/data && chown minimamemosa:minimamemosa /app/data

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh
WORKDIR /app
COPY --from=builder /minimamemosa .
EXPOSE 3000
ENV DATABASE_PATH=/app/data/minimamemosa.db
ENTRYPOINT ["/docker-entrypoint.sh"]
CMD ["./minimamemosa"]
