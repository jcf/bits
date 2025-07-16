# Bits - Decentralized E2EE Content Marketplace

A truly decentralized, end-to-end encrypted content marketplace built on P2P technology and blockchain incentives.

## Vision

Bits democratizes content distribution by enabling creators to monetize their work directly without intermediaries. Using peer-to-peer networking, cryptographic security, and blockchain-based payments, Bits creates a censorship-resistant platform where creators retain full control.

## Architecture

- **P2P Network**: Built with libp2p for decentralized content discovery and distribution
- **Storage**: Content-addressed storage with automatic replication
- **Encryption**: End-to-end encryption using ChaCha20-Poly1305
- **Payments**: Blockchain-based payments with automatic revenue distribution
- **Identity**: Decentralized identities (DIDs) for creators and consumers

## Getting Started

### Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- [devenv](https://devenv.sh/)

### Development

```bash
# Enter development environment
devenv shell

# Run the node
cargo run --bin bits -- --dev

# Run with auto-reload
devenv up
```

### Building

```bash
# Build optimized binary
cargo build --release

# Output: target/release/bits
```

## Project Structure

- `node/` - P2P node implementation
- `contracts/` - Smart contracts for payments and governance
- `desktop/` - Tauri desktop application (planned)
- `web/` - WASM web client (planned)
- `mobile/` - React Native mobile app (planned)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

Dual-licensed under MIT and Apache 2.0.