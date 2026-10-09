# SPDX-License-Identifier: MPL-2.0
# Multi-stage build for echidnabot using Chainguard Wolfi
#
# Build stage: Use Chainguard Rust image for reproducible builds
# Runtime stage: Use minimal wolfi-base image for production deployment
#
# Build with Podman:
#   podman build -t ghcr.io/hyperpolymath/echidnabot:local -f Containerfile .
#
# Run with Podman:
#   podman run -p 8080:8080 --rm ghcr.io/hyperpolymath/echidnabot:local serve
#
# NOTE: Before deploying to production, pin the image digests to full
# SHA256 hashes (estate convention). Use:
#   podman pull cgr.dev/chainguard/rust:1.97.0
#   podman pull cgr.dev/chainguard/wolfi-base:latest
#   podman inspect --format '{{.Digest}}' <image-id>

# Build stage: use Chainguard Rust image for reproducible builds
FROM cgr.dev/chainguard/rust:1.97.0

WORKDIR /build

# Install build dependencies (for sqlx)
RUN apk add --no-cache pkgconf sqlite-dev openssl-dev

# Copy source code
COPY . .

# Build release binary
# Note: echidna-core git dependency is resolved by Cargo from GitHub
RUN --mount=type=cache,target=/root/.cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    cargo build --release

# Runtime stage: minimal Wolfi base
FROM cgr.dev/chainguard/wolfi-base:latest

LABEL org.opencontainers.image.source="https://github.com/hyperpolymath/echidnabot"
LABEL org.opencontainers.image.description="Proof-aware CI bot that orchestrates ECHIDNA for theorem proof verification"
LABEL org.opencontainers.image.licenses="MPL-2.0"
LABEL org.opencontainers.image.version="0.1.0"

# Install runtime dependencies (SQLite, CA certificates)
RUN apk add --no-cache sqlite-libs ca-certificates

# Copy binary from builder
COPY --from=0 /build/target/release/echidnabot /usr/local/bin/echidnabot

# Create non-root user for security
RUN adduser -D -u 1000 echidna
USER echidna

WORKDIR /home/echidna

# Expose default echidnabot port
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/echidnabot"]
CMD ["serve"]
