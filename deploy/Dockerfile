# Build stage
FROM nixos/nix:latest AS builder

WORKDIR /build

# Copy source
COPY . .

# Build release binary
RUN nix develop --command cargo build --release --package bits-colo

# Bundle Dioxus assets
RUN nix develop --command dx bundle --release --package bits-colo

# Runtime stage
FROM debian:bookworm-slim

# Install minimal dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 bits

# Copy binary and assets
COPY --from=builder /build/target/release/bits-colo /usr/local/bin/
COPY --from=builder /build/target/dx/bits-colo/release/public /app/public

USER bits
WORKDIR /app

# Expose port
EXPOSE 8080

# Run server
CMD ["bits-colo"]
