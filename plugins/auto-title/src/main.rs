//! herdr-mihi.auto-title — keeps each tab's title in sync with the work in it.
//! `[[startup]]` runs the daemon (no args); `[[actions]]` `reset` runs `main reset`.
//! Polls session.snapshot over herdr's socket; renames drifted tabs via tab.rename.
//! No subprocess, no network, no LLM. macOS + Linux.
mod config;
mod resolver;
mod sanitize;
mod snapshot;
mod socket;

use std::collections::{HashMap, HashSet};
use std::io::Write as _;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A label herdr assigned by default (empty or the tab number) — safe for us to replace.
fn looks_default(label: &str) -> bool {
    let t = label.trim();
    t.is_empty() || t.chars().all(|c| c.is_ascii_digit())
}

/// Append a diagnostic line to <log_dir>/auto-title.log (best-effort; never panics).
fn log(log_dir: &str, msg: &str) {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{log_dir}/auto-title.log"))
    {
        let _ = writeln!(f, "{ts} {msg}");
    }
}

/// Load our tab_id -> title ownership map so a restart re-adopts titles it set before.
fn load_owned(path: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Some((k, v)) = line.split_once('\t') {
                m.insert(k.to_string(), v.to_string());
            }
        }
    }
    m
}

fn save_owned(path: &str, m: &HashMap<String, String>) {
    let mut s = String::new();
    for (k, v) in m {
        s.push_str(k);
        s.push('\t');
        s.push_str(v);
        s.push('\n');
    }
    let _ = std::fs::write(path, s);
}

/// `reset` action: ask the daemon to re-adopt a tab the user renamed by hand.
/// Writes the focused tab id (or `*` for all, when invoked without a tab context)
/// to a control file the daemon consumes on its next poll.
fn cmd_reset(state_dir: &str, log_dir: &str) {
    let ctx = std::env::var("HERDR_PLUGIN_CONTEXT_JSON").unwrap_or_default();
    let tab_id = serde_json::from_str::<serde_json::Value>(&ctx)
        .ok()
        .and_then(|v| v.get("tab_id").and_then(|t| t.as_str()).map(String::from));
    let target = tab_id.clone().unwrap_or_else(|| "*".to_string());
    let _ = std::fs::create_dir_all(state_dir);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{state_dir}/reset.request"))
    {
        let _ = writeln!(f, "{target}");
    }
    match tab_id {
        Some(t) => println!("auto-title: reset requested for tab {t}"),
        None => println!("auto-title: reset requested for all tabs"),
    }
    log(log_dir, &format!("reset requested: {target}"));
}

