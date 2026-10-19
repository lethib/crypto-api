# Build stage: compile a release binary.
FROM rust:1.96-slim-trixie AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

# Runtime stage: distroless image with glibc, no shell, runs as a non-root user.
# Same Debian release as the builder so the glibc versions match.
FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=builder /app/target/release/crypto_api /usr/local/bin/crypto_api
# The default HOST (127.0.0.1) would be unreachable from outside the container.
ENV HOST=0.0.0.0
ENV PORT=3000
EXPOSE 3000
# HMAC_SECRET is not set here: pass it at runtime with `docker run -e HMAC_SECRET=...`.
ENTRYPOINT ["/usr/local/bin/crypto_api"]
