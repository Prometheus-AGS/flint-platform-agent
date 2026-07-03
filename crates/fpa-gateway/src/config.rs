//! Typed gateway configuration, loaded from the environment.
//!
//! Required values fail fast at startup (Base Rule 33: validate at boundaries,
//! no silent defaults for security-relevant URLs). Optional values have explicit
//! documented defaults.

use anyhow::{Context as _, anyhow};
use std::net::SocketAddr;

/// Env var: gateway bind address. Optional (defaults to [`DEFAULT_ADDR`]).
const ENV_ADDR: &str = "FPA_GATEWAY_ADDR";
/// Env var: flint-forge Quarry gateway base URL. **Required.**
const ENV_FORGE_URL: &str = "FPA_FORGE_URL";
/// Env var: flint-realtime-fabric gateway endpoint. **Required.**
const ENV_FABRIC_ENDPOINT: &str = "FPA_FABRIC_ENDPOINT";
/// Env var: flint-gate **admin** base URL (private; never public). **Required.**
const ENV_GATE_ADMIN_URL: &str = "FPA_GATE_ADMIN_URL";
/// Env var: HS256 shared secret for verifying tokens (optional; one verification
/// path — the other is JWKS via [`ENV_JWKS_URL`]).
const ENV_GATE_JWT_KEY: &str = "FPA_GATE_JWT_KEY";
/// Env var: comma-separated list of headers gate injects for a trusted (through-gate)
/// request, e.g. `X-User-Id,X-Org-Id`. When ALL are present, identity is trusted
/// without token verification. Optional (empty ⇒ no trusted-header path).
const ENV_TRUSTED_HEADERS: &str = "FPA_TRUSTED_IDENTITY_HEADERS";
/// Env var: the IdP JWKS URL (the same JWKS gate verifies against) for verifying
/// directly-received tokens (RS256/ES256). Optional. MUST be `https://`.
const ENV_JWKS_URL: &str = "FPA_JWKS_URL";
/// Env var: expected token issuer(s), comma-separated (enforced when set).
const ENV_JWT_ISSUER: &str = "FPA_JWT_ISSUER";
/// Env var: expected token audience(s), comma-separated (enforced when set).
const ENV_JWT_AUDIENCE: &str = "FPA_JWT_AUDIENCE";
/// Env var: explicit acknowledgment that the gateway is deployed behind gate on a
/// trusted, non-public network. REQUIRED to be `true` when trusted identity
/// headers are configured — otherwise trusting client-set headers is spoofable.
const ENV_BEHIND_GATE: &str = "FPA_BEHIND_TRUSTED_GATE";
/// Env var: forge REST path prefix (Supabase-style, e.g. `/rest`). Optional;
/// defaults to the adapter default. Config so a forge change is a config fix.
const ENV_FORGE_REST_PREFIX: &str = "FPA_FORGE_REST_PREFIX";
/// Env var: bearer token for gate's admin API. Optional.
const ENV_GATE_ADMIN_TOKEN: &str = "FPA_GATE_ADMIN_TOKEN";
/// Env var: Postgres connection URL for the durable `ProjectStore` (p6-c001).
/// Optional — when absent the agent uses the in-memory store. Contains a secret
/// (password); never logged (redacted in `Debug`).
const ENV_PROJECT_DB_URL: &str = "FPA_PROJECT_DB_URL";

/// Default bind address when `FPA_GATEWAY_ADDR` is unset.
const DEFAULT_ADDR: &str = "0.0.0.0:8088";

/// Resolved, validated gateway configuration.
#[derive(Clone)]
pub struct GatewayConfig {
    /// Address the gateway binds.
    pub addr: SocketAddr,
    /// flint-forge Quarry base URL.
    pub forge_url: String,
    /// flint-realtime-fabric endpoint.
    pub fabric_endpoint: String,
    /// flint-gate admin base URL (private).
    pub gate_admin_url: String,
    /// HS256 shared secret for token verification (one of two verify paths).
    pub gate_jwt_key: Option<String>,
    /// Headers gate injects for a trusted through-gate request. When ALL are
    /// present, identity is trusted without token verification.
    pub trusted_identity_headers: Vec<String>,
    /// IdP JWKS URL for verifying directly-received tokens (RS256/ES256).
    pub jwks_url: Option<String>,
    /// Expected token issuer(s), enforced on the verify path when non-empty.
    pub jwt_issuers: Vec<String>,
    /// Expected token audience(s), enforced on the verify path when non-empty.
    pub jwt_audiences: Vec<String>,
    /// Forge REST path prefix override (`None` ⇒ adapter default).
    pub forge_rest_prefix: Option<String>,
    /// Bearer token for gate's admin API (`None` ⇒ unauthenticated admin calls).
    pub gate_admin_token: Option<String>,
    /// Postgres URL for the durable `ProjectStore` (`None` ⇒ in-memory store).
    /// Contains a secret; never logged.
    pub project_db_url: Option<String>,
}