fn main() {
    let cfg = config::Config::load();

    // Diagnostics go to the config dir (findable via `herdr plugin config-dir`), else state/tmp.
    let log_dir = std::env::var("HERDR_PLUGIN_CONFIG_DIR")
        .ok()
        .or_else(|| std::env::var("HERDR_PLUGIN_STATE_DIR").ok())
        .unwrap_or_else(|| "/tmp".to_string());
    let state_dir = std::env::var("HERDR_PLUGIN_STATE_DIR")
        .unwrap_or_else(|_| "/tmp/herdr-mihi-auto-title".to_string());
    let _ = std::fs::create_dir_all(&state_dir);

    // Action dispatch (short-lived), else fall through to the daemon.
    if std::env::args().nth(1).as_deref() == Some("reset") {
        cmd_reset(&state_dir, &log_dir);
        return;
    }

    let sock = std::env::var("HERDR_SOCKET_PATH").unwrap_or_default();
    log(
        &log_dir,
        &format!(
            "started: socket_set={} poll_ms={} max_len={} debug={}",
            !sock.is_empty(),
            cfg.poll_ms,
            cfg.max_length,
            cfg.debug
        ),
    );
    if sock.is_empty() {
        log(&log_dir, "HERDR_SOCKET_PATH not set; exiting");
        std::process::exit(0);
    }

    let pidfile = format!("{state_dir}/daemon.pid");
    let pid = std::process::id().to_string();
    let _ = std::fs::write(&pidfile, &pid);
    let owned_path = format!("{state_dir}/owned.tsv");
    let reset_path = format!("{state_dir}/reset.request");

    let mut client = socket::Client::new(sock);
    let mut mine: HashMap<String, String> = load_owned(&owned_path); // tab_id -> title we set
    let mut manual: HashSet<String> = HashSet::new(); // tabs the user renamed — hands off
    let mut backoff = 1u64;
    let mut logged_raw = false;

    loop {
        if let Ok(cur) = std::fs::read_to_string(&pidfile) {
            if cur.trim() != pid {
                log(&log_dir, "superseded by a newer instance; exiting");
                break;
            }
        }

        let raw = match client.snapshot_raw() {
            Ok(v) => v,
            Err(e) => {
                log(&log_dir, &format!("snapshot connect/read error: {e}"));
                std::thread::sleep(Duration::from_millis(200 * backoff));
                backoff = (backoff * 2).min(30);
                continue;
            }
        };
        if let Some(err) = raw.get("error") {
            log(&log_dir, &format!("snapshot returned error: {err}"));
            std::thread::sleep(Duration::from_millis(200 * backoff));
            backoff = (backoff * 2).min(30);
            continue;
        }
        if cfg.debug && !logged_raw {
            let s: String = raw.to_string().chars().take(4000).collect();
            log(&log_dir, &format!("first raw snapshot: {s}"));
            logged_raw = true;
        }

        let result = raw
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        // herdr nests the session under result.snapshot; fall back to result itself.
        let snap_val = result.get("snapshot").cloned().unwrap_or(result);
        let snap: snapshot::SessionSnapshot = match serde_json::from_value(snap_val) {
            Ok(s) => s,
            Err(e) => {
                log(&log_dir, &format!("snapshot parse error: {e}"));
                std::thread::sleep(Duration::from_millis(cfg.poll_ms));
                continue;
            }
        };
        backoff = 1;

        // Consume any `reset` requests: force these tabs to be re-adopted this poll.
        let mut force: HashSet<String> = HashSet::new();
        let mut force_all = false;
        if let Ok(content) = std::fs::read_to_string(&reset_path) {
            for line in content.lines() {
                match line.trim() {
                    "" => {}
                    "*" => force_all = true,
                    t => {
                        force.insert(t.to_string());
                    }
                }
            }
            let _ = std::fs::remove_file(&reset_path);
            if force_all {
                manual.clear();
            } else {
                for t in &force {
                    manual.remove(t);
                }
            }
            if cfg.debug {
                log(
                    &log_dir,
                    &format!("reset applied: all={force_all} tabs={}", force.len()),
                );
            }
        }

        let mut by_tab: HashMap<&str, Vec<&snapshot::PaneInfo>> = HashMap::new();
        for p in &snap.panes {
            by_tab.entry(p.tab_id.as_str()).or_default().push(p);
        }
        let ws_label: HashMap<&str, &str> = snap
            .workspaces
            .iter()
            .map(|w| (w.workspace_id.as_str(), w.label.as_str()))
            .collect();

        for tab in &snap.tabs {
            let forced = force_all || force.contains(&tab.tab_id);
            if !forced {
                if manual.contains(&tab.tab_id) {
                    continue;
                }
                match mine.get(&tab.tab_id) {
                    Some(set) if &tab.label == set => {} // ours, applied — refresh below
                    Some(_) => {
                        manual.insert(tab.tab_id.clone());
                        if cfg.debug {
                            log(
                                &log_dir,
                                &format!(
                                    "tab {} '{}': user renamed, backing off",
                                    tab.tab_id, tab.label
                                ),
                            );
                        }
                        continue;
                    }
                    None if !looks_default(&tab.label) && !tab.label.contains(" › ") => {
                        manual.insert(tab.tab_id.clone());
                        if cfg.debug {
                            log(
                                &log_dir,
                                &format!(
                                    "tab {} '{}': pre-existing custom name, leaving",
                                    tab.tab_id, tab.label
                                ),
                            );
                        }
                        continue;
                    }
                    None => {}
                }
            }

            let empty: Vec<&snapshot::PaneInfo> = Vec::new();
            let panes = by_tab.get(tab.tab_id.as_str()).unwrap_or(&empty);
            let wslabel = ws_label.get(tab.workspace_id.as_str()).copied();
            let title =
                resolver::resolve(tab, panes, snap.focused_pane_id.as_deref(), wslabel, &cfg);

            if title == tab.label {
                mine.insert(tab.tab_id.clone(), title); // own it, no rename needed
            } else if title != "Shell" {
                match client.rename(&tab.tab_id, &title) {
                    Ok(()) => {
                        if cfg.debug {
                            log(
                                &log_dir,
                                &format!("tab {} renamed -> '{}'", tab.tab_id, title),
                            );
                        }
                        mine.insert(tab.tab_id.clone(), title);
                        save_owned(&owned_path, &mine);
                    }
                    Err(e) => {
                        log(&log_dir, &format!("tab {} rename failed: {e}", tab.tab_id));
                    }
                }
            }
        }

        std::thread::sleep(Duration::from_millis(cfg.poll_ms));
    }
}
