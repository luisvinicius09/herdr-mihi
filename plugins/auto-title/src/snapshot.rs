//! Deserialization of herdr's `session.snapshot` — only the fields the v1 resolver needs.
//! Defensive: every field is optional/defaulted, so a herdr version that adds or removes
//! fields never breaks parsing (fields drift across versions).
use serde::Deserialize;

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
