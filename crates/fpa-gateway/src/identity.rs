//! Gate-derived operator identity.
//!
//! flint-gate is the **only** auth boundary. This module consumes gate-injected
//! credentials (a gate-minted JWT, or gate identity headers) and derives the
//! operator's roles/permissions from the claims. It does **not** call Ory
//! (Kratos/Hydra/Keto/Oathkeeper) and does **not** fetch Ory JWKS.
//!
//! **Position-dependent verification (p4-c002):**
//! 1. **Trust path** — when a request carries the operator-configured trusted
//!    identity headers (gate injected them, so it came through gate), identity is
//!    built from those headers without verifying a token.
//! 2. **Verify path** — a directly-received bearer is signature-verified against
//!    the IdP JWKS (RS256/ES256) or the HS256 shared secret.
//! 3. **Reject** — a raw token with no usable verification key is never decoded
//!    unverified. There is no insecure fallback.

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

/// Header carrying the gate-minted bearer token.
const HDR_AUTHORIZATION: &str = "authorization";

/// Claims this agent reads from a gate-minted JWT. Unknown claims are ignored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateClaims {
    /// Subject — the operator identifier.
    pub sub: String,
    /// Roles granted by gate (derived from Keto/Kratos upstream).
    #[serde(default)]
    pub roles: Vec<String>,
    /// Tenant scope, when present.
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Expiry (epoch seconds) — REQUIRED; verification always validates it. Kept
    /// non-optional so the type matches the enforced requirement (review H5).
    pub exp: usize,
}

/// The authenticated operator for a request, derived solely from gate claims.
///
/// Retains the **raw bearer** so it can be forwarded to an RLS-enforcing
/// downstream (forge) via `p2-c001` credential threading. The token is never
/// logged.
#[derive(Clone)]
pub struct OperatorContext {
    pub subject: String,
    pub roles: Vec<String>,
    #[allow(dead_code)] // reserved for tenant-scoped dispatch
    pub tenant_id: Option<String>,
    /// True only when the JWT signature was verified against a configured key.
    #[allow(dead_code)] // surfaced in audit records
    pub signature_verified: bool,
    /// The raw gate-minted bearer, forwarded downstream for RLS. Never logged.
    pub bearer: String,
}

impl OperatorContext {
    /// The forwardable bearer, or `None` on the trust path (no token to forward).
    /// Never forward `Some("")` downstream (an empty credential is not `None`).
    #[must_use]
    pub fn forwardable_bearer(&self) -> Option<String> {
        if self.bearer.is_empty() {
            None
        } else {
            Some(self.bearer.clone())
        }
    }
}

// Manual Debug that redacts the bearer — deriving Debug would print the token.
impl std::fmt::Debug for OperatorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperatorContext")
            .field("subject", &self.subject)
            .field("roles", &self.roles)
            .field("tenant_id", &self.tenant_id)
            .field("signature_verified", &self.signature_verified)
            .field("bearer", &"<redacted>")
            .finish()
    }
}

/// Rejection when no usable gate identity is present.
///
/// Absence of credentials is **unauthenticated** — never an implicit grant.
#[derive(Debug)]
pub struct NoIdentity;

impl axum::response::IntoResponse for NoIdentity {
    fn into_response(self) -> axum::response::Response {
        // Do not echo any token material in the response.
        (StatusCode::UNAUTHORIZED, "missing or invalid gate identity").into_response()
    }
}

impl FromRequestParts<Arc<AppState>> for OperatorContext {
    type Rejection = NoIdentity;

    #[tracing::instrument(skip_all, name = "gate_identity")]
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 1. TRUST PATH — request came through gate (all configured trusted
        //    headers present). Build identity from them; do not verify a token.
        if let Some(ctx) = trusted_from_headers(parts, &state.config.trusted_identity_headers) {
            tracing::debug!(path = "trusted", "gate-injected identity");
            return Ok(ctx);
        }

        // 2. VERIFY PATH — a directly-received bearer must be signature-verified.
        let token = bearer_token(parts).ok_or(NoIdentity)?;
        let claims = verify_token(&token, state).await.ok_or(NoIdentity)?;
        tracing::debug!(
            path = "verified",
            has_subject = !claims.sub.is_empty(),
            "identity verified"
        );
        Ok(Self {
            subject: claims.sub,
            roles: claims.roles,
            tenant_id: claims.tenant_id,
            signature_verified: true,
            bearer: token,
        })
        // 3. Any other case fell through to NoIdentity above — a raw token that
        //    cannot be verified is NEVER decoded unverified.
    }
}

