//! The testable core: turn a `session.snapshot` into workspace-grouped, attention-sorted rows.
//! Prefers herdr's `agents[]`; falls back to synthesizing from `panes[]`+`tabs[]` on herdr
//! versions that don't populate `agents[]`. All agent-derived text is sanitized.
use crate::shared::sanitize;
use crate::shared::snapshot::SessionSnapshot;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Blocked,
    Done,
    Working,
    Idle,
    Unknown,
}

impl Status {
    pub fn parse(s: Option<&str>) -> Status {
        match s.unwrap_or("").to_lowercase().as_str() {
            "blocked" => Status::Blocked,
            "done" => Status::Done,
            "working" => Status::Working,
            "idle" => Status::Idle,
            _ => Status::Unknown,
        }
    }
    /// Attention order: blocked first, unknown last.
    pub fn rank(self) -> u8 {
        match self {
            Status::Blocked => 0,
            Status::Done => 1,
            Status::Working => 2,
            Status::Idle => 3,
            Status::Unknown => 4,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Status::Blocked => "blocked",
            Status::Done => "done",
            Status::Working => "working",
            Status::Idle => "idle",
            Status::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    pub pane_id: String,
    pub agent: String,
    pub activity: String,
    pub status: Status,
    pub seq: u64,
    pub cwd: String,
    pub tokens: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub workspace: String,
    pub rows: Vec<Row>,
}

/// Unwrap the raw socket envelope into a snapshot (herdr nests it under `result.snapshot`).
pub fn snapshot_from_raw(raw: &serde_json::Value) -> Option<SessionSnapshot> {
    if raw.get("error").is_some() {
        return None;
    }
    let result = raw
        .get("result")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let snap_val = result.get("snapshot").cloned().unwrap_or(result);
    serde_json::from_value(snap_val).ok()
}

fn pick_agent(display: &Option<String>, agent: &Option<String>, name: &Option<String>) -> String {
    display
        .clone()
        .or_else(|| agent.clone())
        .or_else(|| name.clone())
        .map(|s| sanitize::clean(&s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent".to_string())
}

fn pick_activity(title: &Option<String>, term: &Option<String>) -> String {
    title
        .clone()
        .filter(|s| sanitize::meaningful(s))
        .or_else(|| term.clone().filter(|s| sanitize::meaningful(s)))
        .map(|s| sanitize::clean(&s))
        .filter(|s| sanitize::meaningful(s))
        .unwrap_or_default()
}

/// Build the grouped, sorted view. Groups float up by their most-urgent agent; rows within a
/// group sort attention-first (ties: most-recently-changed, then agent name).
pub fn build_view(snap: &SessionSnapshot, max_rows: usize) -> Vec<Group> {
    let ws: HashMap<&str, &str> = snap
        .workspaces
        .iter()
        .map(|w| (w.workspace_id.as_str(), w.label.as_str()))
        .collect();

    let mut by_ws: HashMap<String, Vec<Row>> = HashMap::new();
    let mut n = 0;

    let ws_label = |wid: &str| -> String {
        ws.get(wid)
            .copied()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| {
                if wid.is_empty() {
                    "—".into()
                } else {
                    wid.into()
                }
            })
    };

    if !snap.agents.is_empty() {
        for a in &snap.agents {
            if n >= max_rows {
                break;
            }
            let row = Row {
                pane_id: a.pane_id.clone(),
                agent: pick_agent(&a.display_agent, &a.agent, &a.name),
                activity: pick_activity(&a.title, &a.terminal_title_stripped),
                status: Status::parse(a.agent_status.as_deref()),
                seq: a.state_change_seq,
                cwd: a
                    .foreground_cwd
                    .clone()
                    .or_else(|| a.cwd.clone())
                    .unwrap_or_default(),
                tokens: top_tokens(&a.tokens),
            };
            by_ws
                .entry(ws_label(&a.workspace_id))
                .or_default()
                .push(row);
            n += 1;
        }
    } else {
        // Fallback: derive from panes that carry an agent, mapping pane -> tab -> workspace.
        let tab_ws: HashMap<&str, &str> = snap
            .tabs
            .iter()
            .map(|t| (t.tab_id.as_str(), t.workspace_id.as_str()))
            .collect();
        for p in &snap.panes {
            if n >= max_rows {
                break;
            }
            if p.agent.is_none() {
                continue;
            }
            let wid = tab_ws.get(p.tab_id.as_str()).copied().unwrap_or("");
            let row = Row {
                pane_id: p.pane_id.clone(),
                agent: pick_agent(&p.display_agent, &p.agent, &None),
                activity: pick_activity(&p.title, &p.terminal_title_stripped),
                status: Status::parse(p.agent_status.as_deref()),
                seq: 0,
                cwd: p
                    .foreground_cwd
                    .clone()
                    .or_else(|| p.cwd.clone())
                    .unwrap_or_default(),
                tokens: Vec::new(),
            };
            by_ws.entry(ws_label(wid)).or_default().push(row);
            n += 1;
        }
    }

    let mut groups: Vec<Group> = by_ws
        .into_iter()
        .map(|(workspace, mut rows)| {
            rows.sort_by(|a, b| {
                a.status
                    .rank()
                    .cmp(&b.status.rank())
                    .then(b.seq.cmp(&a.seq))
                    .then(a.agent.cmp(&b.agent))
            });
            Group { workspace, rows }
        })
        .collect();

    groups.sort_by(|a, b| {
        let ua = a.rows.iter().map(|r| r.status.rank()).min().unwrap_or(255);
        let ub = b.rows.iter().map(|r| r.status.rank()).min().unwrap_or(255);
        ua.cmp(&ub).then_with(|| a.workspace.cmp(&b.workspace))
    });
    groups
}

/// Up to 3 display tokens (sorted by key) for a row's chips.
fn top_tokens(m: &HashMap<String, String>) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = m
        .iter()
        .map(|(k, val)| (k.clone(), sanitize::clean(val)))
        .filter(|(_, val)| !val.is_empty())
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v.truncate(3);
    v
}

/// (blocked, done, working, idle) counts across all groups — for the header line.
pub fn counts(groups: &[Group]) -> (usize, usize, usize, usize) {
    let mut c = (0, 0, 0, 0);
    for g in groups {
        for r in &g.rows {
            match r.status {
                Status::Blocked => c.0 += 1,
                Status::Done => c.1 += 1,
                Status::Working => c.2 += 1,
                Status::Idle => c.3 += 1,
                Status::Unknown => {}
            }
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::snapshot::{AgentInfo, WorkspaceInfo};

    fn agent(pane: &str, ws: &str, status: &str, name: &str, seq: u64) -> AgentInfo {
        AgentInfo {
            pane_id: pane.into(),
            workspace_id: ws.into(),
            agent_status: Some(status.into()),
            display_agent: Some(name.into()),
            state_change_seq: seq,
            ..Default::default()
        }
    }

    fn snap(agents: Vec<AgentInfo>, ws: Vec<(&str, &str)>) -> SessionSnapshot {
        SessionSnapshot {
            agents,
            workspaces: ws
                .into_iter()
                .map(|(id, label)| WorkspaceInfo {
                    workspace_id: id.into(),
                    label: label.into(),
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn rows_sort_attention_first_within_group() {
        let s = snap(
            vec![
                agent("w1:p1", "w1", "idle", "a", 1),
                agent("w1:p2", "w1", "blocked", "b", 2),
                agent("w1:p3", "w1", "working", "c", 3),
            ],
            vec![("w1", "proj")],
        );
        let v = build_view(&s, 100);
        let order: Vec<_> = v[0].rows.iter().map(|r| r.status).collect();
        assert_eq!(order, vec![Status::Blocked, Status::Working, Status::Idle]);
    }

    #[test]
    fn workspace_with_blocked_floats_up() {
        let s = snap(
            vec![
                agent("w1:p1", "w1", "idle", "a", 1),
                agent("w2:p1", "w2", "blocked", "b", 2),
            ],
            vec![("w1", "alpha"), ("w2", "beta")],
        );
        let v = build_view(&s, 100);
        assert_eq!(v[0].workspace, "beta"); // has the blocked agent
        assert_eq!(v[1].workspace, "alpha");
    }

    #[test]
    fn ties_break_on_recency() {
        let s = snap(
            vec![
                agent("w1:p1", "w1", "working", "old", 5),
                agent("w1:p2", "w1", "working", "new", 9),
            ],
            vec![("w1", "proj")],
        );
        let v = build_view(&s, 100);
        assert_eq!(v[0].rows[0].agent, "new"); // higher seq first
    }

    #[test]
    fn counts_tally() {
        let s = snap(
            vec![
                agent("w1:p1", "w1", "blocked", "a", 1),
                agent("w1:p2", "w1", "done", "b", 2),
                agent("w2:p1", "w2", "working", "c", 3),
            ],
            vec![("w1", "x"), ("w2", "y")],
        );
        let (b, d, w, i) = counts(&build_view(&s, 100));
        assert_eq!((b, d, w, i), (1, 1, 1, 0));
    }
}
