create table email_verification_codes (
  id bigserial primary key,
  email_address_id bigint not null references email_addresses(id) on delete cascade,
  code_hash text not null,
  created_at timestamptz not null default now(),
  expires_at timestamptz not null,
  last_sent_at timestamptz not null default now(),
  send_count int not null default 1,
  attempt_count int not null default 0,
  verified_at timestamptz
);

-- Only one active unverified code per email address
create unique index idx_verification_codes_active
on email_verification_codes(email_address_id)
where verified_at is null;

create index idx_verification_codes_lookup
on email_verification_codes(email_address_id, code_hash)
where verified_at is null;

-- Track resend attempts for rate limiting
create table email_verification_resend_log (
  id bigserial primary key,
  email_address_id bigint not null,
  ip_address inet,
  created_at timestamptz not null default now()
);

create index idx_resend_log_email
on email_verification_resend_log(email_address_id, created_at desc);

create index idx_resend_log_ip
on email_verification_resend_log(ip_address, created_at desc);
