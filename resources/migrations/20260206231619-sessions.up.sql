CREATE TABLE sessions (
    sid_hash    TEXT NOT NULL,
    tenant_id   UUID NOT NULL,
    user_id     UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '30 days',
    data        JSONB NOT NULL DEFAULT '{}',
    PRIMARY KEY (sid_hash, tenant_id)
);

COMMENT ON COLUMN sessions.sid_hash IS 'SHA-256 hash of session ID (hex encoded)';
COMMENT ON COLUMN sessions.tenant_id IS 'Tenant UUID from Datomic';
COMMENT ON COLUMN sessions.user_id IS 'References user entity in Datomic';

CREATE INDEX sessions_tenant_id_idx ON sessions(tenant_id);
CREATE INDEX sessions_user_id_idx ON sessions(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX sessions_expires_at_idx ON sessions(expires_at);
