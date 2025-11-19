-- Create tenants table in public schema
CREATE TABLE public.tenants (
    name TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed initial tenant
INSERT INTO public.tenants (name) VALUES ('jcf');
