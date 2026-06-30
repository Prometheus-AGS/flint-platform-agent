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
/// Env var: key used to validate gate-minted JWTs. Optional at this stage
/// (placeholder until gate's published key/JWKS endpoint is wired — see
/// [`GatewayConfig::gate_jwt_key`]).
const ENV_GATE_JWT_KEY: &str = "FPA_GATE_JWT_KEY";

/// Default bind address when `FPA_GATEWAY_ADDR` is unset.
const DEFAULT_ADDR: &str = "0.0.0.0:8088";

/// Resolved, validated gateway configuration.
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Address the gateway binds.
    pub addr: SocketAddr,
    /// flint-forge Quarry base URL.
    pub forge_url: String,
    /// flint-realtime-fabric endpoint.
    pub fabric_endpoint: String,
    /// flint-gate admin base URL (private).
    pub gate_admin_url: String,
    /// Shared secret / key used to validate gate-minted JWTs.
    ///
    /// **Interim:** sourced from `FPA_GATE_JWT_KEY`. Full signature verification
    /// against gate's published key/JWKS endpoint is a later task; absence here
    /// means tokens are decoded but not signature-verified.
    pub gate_jwt_key: Option<String>,
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

        Ok(Self {
            addr,
            forge_url: require(&lookup, ENV_FORGE_URL)?,
            fabric_endpoint: require(&lookup, ENV_FABRIC_ENDPOINT)?,
            gate_admin_url: require(&lookup, ENV_GATE_ADMIN_URL)?,
            gate_jwt_key: lookup(ENV_GATE_JWT_KEY).filter(|v| !v.trim().is_empty()),
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
