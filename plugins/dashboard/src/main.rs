//! herdr-mihi.dashboard — a non-disruptive agent-status dashboard.
//! `[[startup]]` runs `watch` (the toast watcher); `open` opens the popup; `pane` runs the TUI;
//! `setup` binds a key. Read-only over herdr's socket; the only mutation is an explicit jump.
//! No network, no OS-notify subprocess, no LLM. macOS + Linux.
mod config;
mod effects;
mod model;
mod shared; // canonical modules synced from shared/rust/ (see just sync-shared)
mod tui;
mod watcher;

use config::Config;
use std::io::Write as _;
use std::process::Command;

fn herdr_bin() -> String {
    std::env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string())
}

/// `open` action: open the dashboard popup pane.
fn cmd_open() {
    let cfg = Config::load();
    let _ = Command::new(herdr_bin())
        .args([
            "plugin",
            "pane",
            "open",
            "--plugin",
            "herdr-mihi.dashboard",
            "--entrypoint",
            "dashboard",
            "--placement",
            "popup",
            "--width",
            &cfg.popup_width,
            "--height",
            &cfg.popup_height,
            "--focus",
        ])
        .status();
}

fn herdr_config_path() -> String {
    let base = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
    format!("{base}/herdr/config.toml")
}

/// Opt-in `setup` action: append the open keybind to the user's herdr config.toml (idempotent).
fn cmd_setup() {
    let action = "herdr-mihi.dashboard.open";
    let key = "prefix+d";
    let herdr = herdr_bin();
    let cfg = herdr_config_path();
    let content = match std::fs::read_to_string(&cfg) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("herdr-mihi.dashboard: herdr config not found at {cfg}");
            std::process::exit(1);
        }
    };
    if content.contains(&format!("invoke {action}")) {
        println!("herdr-mihi.dashboard: keybind already configured in {cfg}");
        return;
    }
    let block = format!(
        "\n# --- herdr-mihi.dashboard (added by: herdr plugin action invoke herdr-mihi.dashboard.setup) ---\n\
         [[keys.command]]\nkey = \"{key}\"\ntype = \"shell\"\ncommand = \"{herdr} plugin action invoke {action}\"\n"
    );
    if content.contains(&format!("\"{key}\"")) {
        eprintln!("herdr-mihi.dashboard: key '{key}' is already bound — add this yourself with a free key:");
        eprintln!("{block}");
        return;
    }
    let write = std::fs::OpenOptions::new()
        .append(true)
        .open(&cfg)
        .and_then(|mut f| f.write_all(block.as_bytes()));
    if let Err(e) = write {
        eprintln!("herdr-mihi.dashboard: cannot write {cfg}: {e}");
        std::process::exit(1);
    }
    println!("herdr-mihi.dashboard: added keybind '{key}' -> {action} in {cfg}");
    let reloaded = Command::new(&herdr)
        .args(["server", "reload-config"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if reloaded {
        println!("reloaded — press your prefix then 'd' to open the dashboard");
    } else {
        println!("run 'herdr server reload-config' to apply");
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("watch") => watcher::run(),
        Some("open") => cmd_open(),
        Some("pane") => tui::run(),
        Some("setup") => cmd_setup(),
        _ => {
            eprintln!("usage: run.sh {{watch|open|pane|setup}}");
            std::process::exit(2);
        }
    }
}
