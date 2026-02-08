CREATE TABLE authentication_attempts (
    id          BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email       TEXT NOT NULL,
    ip_address  INET NOT NULL,
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    success     BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX authentication_attempts_email_idx
    ON authentication_attempts (email, attempted_at DESC)
    WHERE NOT success;

CREATE INDEX authentication_attempts_ip_idx
    ON authentication_attempts (ip_address, attempted_at DESC)
    WHERE NOT success;

CREATE INDEX authentication_attempts_cleanup_idx
    ON authentication_attempts (attempted_at);

COMMENT ON TABLE authentication_attempts IS 'Rate limiting for authentication — lives in PostgreSQL to avoid Datahike history accumulation';
