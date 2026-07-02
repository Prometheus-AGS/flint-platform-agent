//! JWKS verifier for **directly-received** tokens.
//!
//! When a token arrives NOT through gate (no trusted headers), it must be
//! signature-verified. This fetches the IdP's JWK Set (the same one gate uses),
//! caches it with a bounded TTL, and verifies RS256/ES256 tokens. Mirrors
//! flint-gate's `jwt_verify.rs` approach.
//!
//! Security posture: absence of a usable key is a **rejection**, never an
//! unverified accept. Tokens/claims are never logged.

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::de::DeserializeOwned;
use std::time::Duration;
use tokio::sync::RwLock;

/// JWKS cache TTL (matches gate's 300s).
const JWKS_TTL: Duration = Duration::from_secs(300);
/// Server-fixed allowed asymmetric algorithms — NOT derived from the token header
/// (prevents algorithm-confusion: an attacker cannot pick the verification alg).
const ALLOWED_ALGS: &[Algorithm] = &[Algorithm::RS256, Algorithm::ES256];
/// Bound the JWKS fetch so a slow IdP can't hang the worker pool.
const JWKS_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

/// Cached JWK set with a fetch instant (monotonic via a tokio Instant).
struct Cached {
    jwks: JwkSet,
    fetched: tokio::time::Instant,
}

/// Verifies directly-received JWTs against a cached IdP JWK set.
pub struct JwksVerifier {
    url: String,
    http: reqwest::Client,
    cache: RwLock<Option<Cached>>,
    /// Expected token issuer(s) — enforced when non-empty (defence against
    /// cross-environment token reuse).
    issuers: Vec<String>,
    /// Expected token audience(s) — enforced when non-empty.
    audiences: Vec<String>,
}

/// Verification failure (never carries token material).
#[derive(Debug)]
pub struct VerifyError;

impl JwksVerifier {
    /// Construct a verifier for the given JWKS URL with optional expected
    /// issuer/audience allowlists (each enforced only when non-empty).
    #[must_use]
    pub fn new(url: String, issuers: Vec<String>, audiences: Vec<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(JWKS_HTTP_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            url,
            http,
            cache: RwLock::new(None),
            issuers,
            audiences,
        }
    }

    /// Return the cached JWK set, fetching (and caching) if absent or stale.
    async fn jwks(&self) -> Result<JwkSet, VerifyError> {
        {
            let guard = self.cache.read().await;
            if let Some(c) = guard.as_ref()
                && c.fetched.elapsed() < JWKS_TTL
            {
                return Ok(c.jwks.clone());
            }
        }
        let resp = self
            .http
            .get(&self.url)
            .send()
            .await
            .map_err(|_| VerifyError)?;
        if !resp.status().is_success() {
            return Err(VerifyError);
        }
        let jwks: JwkSet = resp.json().await.map_err(|_| VerifyError)?;
        // Never cache an empty/malformed key set (poisoning defence).
        if jwks.keys.is_empty() {
            return Err(VerifyError);
        }
        let mut guard = self.cache.write().await;
        *guard = Some(Cached {
            jwks: jwks.clone(),
            fetched: tokio::time::Instant::now(),
        });
        Ok(jwks)
    }

    /// Verify `token`'s signature against the JWKS and decode its claims.
    ///
    /// Security: the accepted algorithm is **server-fixed** ([`ALLOWED_ALGS`]) and
    /// never derived from the token header (prevents algorithm confusion). Expiry
    /// is validated; issuer/audience are enforced when configured. A token whose
    /// header names an alg outside the allowlist, or with no matching key, yields
    /// [`VerifyError`] — never an unverified accept.
    pub async fn verify<C: DeserializeOwned>(&self, token: &str) -> Result<C, VerifyError> {
        let header = decode_header(token).map_err(|_| VerifyError)?;
        // Reject any algorithm the server does not allow BEFORE touching keys.
        if !ALLOWED_ALGS.contains(&header.alg) {
            return Err(VerifyError);
        }
        let jwks = self.jwks().await?;

        let jwk = match &header.kid {
            Some(kid) => jwks.find(kid).ok_or(VerifyError)?,
            // No kid: only acceptable if the set has exactly one key.
            None if jwks.keys.len() == 1 => &jwks.keys[0],
            None => return Err(VerifyError),
        };

        let decoding = DecodingKey::from_jwk(jwk).map_err(|_| VerifyError)?;
        // Validate against the SERVER-fixed algorithm set, not header.alg.
        let mut validation = Validation::new(header.alg);
        validation.algorithms = ALLOWED_ALGS.to_vec();
        validation.validate_exp = true;
        if !self.issuers.is_empty() {
            validation.set_issuer(&self.issuers);
        }
        if self.audiences.is_empty() {
            // No audience configured: don't let a token's aud silently pass.
            validation.validate_aud = false;
        } else {
            validation.set_audience(&self.audiences);
        }
        let data = decode::<C>(token, &decoding, &validation).map_err(|_| VerifyError)?;
        Ok(data.claims)
    }
}
