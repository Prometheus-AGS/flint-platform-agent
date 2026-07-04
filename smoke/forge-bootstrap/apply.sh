#!/usr/bin/env sh
# Real-forge DB pre-seed for the smoke — the `forge-bootstrap` one-shot.
#
# Seeds ONLY the prerequisites that forge's migrations ASSUME already exist:
#   1. roles (authenticated/anon/service_role/vault_admin) — forge GRANTs to them but
#      never CREATE ROLEs them (00-roles.sql).
#   2. the flint_meta schema — migration 0002 declares "Depends on: flint_meta schema"
#      (01-flint-meta.sql, vendored pure SQL).
#
# It does NOT run forge's migrations. The gateway (fdb-gateway) runs them itself at
# boot via `sqlx::migrate!("../../migrations")` (embedded at compile time, idempotent,
# tracked in `_sqlx_migrations`). Running them here too would leave the objects present
# but UN-tracked, so the gateway's migrator then fails with "relation ... already
# exists". Pre-seed prerequisites only; let the gateway own migrations.
#
# Runs inside a postgres:18 client image (has psql). Runs to completion, then exits 0.
#
# Env:
#   DATABASE_URL   required — points at forge-postgres (superuser).
#   BOOTSTRAP_DIR  default /bootstrap  (smoke/forge-bootstrap mounted here)
set -eu

DATABASE_URL="${DATABASE_URL:?DATABASE_URL must be set}"
BOOTSTRAP_DIR="${BOOTSTRAP_DIR:-/bootstrap}"

echo "==> forge pre-seed: roles + flint_meta from ${BOOTSTRAP_DIR}"
for f in "${BOOTSTRAP_DIR}"/[0-9]*.sql; do
    [ -e "$f" ] || continue
    echo "    apply $(basename "$f")"
    psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -q -f "$f"
done

echo "OK: forge prerequisites seeded (roles + flint_meta) — gateway runs migrations"
