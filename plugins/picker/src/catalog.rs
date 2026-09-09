//! Catalog + installed-state logic for the picker. Pure where possible (unit-tested);
//! the only impurity is reading catalog.json and running `herdr plugin list`.
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct Catalog {
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub plugins: Vec<CatalogPlugin>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct CatalogPlugin {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Available,
    Installed(String),
    Update { installed: String, latest: String },
}

/// Compare the catalog's version against what's installed.
pub fn state_for(latest: &str, installed: Option<&str>) -> State {
    match installed {
        None => State::Available,
        Some(v) if v == latest => State::Installed(v.to_string()),
        Some(v) => State::Update {
            installed: v.to_string(),
            latest: latest.to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Install,
    Uninstall,
}

/// The action taken when a row in the given state is selected.
pub fn action_for(state: &State) -> Action {
    match state {
        State::Available | State::Update { .. } => Action::Install, // install or upgrade
        State::Installed(_) => Action::Uninstall,
    }
}

/// The herdr CLI argv (after the leading `herdr`) for a plugin action.
pub fn command_args(owner: &str, p: &CatalogPlugin, action: &Action) -> Vec<String> {
    match action {
        Action::Install => vec![
            "plugin".into(),
            "install".into(),
            owner.into(),
            "--ref".into(),
            format!("{}-latest", p.name),
            "--yes".into(),
        ],
        Action::Uninstall => vec!["plugin".into(), "uninstall".into(), p.id.clone()],
    }
}

/// Load the catalog from the plugin root, falling back to the repo root (dev via `plugin link`).
pub fn load(plugin_root: &str) -> Result<Catalog, String> {
    for path in [
        format!("{plugin_root}/catalog.json"),
        format!("{plugin_root}/../../catalog.json"),
    ] {
        if let Ok(content) = std::fs::read_to_string(&path) {
            return serde_json::from_str(&content).map_err(|e| format!("catalog parse error: {e}"));
        }
    }
    Err("catalog.json not found (bundled with the plugin, or at the repo root for dev)".into())
}

/// Parse `herdr plugin list --json` output into id -> version.
pub fn parse_installed(json: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(json) {
        if let Some(arr) = v
            .get("result")
            .and_then(|r| r.get("plugins"))
            .and_then(|p| p.as_array())
        {
            for p in arr {
                if let Some(id) = p.get("plugin_id").and_then(|x| x.as_str()) {
                    let ver = p
                        .get("version")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    m.insert(id.to_string(), ver);
                }
            }
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_reflects_installed_version() {
        assert_eq!(state_for("1.0.0", None), State::Available);
        assert_eq!(
            state_for("1.0.0", Some("1.0.0")),
            State::Installed("1.0.0".into())
        );
        assert_eq!(
            state_for("1.1.0", Some("1.0.0")),
            State::Update {
                installed: "1.0.0".into(),
                latest: "1.1.0".into()
            }
        );
    }

    #[test]
    fn action_follows_state() {
        assert_eq!(action_for(&State::Available), Action::Install);
        assert_eq!(
            action_for(&State::Update {
                installed: "1".into(),
                latest: "2".into()
            }),
            Action::Install
        );
        assert_eq!(action_for(&State::Installed("1".into())), Action::Uninstall);
    }

    #[test]
    fn commands_are_pinned_and_by_ref() {
        let p = CatalogPlugin {
            id: "herdr-mihi.lazygit".into(),
            name: "lazygit".into(),
            ..Default::default()
        };
        assert_eq!(
            command_args("you/herdr-mihi", &p, &Action::Install),
            vec![
                "plugin",
                "install",
                "you/herdr-mihi",
                "--ref",
                "lazygit-latest",
                "--yes"
            ]
        );
        assert_eq!(
            command_args("you/herdr-mihi", &p, &Action::Uninstall),
            vec!["plugin", "uninstall", "herdr-mihi.lazygit"]
        );
    }

    #[test]
    fn parse_installed_reads_id_and_version() {
        let json = r#"{"result":{"plugins":[{"plugin_id":"herdr-mihi.lazygit","version":"0.1.0"},{"plugin_id":"herdr-mihi.auto-title","version":"0.2.0"}]}}"#;
        let m = parse_installed(json);
        assert_eq!(
            m.get("herdr-mihi.lazygit").map(String::as_str),
            Some("0.1.0")
        );
        assert_eq!(
            m.get("herdr-mihi.auto-title").map(String::as_str),
            Some("0.2.0")
        );
        assert_eq!(m.len(), 2);
    }
}
