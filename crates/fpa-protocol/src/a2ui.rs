//! A2UI component primitives — the platform-wide UI vocabulary.
//!
//! These primitives are emitted by the agent and rendered by any host (web,
//! Tauri desktop, CLI). They are the canonical set reused by every downstream
//! Prometheus agent. Per Base Rule 39, they are typed, versioned, inspectable,
//! and host-portable — **do not** invent ad-hoc UI schemas; extend this set.
//!
//! Components bind to fabric functions via the `action` field (an A2A task `kind`
//! plus input template), so a rendered control can invoke an administrative task.

use serde::{Deserialize, Serialize};

/// A renderable A2UI component primitive.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Component {
    /// Vertical/horizontal grouping container.
    Stack {
        direction: StackDirection,
        children: Vec<Component>,
    },

    /// Static text / heading.
    Text { content: String },

    /// A button that, when pressed, submits an [`Action`].
    Button { label: String, action: Action },

    /// A single-line input bound to a field name in the action input.
    TextField {
        label: String,
        field: String,
        placeholder: Option<String>,
    },

    /// Tabular display of fabric data (e.g. a forge `TableMeta` result).
    Table {
        columns: Vec<String>,
        rows: Vec<Vec<serde_json::Value>>,
    },

    /// Unrecognized or future variant — preserved without loss.
    #[serde(other)]
    Unknown,
}

/// Layout direction for a [`Component::Stack`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackDirection {
    Vertical,
    Horizontal,
}

/// Binds a UI control to a fabric administrative task.
///
/// `kind` is an A2A task catalog key; `input` is a template merged with the
/// component's collected field values at invocation time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub kind: String,
    pub input: serde_json::Value,
}
