create table public.tenants (
  id bigserial primary key,
  name text not null,
  created_at timestamptz not null default now()
);

create table tenant_domains (
  id bigserial primary key,
  tenant_id bigint not null references tenants(id),
  domain text not null,
  valid_from timestamptz not null default now(),
  valid_to timestamptz not null default 'infinity',
  added_by bigint not null,
  removed_by bigint,

  -- Prevent tenants from sharing domains.
  exclude using gist (
    domain with =,
    tstzrange(valid_from, valid_to) with &&
  )
);

create index idx_tenant_domains_current
on tenant_domains(domain)
where valid_to = 'infinity';

create index idx_tenant_active_domains
on tenant_domains(domain)
where valid_to = 'infinity';

create index idx_tenant_domains_history
on tenant_domains
using gist (domain, tstzrange(valid_from, valid_to));