/// Build a trusted [`OperatorContext`] from gate-injected headers, iff the
/// operator configured a non-empty trusted-header set and ALL of them are present.
/// Returns `None` (fall through to verification) otherwise.
fn trusted_from_headers(parts: &Parts, trusted: &[String]) -> Option<OperatorContext> {
    if trusted.is_empty() {
        return None;
    }
    // Every configured trusted header must be present.
    let get = |name: &str| -> Option<String> {
        parts
            .headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    if !trusted.iter().all(|h| get(h).is_some()) {
        return None;
    }
    // Conventional gate identity headers → subject / roles / tenant.
    let subject = get("x-user-id").unwrap_or_default();
    let roles = get("x-user-roles")
        .map(|r| r.split(',').map(|s| s.trim().to_owned()).collect())
        .unwrap_or_default();
    let tenant_id = get("x-org-id").or_else(|| get("x-tenant-id"));
    Some(OperatorContext {
        subject,
        roles,
        tenant_id,
        signature_verified: false, // trusted via gate, not by us verifying a signature
        bearer: String::new(),     // no forwardable bearer on the header path
    })
}

/// Verify a directly-received token: JWKS first (if configured), then the HS256
/// shared secret (if configured). No usable key ⇒ `None` (reject) — never an
/// unverified decode.
async fn verify_token(token: &str, state: &Arc<AppState>) -> Option<GateClaims> {
    if let Some(verifier) = state.jwks.as_ref()
        && let Ok(claims) = verifier.verify::<GateClaims>(token).await
    {
        return Some(claims);
    }
    if let Some(secret) = state.config.gate_jwt_key.as_deref()
        && let Ok((claims, _)) = verify_hs256(token, secret)
    {
        return Some(claims);
    }
    None
}

/// Optional extraction: absent/invalid gate identity yields `None` rather than a
/// rejection, so unauthenticated handshake endpoints (MCP `initialize`,
/// `tools/list`) still work while `tools/call` can require `Some`.
impl OptionalFromRequestParts<Arc<AppState>> for OperatorContext {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(
            <Self as FromRequestParts<Arc<AppState>>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}

/// Extract the bearer token from the `Authorization` header.
fn bearer_token(parts: &Parts) -> Option<String> {
    let raw = parts.headers.get(HDR_AUTHORIZATION)?.to_str().ok()?;
    raw.strip_prefix("Bearer ")
        .or_else(|| raw.strip_prefix("bearer "))
        .map(str::to_owned)
}

/// Decode a gate-minted JWT. Returns the claims and whether the signature was
/// verified. With a configured key, the signature + expiry are validated; without
/// one, claims are decoded unverified (interim stopgap, documented above).
fn verify_hs256(
    token: &str,
    secret: &str,
) -> Result<(GateClaims, bool), jsonwebtoken::errors::Error> {
    // Signature-verify with the shared secret + validate expiry. The algorithm is
    // SERVER-FIXED to HS256 — never derived from the token header (prevents
    // algorithm confusion, e.g. an RS256 token downgraded to HS256). The old
    // "decode unverified when no key" branch is retired (p4-c002).
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.validate_aud = false;
    let data = decode::<GateClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;
    Ok((data.claims, true))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};

    fn mint(secret: &str, roles: &[&str]) -> String {
        let claims = GateClaims {
            sub: "op-123".into(),
            roles: roles.iter().map(|s| (*s).to_owned()).collect(),
            tenant_id: Some("tenant-a".into()),
            exp: 4_102_444_800, // year 2100
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("mint")
    }

    #[test]
    fn hs256_verifies_with_configured_key() {
        let token = mint("shared-key", &["admin", "operator"]);
        let (claims, verified) = verify_hs256(&token, "shared-key").expect("verify");
        assert!(verified);
        assert_eq!(claims.sub, "op-123");
        assert_eq!(claims.roles, vec!["admin", "operator"]);
    }

    #[test]
    fn hs256_rejects_bad_signature() {
        let token = mint("shared-key", &["admin"]);
        assert!(
            verify_hs256(&token, "wrong-key").is_err(),
            "wrong key must fail signature verification"
        );
    }

    // SECURITY (p4-c002): there is no longer any unverified-decode path. A token
    // is only accepted via a verifying function (verify_hs256 / JWKS). This test
    // documents that verify_hs256 hard-fails on a wrong key — no silent accept.
    #[test]
    fn no_unverified_accept() {
        let token = mint("real-secret", &["operator"]);
        assert!(
            verify_hs256(&token, "attacker-guess").is_err(),
            "a token must never be accepted without correct-key verification"
        );
    }

    #[test]
    fn debug_redacts_bearer() {
        let ctx = OperatorContext {
            subject: "op-123".into(),
            roles: vec![],
            tenant_id: None,
            signature_verified: false,
            bearer: "super-secret-token".into(),
        };
        let dbg = format!("{ctx:?}");
        assert!(
            !dbg.contains("super-secret-token"),
            "bearer must be redacted in Debug"
        );
        assert!(dbg.contains("<redacted>"));
    }
}
