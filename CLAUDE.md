# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bits is a decentralized, end-to-end encrypted content marketplace built in Rust. It enables creators to monetize content directly through P2P networking without intermediaries, censorship, or centralized control.

## Architecture

### Core Components

- **P2P Node** (`/node`) - Rust implementation using libp2p for networking
- **Smart Contracts** (`/contracts`) - Ethereum L2 contracts for payments and governance
- **Desktop App** (`/desktop`) - Tauri-based native application (planned)
- **Web Client** (`/web`) - WASM-based web interface (planned)

### Technology Stack

- **Language**: Rust (memory safety, performance, single binary deployment)
- **Networking**: libp2p (Kademlia DHT, Gossipsub, mDNS discovery)
- **Storage**: SQLite (embedded, zero-config) + content-addressed P2P storage
- **Encryption**: ChaCha20-Poly1305 (content), Ed25519 (signatures)
- **Blockchain**: Ethereum L2 for payments and identity
- **Development**: Nix/devenv for reproducible environments

## Development Commands

### Core Commands (in `/bin`)

```bash
bin/setup    # Initial environment setup and database initialization
bin/test     # Run tests (prefers cargo-nextest)
bin/build    # Build release binaries
bin/fmt      # Format code with rustfmt
bin/lint     # Lint with clippy
bin/check    # Type checking
bin/clean    # Clean build artifacts
```

### Development Workflow

```bash
devenv shell                    # Enter Nix development environment
devenv up                       # Start node with auto-reload (cargo-watch)
cargo run --bin bits -- --dev   # Run node directly in dev mode
```

### Testing

```bash
cargo test                      # Run all tests
cargo nextest run              # Better test output (if available)
cargo test -p node             # Test specific package
```

## High-Level Architecture

### P2P Network Design

The node implements a distributed hash table (DHT) for content discovery without central servers:

1. **Discovery** (`node/src/p2p/discovery.rs`) - mDNS for local peers, Kademlia for global
2. **Routing** (`node/src/p2p/routing.rs`) - Content-addressed routing
3. **Transport** (`node/src/p2p/transport.rs`) - Encrypted P2P connections

### Storage Architecture

- Content is chunked, encrypted, and distributed across the network
- Nodes earn tokens for storing and serving content
- Automatic replication based on demand
- SQLite for local node metadata and indexes

### Economic Model

Revenue distribution enforced by smart contracts:
- 95% to content creators
- 3% to protocol developers
- 2% to node operators

### API Design

The node exposes a local API (port 8080) for client applications:
- `/api/content` - Upload/download encrypted content
- `/api/peers` - P2P network status
- `/api/identity` - DID management

## Key Development Principles

1. **Decentralization First** - No AWS, no Stripe, no central servers
2. **Privacy by Design** - E2E encryption before any network transmission
3. **Economic Incentives** - Everyone in the network earns
4. **Single Binary** - Easy deployment for node operators
5. **Progressive Enhancement** - Start simple, add features incrementally

## Testing Approach

- Unit tests for cryptographic operations
- Integration tests for P2P networking
- Property-based testing for distributed algorithms
- Testnet deployment before mainnet

## Migration Status

The project has pivoted from a centralized TypeScript/React app to a fully decentralized Rust implementation. Any references to AWS, Stripe, PostgreSQL, or npm packages are outdated and should be removed.

## Common Tasks

### Running a Local Network

```bash
# Terminal 1 - Bootstrap node
cargo run --bin bits -- --dev --port 9000

# Terminal 2 - Peer node
cargo run --bin bits -- --dev --port 9001 --bootstrap /ip4/127.0.0.1/tcp/9000
```

### Database Operations

```bash
# Initialize new database
sqlite3 data/node.db < node/migrations/20250116_initial.sql

# View node data
sqlite3 data/node.db "SELECT * FROM peers;"
```

### Smart Contract Development

```bash
cd contracts
forge test          # Run contract tests
forge build         # Compile contracts
```

## Important Notes

- Focus on P2P and decentralization - this is NOT a web2 app with blockchain payments
- The goal is to empower individuals and redistribute control of digital content
- Node operators are first-class citizens, not just users
- Every design decision should support censorship resistance and user sovereignty

## Development Practices

### Git Workflow
- **Commit frequently** - Make atomic commits with clear messages
- **Test before commit** - Run `bin/test` before committing changes
- **Lint your code** - Run `bin/lint` and `bin/fmt` before commits
- **Verify it runs** - Ensure `cargo run --bin bits -- --dev` works before committing
- **Clean working tree** - Don't leave unstaged changes; commit or stash regularly

### Pre-commit Checklist
```bash
bin/fmt      # Format code
bin/lint     # Run clippy
bin/test     # Run tests
cargo build  # Ensure it compiles
cargo run --bin bits -- --dev  # Verify it runs
git add -p   # Stage changes selectively
git commit   # Commit with descriptive message
```