// Manual Debug redacts the HS256 secret — deriving would leak it in logs/panics.
impl std::fmt::Debug for GatewayConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayConfig")
            .field("addr", &self.addr)
            .field("forge_url", &self.forge_url)
            .field("fabric_endpoint", &self.fabric_endpoint)
            .field("gate_admin_url", &self.gate_admin_url)
            .field(
                "gate_jwt_key",
                &self.gate_jwt_key.as_ref().map(|_| "<redacted>"),
            )
            .field("trusted_identity_headers", &self.trusted_identity_headers)
            .field("jwks_url", &self.jwks_url)
            .field("jwt_issuers", &self.jwt_issuers)
            .field("jwt_audiences", &self.jwt_audiences)
            .field("forge_rest_prefix", &self.forge_rest_prefix)
            .field(
                "gate_admin_token",
                &self.gate_admin_token.as_ref().map(|_| "<redacted>"),
            )
            .field(
                "project_db_url",
                &self.project_db_url.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

/// Split a comma-separated env value into trimmed, non-empty parts.
fn csv(v: Option<String>) -> Vec<String> {
    v.map(|s| {
        s.split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(str::to_owned)
            .collect()
    })
    .unwrap_or_default()
}

impl GatewayConfig {
    /// Load and validate configuration from the process environment.
    ///
    /// # Errors
    /// Returns an error naming the first missing/invalid required variable so
    /// startup aborts before binding a port.
    pub fn from_env() -> anyhow::Result<Self> {
        Self::from_lookup(|k| std::env::var(k).ok())
    }

    /// Parse + validate configuration from an arbitrary key→value lookup.
    ///
    /// Pure (no process env access) so it is testable without env-var races.
    ///
    /// # Errors
    /// Returns an error naming the first missing/invalid required variable.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> anyhow::Result<Self> {
        let addr_raw = lookup(ENV_ADDR).unwrap_or_else(|| DEFAULT_ADDR.to_owned());
        let addr: SocketAddr = addr_raw
            .parse()
            .with_context(|| format!("{ENV_ADDR} must be host:port, got {addr_raw:?}"))?;

        let trusted_identity_headers: Vec<String> = csv(lookup(ENV_TRUSTED_HEADERS))
            .into_iter()
            .map(|h| h.to_ascii_lowercase())
            .collect();

        // C1 (security review): trusting client-set headers is spoofable unless the
        // gateway is deployed behind gate on a private network. Require an explicit
        // acknowledgment so a misconfiguration fails at startup, not silently.
        if !trusted_identity_headers.is_empty() {
            let ack = lookup(ENV_BEHIND_GATE).is_some_and(|v| v.eq_ignore_ascii_case("true"));
            if !ack {
                return Err(anyhow!(
                    "{ENV_TRUSTED_HEADERS} is set (header-trust auth) but {ENV_BEHIND_GATE} \
                     is not 'true'. Trusting client-set identity headers is spoofable unless \
                     this gateway is reachable ONLY through flint-gate on a private network. \
                     Set {ENV_BEHIND_GATE}=true to acknowledge that deployment constraint."
                ));
            }
        }

        // M4 (security review): JWKS must be fetched over TLS to prevent MITM key
        // injection.
        let jwks_url = lookup(ENV_JWKS_URL).filter(|v| !v.trim().is_empty());
        if let Some(url) = &jwks_url
            && !url.starts_with("https://")
        {
            return Err(anyhow!(
                "{ENV_JWKS_URL} must be an https:// URL, got {url:?}"
            ));
        }

        Ok(Self {
            addr,
            forge_url: require(&lookup, ENV_FORGE_URL)?,
            fabric_endpoint: require(&lookup, ENV_FABRIC_ENDPOINT)?,
            gate_admin_url: require(&lookup, ENV_GATE_ADMIN_URL)?,
            gate_jwt_key: lookup(ENV_GATE_JWT_KEY).filter(|v| !v.trim().is_empty()),
            trusted_identity_headers,
            jwks_url,
            jwt_issuers: csv(lookup(ENV_JWT_ISSUER)),
            jwt_audiences: csv(lookup(ENV_JWT_AUDIENCE)),
            forge_rest_prefix: lookup(ENV_FORGE_REST_PREFIX).filter(|v| !v.trim().is_empty()),
            gate_admin_token: lookup(ENV_GATE_ADMIN_TOKEN).filter(|v| !v.trim().is_empty()),
            project_db_url: lookup(ENV_PROJECT_DB_URL).filter(|v| !v.trim().is_empty()),
        })
    }
}

