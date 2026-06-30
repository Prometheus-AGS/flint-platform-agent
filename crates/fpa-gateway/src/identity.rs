//! Gate-derived operator identity.
//!
//! flint-gate is the **only** auth boundary. This module consumes gate-injected
//! credentials (a gate-minted JWT, or gate identity headers) and derives the
//! operator's roles/permissions from the claims. It does **not** call Ory
//! (Kratos/Hydra/Keto/Oathkeeper) and does **not** fetch Ory JWKS.
//!
//! **Interim signature verification:** when a `gate_jwt_key` is configured the
//! token signature is validated; otherwise the token is decoded *without*
//! signature verification (claims-only) until gate's published key/JWKS endpoint
//! is wired. This is a documented stopgap, not the end state.

use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};
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
    /// Expiry (epoch seconds) — validated when signature verification is on.
    #[serde(default)]
    pub exp: Option<usize>,
}

/// The authenticated operator for a request, derived solely from gate claims.
///
/// `roles`/`tenant_id`/`signature_verified` and [`Self::has_role`] are produced
/// here (the c001 identity contract) but consumed by the permission checks in
/// `p1-c003-a2a-task-catalog`; `#[allow(dead_code)]` marks that seam explicitly
/// rather than dropping fields the next change needs.
#[derive(Debug, Clone)]
pub struct OperatorContext {
    pub subject: String,
    #[allow(dead_code)] // consumed by permission checks in p1-c003
    pub roles: Vec<String>,
    #[allow(dead_code)] // consumed by tenant-scoped dispatch in p1-c003
    pub tenant_id: Option<String>,
    /// True only when the JWT signature was verified against a configured key.
    #[allow(dead_code)] // surfaced in audit records in p1-c003
    pub signature_verified: bool,
}

impl OperatorContext {
    /// Whether the operator holds a given role.
    #[allow(dead_code)] // used by permission enforcement in p1-c003
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
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
        let token = bearer_token(parts).ok_or(NoIdentity)?;
        let claims = decode_gate_jwt(&token, state.config.gate_jwt_key.as_deref())
            .map_err(|_| NoIdentity)?;
        // Never log the token or full claim set — only the subject presence.
        tracing::debug!(
            has_subject = !claims.0.sub.is_empty(),
            "gate identity resolved"
        );
        Ok(Self {
            subject: claims.0.sub,
            roles: claims.0.roles,
            tenant_id: claims.0.tenant_id,
            signature_verified: claims.1,
        })
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
fn decode_gate_jwt(
    token: &str,
    key: Option<&str>,
) -> Result<(GateClaims, bool), jsonwebtoken::errors::Error> {
    let header = decode_header(token)?;
    if let Some(secret) = key {
        let mut validation = Validation::new(header.alg);
        validation.validate_exp = true;
        let data = decode::<GateClaims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )?;
        Ok((data.claims, true))
    } else {
        // Interim: structurally decode + read claims WITHOUT signature/exp
        // verification. Replace when gate's key/JWKS endpoint is wired.
        let mut validation = Validation::new(header.alg);
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        validation.required_spec_claims.clear();
        let data = decode::<GateClaims>(token, &DecodingKey::from_secret(b"unused"), &validation)?;
        Ok((data.claims, false))
    }
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
            exp: Some(4_102_444_800), // year 2100
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .expect("mint")
    }

    #[test]
    fn verifies_with_configured_key() {
        let token = mint("shared-key", &["admin", "operator"]);
        let (claims, verified) = decode_gate_jwt(&token, Some("shared-key")).expect("decode");
        assert!(verified);
        assert_eq!(claims.sub, "op-123");
        assert_eq!(claims.roles, vec!["admin", "operator"]);
    }

    #[test]
    fn rejects_bad_signature_when_key_configured() {
        let token = mint("shared-key", &["admin"]);
        let err = decode_gate_jwt(&token, Some("wrong-key"));
        assert!(err.is_err(), "wrong key must fail signature verification");
    }

    #[test]
    fn decodes_claims_unverified_without_key() {
        let token = mint("shared-key", &["operator"]);
        let (claims, verified) = decode_gate_jwt(&token, None).expect("decode unverified");
        assert!(!verified, "no key configured → signature not verified");
        assert_eq!(claims.sub, "op-123");
        assert!(claims.roles.contains(&"operator".to_owned()));
    }

    #[test]
    fn has_role_checks_membership() {
        let ctx = OperatorContext {
            subject: "op-123".into(),
            roles: vec!["admin".into()],
            tenant_id: None,
            signature_verified: true,
        };
        assert!(ctx.has_role("admin"));
        assert!(!ctx.has_role("nope"));
    }
}
