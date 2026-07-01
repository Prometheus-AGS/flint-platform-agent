//! Application-layer authorization context.
//!
//! A decoupled view of the operator's gate-derived roles. The gateway builds its
//! own `OperatorContext` from the gate JWT (Layer: interface) and maps it into
//! this type before invoking a use-case, so `fpa-app` never depends on the
//! gateway or on JWT specifics.

/// The operator's authorization context for a use-case invocation.
///
/// Carries the raw gate bearer so an RLS-enforcing downstream (forge) receives
/// the operator's identity. The bearer is never logged (redacted in `Debug`).
#[derive(Clone, Default)]
pub struct AuthContext {
    /// Operator subject id.
    pub subject: String,
    /// Roles granted by gate.
    pub roles: Vec<String>,
    /// The raw gate-minted bearer, forwarded to downstreams that enforce RLS.
    /// `None` when the request carried no gate identity.
    pub bearer: Option<String>,
}

// Manual Debug redacts the bearer — deriving would leak the token.
impl std::fmt::Debug for AuthContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthContext")
            .field("subject", &self.subject)
            .field("roles", &self.roles)
            .field("bearer", &self.bearer.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl AuthContext {
    /// Whether the operator holds the given role.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}
