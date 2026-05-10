# PAI-Kernel governance daemon — Docker image
#
# Build locally:
#   docker build -t pai-kernel:local .
#
# Pull from GitHub Container Registry:
#   docker pull ghcr.io/pai-kernel/pai-kernel:v2.2.1
#
# Run:
#   docker run --rm -p 9100:9100 ghcr.io/pai-kernel/pai-kernel:v2.2.1
#
# Daemon listens on 127.0.0.1:9100 inside the container; published via -p to host.
# For external access (e.g. curl from host): daemon binds to 0.0.0.0 inside container.
#
# Persistent storage (SQLite witness DB):
#   docker run -v pai-kernel-data:/data -p 9100:9100 \
#     ghcr.io/pai-kernel/pai-kernel:v2.2.1
#
# The SQLite DB + any runtime state lives under /data inside the container.

# ---- stage 1: build ----
# Pin к sha256 для Pinned-Dependencies OpenSSF Scorecard hardening (PF-057)
FROM rust:1.86-alpine@sha256:661d708cc863ce32007cf46807a72062a80d2944a6fae9e0d83742d2e04d5375 AS builder

# git is required by some transitive build scripts (regorus Rego engine
# invokes `git rev-parse HEAD` during build). ca-certificates for TLS on
# cargo fetch.
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static sqlite-dev git ca-certificates

WORKDIR /build
COPY . .

# Build the release binary with statically linked dependencies where possible
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release --locked -p pai_kernel && \
    strip target/release/pai_governance_daemon

# ---- stage 2: runtime ----
# Pin к sha256 для Pinned-Dependencies OpenSSF Scorecard hardening (PF-057)
FROM alpine:3.20@sha256:d9e853e87e55526f6b2917df91a2115c36dd7c696a35be12163d44e6e2a4b6bc

# Install runtime dependencies only
RUN apk add --no-cache ca-certificates sqlite-libs libgcc

# Create non-root user for daemon
RUN addgroup -S pai && adduser -S -G pai pai

# Directories for config + policies + persistent data
RUN mkdir -p /app /data /app/policies && chown -R pai:pai /app /data

# Copy binary + default config + policies from builder
COPY --from=builder /build/target/release/pai_governance_daemon /app/pai_governance_daemon
COPY --from=builder /build/pai-kernel.toml /app/pai-kernel.toml
COPY --from=builder /build/policies /app/policies
COPY --from=builder /build/LICENSE /app/LICENSE
COPY --from=builder /build/README.md /app/README.md
# Framework corpus (v2.2 canonical documents)
COPY --from=builder /build/corpus/PAI_Constitutional_Document.md /app/PAI_Constitutional_Document.md
COPY --from=builder /build/corpus/PAI_Bill_of_Authorial_Rights.md /app/PAI_Bill_of_Authorial_Rights.md
COPY --from=builder /build/corpus/Glossary.md /app/Glossary.md

RUN chown -R pai:pai /app

USER pai
WORKDIR /data

EXPOSE 9100

# Default: run daemon with /app/pai-kernel.toml, binding to 0.0.0.0:9100 inside container
# (Host maps to 127.0.0.1:9100 via `docker run -p 9100:9100`)
# Override config by mounting your own: -v $PWD/my-config.toml:/app/pai-kernel.toml
ENTRYPOINT ["/app/pai_governance_daemon"]
CMD ["--config", "/app/pai-kernel.toml"]

# Image metadata
LABEL org.opencontainers.image.title="PAI-Kernel Governance Daemon"
LABEL org.opencontainers.image.description="Constitutional governance runtime for AI systems (PAI-CD framework)"
LABEL org.opencontainers.image.source="https://github.com/PAI-Kernel/pai-kernel"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.documentation="https://github.com/PAI-Kernel/pai-kernel/blob/main/docs/INSTALL.md"
