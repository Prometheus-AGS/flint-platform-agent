-- Smoke-only role bootstrap for the REAL forge Postgres.
--
-- forge's SQL (ext-flint-auth/sql/flint_auth.sql, ext-flint-meta/sql/flint_meta.sql)
-- GRANTs privileges to the Supabase-convention roles `authenticated`, `anon`,
-- `service_role`, and `vault_admin` — but forge ships NO `CREATE ROLE` for them (they
-- are assumed pre-seeded by the platform Postgres image, Supabase-style). forge's CI
-- image (docker/postgres/Dockerfile) is a bare `postgres:18` that does not seed them,
-- so the migrations' GRANTs would fail. This file seeds them idempotently.
--
-- This is the SMOKE's provisioning, authored HERE (never in ../flint-forge). forge
-- still owns wiring a proper seed into its own CheckDb — see the parked follow-up in
-- forge's p35-c003-ci-postgres-service/tasks.md ("[~] flint_meta bootstrap ...").
--
-- Applied by smoke/forge-bootstrap.sh BEFORE the flint_meta schema + migrations.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'service_role') THEN
        CREATE ROLE service_role NOLOGIN BYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'vault_admin') THEN
        CREATE ROLE vault_admin NOLOGIN;
    END IF;
END
$$;
