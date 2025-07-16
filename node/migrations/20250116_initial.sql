-- Initial schema for Bits node

-- Chunks table for storing encrypted content pieces
CREATE TABLE IF NOT EXISTS chunks (
    key BLOB PRIMARY KEY,
    data BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    access_count INTEGER DEFAULT 0
);

-- Content metadata (minimal on-node storage)
CREATE TABLE IF NOT EXISTS content (
    cid TEXT PRIMARY KEY,
    creator TEXT NOT NULL,
    size INTEGER NOT NULL,
    price INTEGER NOT NULL,
    encrypted_key BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_accessed TIMESTAMP
);

-- Peer information for network health
CREATE TABLE IF NOT EXISTS peers (
    peer_id TEXT PRIMARY KEY,
    multiaddr TEXT NOT NULL,
    last_seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    reputation INTEGER DEFAULT 0
);

-- Local node configuration
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_chunks_created_at ON chunks(created_at);
CREATE INDEX idx_content_creator ON content(creator);
CREATE INDEX idx_content_price ON content(price);
CREATE INDEX idx_peers_last_seen ON peers(last_seen);