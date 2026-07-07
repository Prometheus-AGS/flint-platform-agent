## 1. Generate the key pair

- [ ] 1.1 Generate a 2048-bit RSA key pair (no passphrase):
  `openssl genrsa -out smoke/dev-idp/private-key.pem 2048`
  `openssl rsa -in smoke/dev-idp/private-key.pem -pubout -out smoke/dev-idp/public-key.pem`
- [ ] 1.2 Convert the public key to JWKS format (`smoke/dev-idp/jwks.json`). The JWKS must
  include `kid` (a stable string, e.g. `"dev-smoke-key-1"`), `kty: "RSA"`, `use: "sig"`,
  `alg: "RS256"`, and the RSA modulus/exponent (`n`, `e`) in Base64url. A Python one-liner or
  the `jwkconv` / `python-jose` CLI can do this; or compute it manually with `openssl`.

## 2. Document and protect

- [ ] 2.1 `smoke/dev-idp/README.md`: "These are THROWAWAY smoke-only keys. The private key
  is intentionally committed — it only authenticates against this smoke's dev IdP, not any
  real system. Treat them like the HS256 secret in the composes."
- [ ] 2.2 Confirm `smoke/dev-idp/` is NOT in `.gitignore` (it must be committed).

## 3. Verification

- [ ] 3.1 `jq . smoke/dev-idp/jwks.json` — valid JSON with `keys[0].kid`, `n`, `e`, `kty=RSA`, `alg=RS256`.
- [ ] 3.2 Mint a test RS256 JWT with `node -e "..."` (using `jsonwebtoken` + the PEM private key)
  and verify it decodes against the JWKS — confirms the key pair is consistent before
  wiring the compose.
