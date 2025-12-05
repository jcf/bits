-- Track authentication attempts for rate limiting
create table auth_attempts (
  id bigserial primary key,
  email text not null,
  endpoint text not null,
  ip_address inet,
  created_at timestamptz not null default now()
);

-- Index for email-based rate limiting queries
create index idx_auth_attempts_email
on auth_attempts(email, created_at desc);

-- Index for cleanup queries
create index idx_auth_attempts_created_at
on auth_attempts(created_at);
