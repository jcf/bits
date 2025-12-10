-- Add is_fallback column to tenants table
-- This allows marking a single tenant as the fallback for unknown domains
-- Solo mode: One tenant marked as fallback (all unknown domains route there)
-- Colo mode: No fallback tenant (unknown domains return 404)

alter table tenants
  add column is_fallback boolean not null default false;

-- Ensure only one tenant can be marked as fallback
-- Partial unique index allows multiple false but only one true
create unique index idx_tenants_is_fallback
  on tenants(is_fallback)
  where is_fallback = true;

-- Add index for fast fallback tenant lookup
create index idx_tenants_fallback_lookup
  on tenants(id)
  where is_fallback = true;
