CREATE TABLE authentication_attempts (
    id           BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    tenant_id    UUID NOT NULL,
    email        TEXT NOT NULL,
    ip_hash      TEXT NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    success      BOOLEAN NOT NULL DEFAULT false
);

COMMENT ON TABLE authentication_attempts IS 'Rate limiting for authentication';
COMMENT ON COLUMN authentication_attempts.tenant_id IS 'Tenant UUID from Datomic';
COMMENT ON COLUMN authentication_attempts.ip_hash IS 'SHA-256 hash of IP address (hex encoded)';

CREATE INDEX authentication_attempts_email_idx
    ON authentication_attempts (tenant_id, email, attempted_at DESC)
    WHERE NOT success;

CREATE INDEX authentication_attempts_ip_idx
    ON authentication_attempts (tenant_id, ip_hash, attempted_at DESC)
    WHERE NOT success;

CREATE INDEX authentication_attempts_cleanup_idx
    ON authentication_attempts (attempted_at);
