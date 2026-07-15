# Stage 1: Build the binary
FROM rust:1.97.0-trixie AS builder

RUN apt-get update && apt-get install -y \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY config/ config/
COPY templates/ templates/

RUN rustup target add x86_64-unknown-linux-musl && \
    cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Minimal runtime
FROM alpine:3.24.1

RUN apk add --no-cache libssl3 libgcc

RUN addgroup -S app && adduser -S app -G app
USER app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/gerrit-faster /usr/local/bin/gerrit-faster

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/bot/health || exit 1

ENTRYPOINT ["/usr/local/bin/gerrit-faster"]
