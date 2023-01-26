FROM rust:1.66.1 as builder

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --target x86_64-unknown-linux-musl --release && rm src/main.rs

COPY src ./

RUN cargo build --release --target x86_64-unknown-linux-musl

CMD ["./target/x86_64-unknown-linux-musl/release/golinks"]

FROM alpine:latest as debug

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/golinks /golinks

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

CMD ["/golinks"]


FROM scratch as runner

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/golinks /golinks

ENV ROCKET_ADDRESS 0.0.0.0
ENV ROCKET_PORT 8000
ENV ROCKET_IDENT false

CMD ["/golinks"]