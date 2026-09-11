//! Deterministic title from a tab's context pane:
//! - Agent pane -> `agent › activity`   (e.g. `claude › Add observation fields`)
//! - Other pane -> `project › activity`  (just `project` when idle)
//!
//! The project is dropped when it's merely the workspace name (herdr already shows the workspace
//! above the tabs). `Shell` if nothing meaningful. No git branch / transcript / LLM in v1.
use crate::config::Config;
use crate::shared::sanitize;
use crate::shared::snapshot::{PaneInfo, TabInfo};

pub fn resolve(
    _tab: &TabInfo,
    panes: &[&PaneInfo],
    focused_pane_id: Option<&str>,
    workspace_label: Option<&str>,
    cfg: &Config,
) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(p) = pick_context(panes, focused_pane_id) {
        // activity: the agent's own title, else the terminal title (noise filtered out)
        let activity = p
            .title
            .clone()
            .filter(|s| sanitize::meaningful(s))
            .or_else(|| {
                p.terminal_title_stripped
                    .clone()
                    .filter(|s| sanitize::meaningful(s))
            })
            .map(|s| sanitize::clean(&s))
            .filter(|s| sanitize::meaningful(s));

        // agent name (e.g. "claude") if this pane is running an agent
        let agent = p
            .agent
            .clone()
            .or_else(|| p.display_agent.clone())
            .map(|s| sanitize::clean(&s))
            .filter(|s| sanitize::meaningful(s));

        if let Some(ag) = agent {
            // agent pane: `agent › activity`
            parts.push(ag);
            if let Some(act) = activity {
                parts.push(act);
            }
        } else {
            // plain pane: `project › activity` — but drop the project if it's just the workspace name
            if let Some(d) = p.foreground_cwd.clone().or_else(|| p.cwd.clone()) {
                if let Some(base) = basename(&d) {
                    if sanitize::meaningful(&base) && Some(base.as_str()) != workspace_label {
                        parts.push(base);
                    }
                }
            }
            if let Some(act) = activity {
                parts.push(act);
            }
        }
    }

    let joined = if parts.is_empty() {
        "Shell".to_string()
    } else {
        parts.join(" › ")
    };
    sanitize::truncate_cols(&joined, cfg.max_length)
}

/// Which pane speaks for a multi-pane tab: focused -> a working/blocked agent -> the first.
fn pick_context<'a>(panes: &[&'a PaneInfo], focused: Option<&str>) -> Option<&'a PaneInfo> {
    if let Some(f) = focused {
        if let Some(p) = panes.iter().find(|p| p.pane_id == f) {
            return Some(*p);
        }
    }
    if let Some(p) = panes
        .iter()
        .find(|p| matches!(p.agent_status.as_deref(), Some("working") | Some("blocked")))
    {
        return Some(*p);
    }
    panes.first().copied()
}

fn basename(path: &str) -> Option<String> {
    let t = path.trim_end_matches('/');
    t.rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config {
            poll_ms: 500,
            max_length: 50,
            debug: false,
        }
    }
    fn pane(
        id: &str,
        cwd: Option<&str>,
        agent: Option<&str>,
        title: Option<&str>,
        term: Option<&str>,
    ) -> PaneInfo {
        PaneInfo {
            pane_id: id.into(),
            cwd: cwd.map(str::to_string),
            agent: agent.map(str::to_string),
            title: title.map(str::to_string),
            terminal_title_stripped: term.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn empty_tab_is_shell() {
        assert_eq!(
            resolve(&TabInfo::default(), &[], None, None, &cfg()),
            "Shell"
        );
    }
    #[test]
    fn plain_pane_just_cwd_when_idle() {
        let p = pane("p1", Some("/home/u/proj"), None, None, None);
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, None, &cfg()),
            "proj"
        );
    }
    #[test]
    fn plain_pane_cwd_and_activity() {
        let p = pane(
            "p1",
            Some("/home/u/proj"),
            None,
            None,
            Some("editing main.rs"),
        );
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, None, &cfg()),
            "proj › editing main.rs"
        );
    }
    #[test]
    fn plain_pane_ignores_prompt_terminal_title() {
        let p = pane(
            "p1",
            Some("/home/u/proj"),
            None,
            None,
            Some("luisvinicius@Luiss-MacBook-Air"),
        );
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, None, &cfg()),
            "proj"
        );
    }
    #[test]
    fn drops_project_when_it_equals_workspace() {
        let p = pane("p1", Some("/home/u/herdr-mihi"), None, None, Some("nvim"));
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, Some("herdr-mihi"), &cfg()),
            "nvim"
        );
    }
    #[test]
    fn keeps_project_when_deeper_than_workspace() {
        let p = pane(
            "p1",
            Some("/home/u/herdr-mihi/plugins/auto-title"),
            None,
            None,
            Some("nvim"),
        );
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, Some("herdr-mihi"), &cfg()),
            "auto-title › nvim"
        );
    }
    #[test]
    fn agent_pane_shows_agent_and_title() {
        let p = pane(
            "p1",
            Some("/x/agrotroca"),
            Some("claude"),
            Some("Add observation fields"),
            None,
        );
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, None, &cfg()),
            "claude › Add observation fields"
        );
    }
    #[test]
    fn agent_pane_idle_shows_just_agent() {
        let p = pane(
            "p1",
            Some("/x/agrotroca"),
            Some("claude"),
            None,
            Some("luisvinicius@host"),
        );
        assert_eq!(
            resolve(&TabInfo::default(), &[&p], None, None, &cfg()),
            "claude"
        );
    }
    #[test]
    fn focused_pane_wins_context() {
        let a = pane("a", Some("/x/alpha"), None, None, None);
        let b = pane("b", Some("/y/beta"), None, None, None);
        assert_eq!(
            resolve(&TabInfo::default(), &[&a, &b], Some("b"), None, &cfg()),
            "beta"
        );
    }
}
