//! Application-layer authorization context.
//!
//! A decoupled view of the operator's gate-derived roles. The gateway builds its
//! own `OperatorContext` from the gate JWT (Layer: interface) and maps it into
//! this type before invoking a use-case, so `fpa-app` never depends on the
//! gateway or on JWT specifics.

/// The operator's authorization context for a use-case invocation.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Operator subject id.
    pub subject: String,
    /// Roles granted by gate.
    pub roles: Vec<String>,
}

impl AuthContext {
    /// Whether the operator holds the given role.
    #[must_use]
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}