/// Read a required non-empty value from the lookup.
fn require(lookup: &impl Fn(&str) -> Option<String>, key: &str) -> anyhow::Result<String> {
    match lookup(key) {
        Some(v) if !v.trim().is_empty() => Ok(v),
        Some(_) => Err(anyhow!("{key} is set but empty")),
        None => Err(anyhow!("{key} is required")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// Build a lookup closure from an in-memory map (no process env — no races).
    fn lookup_from(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        move |k: &str| map.get(k).cloned()
    }

    #[test]
    fn rejects_missing_required_var() {
        // FORGE present; FABRIC + GATE missing → must fail naming one.
        let lookup = lookup_from(&[(ENV_FORGE_URL, "http://forge:8080")]);
        let err = GatewayConfig::from_lookup(lookup).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains(ENV_FABRIC_ENDPOINT) || msg.contains(ENV_GATE_ADMIN_URL),
            "error should name a missing required var, got: {msg}"
        );
    }

    #[test]
    fn parses_full_config_with_default_addr() {
        let lookup = lookup_from(&[
            (ENV_FORGE_URL, "http://forge:8080"),
            (ENV_FABRIC_ENDPOINT, "http://fabric:9090"),
            (ENV_GATE_ADMIN_URL, "http://gate:4457"),
        ]);
        let cfg = GatewayConfig::from_lookup(lookup).expect("valid config");
        assert_eq!(cfg.addr.to_string(), "0.0.0.0:8088");
        assert_eq!(cfg.forge_url, "http://forge:8080");
        assert!(cfg.gate_jwt_key.is_none());
        assert!(cfg.trusted_identity_headers.is_empty());
        assert!(cfg.jwks_url.is_none());
    }

    #[test]
    fn parses_trusted_headers_lowercased() {
        let lookup = lookup_from(&[
            (ENV_FORGE_URL, "http://forge:8080"),
            (ENV_FABRIC_ENDPOINT, "http://fabric:9090"),
            (ENV_GATE_ADMIN_URL, "http://gate:4457"),
            (ENV_TRUSTED_HEADERS, "X-User-Id, X-Org-Id ,"),
            (ENV_BEHIND_GATE, "true"),
            (ENV_JWKS_URL, "https://idp/.well-known/jwks.json"),
        ]);
        let cfg = GatewayConfig::from_lookup(lookup).expect("valid config");
        // Split, trimmed, empties dropped, lowercased (HTTP headers are case-insensitive).
        assert_eq!(cfg.trusted_identity_headers, vec!["x-user-id", "x-org-id"]);
        assert_eq!(
            cfg.jwks_url.as_deref(),
            Some("https://idp/.well-known/jwks.json")
        );
    }

    #[test]
    fn trusted_headers_require_behind_gate_ack() {
        // C1: setting trusted headers without the deployment ack must FAIL.
        let lookup = lookup_from(&[
            (ENV_FORGE_URL, "http://forge:8080"),
            (ENV_FABRIC_ENDPOINT, "http://fabric:9090"),
            (ENV_GATE_ADMIN_URL, "http://gate:4457"),
            (ENV_TRUSTED_HEADERS, "X-User-Id"),
            // FPA_BEHIND_TRUSTED_GATE not set
        ]);
        let err = GatewayConfig::from_lookup(lookup).unwrap_err();
        assert!(err.to_string().contains(ENV_BEHIND_GATE));
    }

    #[test]
    fn jwks_url_must_be_https() {
        // M4: plaintext JWKS is rejected (MITM key-injection defence).
        let lookup = lookup_from(&[
            (ENV_FORGE_URL, "http://forge:8080"),
            (ENV_FABRIC_ENDPOINT, "http://fabric:9090"),
            (ENV_GATE_ADMIN_URL, "http://gate:4457"),
            (ENV_JWKS_URL, "http://idp/jwks.json"),
        ]);
        let err = GatewayConfig::from_lookup(lookup).unwrap_err();
        assert!(err.to_string().contains("https"));
    }

    #[test]
    fn rejects_empty_required_var() {
        let lookup = lookup_from(&[
            (ENV_FORGE_URL, "   "),
            (ENV_FABRIC_ENDPOINT, "http://fabric:9090"),
            (ENV_GATE_ADMIN_URL, "http://gate:4457"),
        ]);
        let err = GatewayConfig::from_lookup(lookup).unwrap_err();
        assert!(err.to_string().contains(ENV_FORGE_URL));
    }
}
