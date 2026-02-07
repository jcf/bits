CREATE TABLE sessions (
    sid         TEXT PRIMARY KEY,
    user_id     UUID,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '30 days',
    data        JSONB NOT NULL DEFAULT '{}'
);

COMMENT ON COLUMN sessions.user_id IS 'References user entity in Datahike';

CREATE INDEX sessions_user_id_idx ON sessions(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX sessions_expires_at_idx ON sessions(expires_at);
