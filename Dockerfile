# Build
FROM rust:1.73-alpine as builder

RUN apk update && apk add pkgconfig libressl-dev musl-dev

RUN rustup target add aarch64-unknown-linux-musl

WORKDIR "/build"

COPY Cargo.lock .
COPY Cargo.toml .

RUN mkdir src \ 
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release --target aarch64-unknown-linux-musl

COPY src/ src/

RUN touch src/main.rs && cargo build --release --target aarch64-unknown-linux-musl

# Run
FROM alpine as runtime

WORKDIR /app

COPY --from=builder "/build/target/aarch64-unknown-linux-musl/release/che-guarde-bot" .

ENTRYPOINT ["./che-guarde-bot"]
