#![allow(dead_code)] // shared module: not every consumer uses every field
//! Deserialization of herdr's `session.snapshot` — only the fields herdr-mihi plugins need.
//! Defensive: every field is optional/defaulted, so a herdr version that adds or removes
//! fields never breaks parsing (fields drift across versions).
//! Canonical: `shared/rust/snapshot.rs` — synced into each plugin, CI drift-checked.
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct SessionSnapshot {
    #[serde(default)]
    pub focused_pane_id: Option<String>,
    #[serde(default)]
    pub tabs: Vec<TabInfo>,
    #[serde(default)]
    pub panes: Vec<PaneInfo>,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceInfo>,
    /// Agent-centric view (richer than `panes[]`); used by the dashboard.
    #[serde(default)]
    pub agents: Vec<AgentInfo>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct WorkspaceInfo {
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TabInfo {
    #[serde(default)]
    pub tab_id: String,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub label: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct PaneInfo {
    #[serde(default)]
    pub pane_id: String,
    #[serde(default)]
    pub tab_id: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub foreground_cwd: Option<String>,
    /// The agent running in this pane (e.g. "claude"), if any.
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    /// The agent's own reported title (highest-confidence activity source).
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub terminal_title_stripped: Option<String>,
    /// herdr serializes this as a lowercase string ("working"/"blocked"/…); kept as a plain
    /// Option<String> so an unknown value can never fail deserialization.
    #[serde(default)]
    pub agent_status: Option<String>,
}

/// An agent as herdr reports it in `snapshot.agents[]` / `agent.list` — richer than `PaneInfo`.
/// The dashboard renders + jumps from these. **`pane_id` (`w:p`) is the jump target**, never
/// `terminal_id` (herdr's `agent focus` rejects terminal ids).
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AgentInfo {
    #[serde(default)]
    pub pane_id: String,
    #[serde(default)]
    pub terminal_id: String,
    #[serde(default)]
    pub workspace_id: String,
    #[serde(default)]
    pub tab_id: String,
    #[serde(default)]
    pub focused: bool,
    /// "idle" | "working" | "blocked" | "done" | "unknown" (kept as a string; unknowns are safe).
    #[serde(default)]
    pub agent_status: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub display_agent: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub foreground_cwd: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub terminal_title_stripped: Option<String>,
    /// Monotonic; herdr bumps it on every state transition — usable for sort/dedup.
    #[serde(default)]
    pub state_change_seq: u64,
    /// Arbitrary display tokens other integrations set (rendered as `$name` in herdr's sidebar).
    #[serde(default)]
    pub tokens: HashMap<String, String>,
    /// Per-status custom labels (keys: idle/working/blocked/done/unknown).
    #[serde(default)]
    pub state_labels: HashMap<String, String>,
}
