FROM rust:1.81-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release && cp target/release/minimamemosa /minimamemosa

FROM alpine:3.20
RUN apk add --no-cache ca-certificates sqlite-libs
RUN adduser -D -h /app minimamemosa
USER minimamemosa
WORKDIR /app
COPY --from=builder /minimamemosa .
EXPOSE 3000
ENV DATABASE_PATH=/app/data/minimamemosa.db
CMD ["./minimamemosa"]
