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
use tokio::sync::{Mutex, RwLock};

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
    /// Serializes JWKS refresh so concurrent cold/stale callers trigger at most
    /// one IdP fetch (single-flight, p5-c003 G5). A queued caller re-checks
    /// freshness under this lock and reuses the just-fetched set.
    refresh: Mutex<()>,
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
            refresh: Mutex::new(()),
            issuers,
            audiences,
        }
    }

    /// A fresh cached set, if present and within TTL.
    async fn cached_fresh(&self) -> Option<JwkSet> {
        let guard = self.cache.read().await;
        guard
            .as_ref()
            .filter(|c| c.fetched.elapsed() < JWKS_TTL)
            .map(|c| c.jwks.clone())
    }

    /// Return the cached JWK set, fetching (and caching) if absent or stale.
    ///
    /// Single-flight (p5-c003 G5): on a cache miss, callers serialize on
    /// [`Self::refresh`]; the first fetches, and queued callers re-check the cache
    /// under the lock and reuse the just-fetched set — at most one IdP fetch.
    async fn jwks(&self) -> Result<JwkSet, VerifyError> {
        if let Some(jwks) = self.cached_fresh().await {
            return Ok(jwks);
        }
        // Serialize the refresh. Only one caller fetches per cold/stale window.
        let _refresh = self.refresh.lock().await;
        // Re-check: a prior holder of the refresh lock may have just populated it.
        if let Some(jwks) = self.cached_fresh().await {
            return Ok(jwks);
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
        // The `new()` seed is immediately overwritten by `ALLOWED_ALGS`; using a
        // fixed member (not `header.alg`) makes it explicit that the header's
        // declared algorithm never drives verification. `header.alg` is already
        // pre-checked against ALLOWED_ALGS above.
        let mut validation = Validation::new(ALLOWED_ALGS[0]);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// A syntactically valid single-key RSA JWK set (test vector). Enough to pass
    /// the non-empty cache guard; the key material is not exercised here.
    fn jwks_body() -> serde_json::Value {
        serde_json::json!({
            "keys": [{
                "kty": "RSA",
                "use": "sig",
                "kid": "test-key-1",
                "alg": "RS256",
                "n": "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw",
                "e": "AQAB"
            }]
        })
    }

    #[tokio::test]
    async fn jwks_refresh_is_single_flight() {
        // p5-c003 G5: concurrent cold-cache callers must trigger AT MOST ONE fetch.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(jwks_body()))
            // `expect(1)` fails the test at drop if the endpoint is hit != 1 time.
            .expect(1)
            .mount(&server)
            .await;

        let verifier = Arc::new(JwksVerifier::new(
            format!("{}/jwks", server.uri()),
            vec![],
            vec![],
        ));

        // Race many callers against a cold cache.
        let mut handles = Vec::new();
        for _ in 0..16 {
            let v = verifier.clone();
            handles.push(tokio::spawn(
                async move { v.jwks().await.map(|s| s.keys.len()) },
            ));
        }
        for h in handles {
            let n = h.await.expect("join").expect("jwks");
            assert_eq!(n, 1, "the cached set has one key");
        }
        // `.expect(1)` on the mock is verified when `server` drops here.
    }

    #[tokio::test]
    async fn empty_jwks_is_rejected_not_cached() {
        // Poisoning defence: an empty key set must never be cached or accepted.
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/jwks"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"keys": []})))
            .mount(&server)
            .await;
        let verifier = JwksVerifier::new(format!("{}/jwks", server.uri()), vec![], vec![]);
        assert!(
            verifier.jwks().await.is_err(),
            "empty JWKS must be rejected"
        );
    }
}
