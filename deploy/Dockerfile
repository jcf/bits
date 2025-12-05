# ------------------------------------------------------------------------------
# Build
FROM nixos/nix:latest AS builder
WORKDIR /build
COPY . .
RUN nix develop --command cargo build --release --package bits-colo
RUN nix develop --command dx bundle --release --package bits-colo

# ------------------------------------------------------------------------------
# Run

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /build/target/release/bits-colo /bits
COPY --from=builder /build/target/dx/bits-colo/release/public /public

EXPOSE 8080
ENTRYPOINT ["/bits"]
