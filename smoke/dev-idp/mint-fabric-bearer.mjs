// Mint the long-lived RS256 bearer the agent forwards to fabric (FPA_FABRIC_BEARER).
//
// The /fabric/subscribe bridge forwards this token to the fabric gateway, whose
// OryIdentityVerifier verifies it (RS256) against the dev JWKS server. Fabric then
// authorizes the subscribe against Keto using this token's `sub` — so `sub` MUST
// equal the subject the smoke seeds Keto tuples for (SUBJECT in smoke.real.spec.ts).
//
// Long-lived + fixed-subject on purpose: a compose env var can't hold a per-run
// minted JWT, so this is a stable throwaway smoke credential signed by the
// committed dev key (smoke/dev-idp/private-key.pem — see README.md). NOT a real
// credential. Prints the token to stdout; run-real.sh captures it into the env.
//
//   node smoke/dev-idp/mint-fabric-bearer.mjs
//
// Keep `SUB` in sync with SUBJECT in smoke.real.spec.ts and the Keto seeds there.

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const jwt = require("jsonwebtoken");

const here = dirname(fileURLToPath(import.meta.url));
const privateKey = readFileSync(join(here, "private-key.pem"), "utf8");

// Must match SUBJECT + FABRIC_TENANT_ID in smoke.real.spec.ts.
const SUB = "smoke-realtime-user";
const TENANT_ID = "00000000-0000-0000-0000-000000000001";

const token = jwt.sign(
  { sub: SUB, tenant_id: TENANT_ID, aud: "frf-gateway", jti: "smoke-fabric-bearer-static" },
  privateKey,
  { algorithm: "RS256", keyid: "dev-smoke-key-1", expiresIn: "3650d" },
);

process.stdout.write(token);
