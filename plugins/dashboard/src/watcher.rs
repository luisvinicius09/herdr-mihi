//! The background watcher: poll `session.snapshot`, diff each agent's status against the last
//! poll, and on a transition **into** a configured state (blocked/done) raise a herdr toast.
//! Also stamps each agent's (status, since) to `STATE_DIR/agents.tsv` so the popup can show
//! durations. Self-supervising (`daemon.pid`); read-only except the toast; no OS-notify.
use crate::config::Config;
use crate::model::{self, Status};
use crate::shared::sanitize;
use crate::shared::snapshot::AgentInfo;
use crate::shared::socket::Client;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const COOLDOWN_MS: u64 = 20_000; // per-pane minimum gap between alerts

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn run() {
    let cfg = Config::load();
    if !cfg.watcher_on {
        return;
    }
    let sock = std::env::var("HERDR_SOCKET_PATH").unwrap_or_default();
    if sock.is_empty() {
        return;
    }
    let state_dir = std::env::var("HERDR_PLUGIN_STATE_DIR")
        .unwrap_or_else(|_| "/tmp/herdr-mihi-dashboard".to_string());
    let _ = std::fs::create_dir_all(&state_dir);
    let pidfile = format!("{state_dir}/daemon.pid");
    let agents_file = format!("{state_dir}/agents.tsv");
    let pid = std::process::id().to_string();
    let _ = std::fs::write(&pidfile, &pid);

    let mut client = Client::new(sock);
    let mut since: HashMap<String, (String, u64)> = read_since(&agents_file); // pane -> (status, since_ms)
    let mut last_alert: HashMap<String, u64> = HashMap::new();
    let mut backoff = 1u64;
    let mut first = true; // baseline the first poll silently (no startup toast storm)

    loop {
        if let Ok(cur) = std::fs::read_to_string(&pidfile) {
            if cur.trim() != pid {
                break;
            }
        }

        let raw = match client.snapshot_raw() {
            Ok(v) => v,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(200 * backoff));
                backoff = (backoff * 2).min(30);
                continue;
            }
        };
        let snap = match model::snapshot_from_raw(&raw) {
            Some(s) => s,
            None => {
                std::thread::sleep(Duration::from_millis(200 * backoff));
                backoff = (backoff * 2).min(30);
                continue;
            }
        };
        backoff = 1;

        let ws: HashMap<&str, &str> = snap
            .workspaces
            .iter()
            .map(|w| (w.workspace_id.as_str(), w.label.as_str()))
            .collect();
        let now = now_ms();

        let mut seen: HashMap<String, (String, u64)> = HashMap::new();
        let mut fired: Vec<(&AgentInfo, Status)> = Vec::new();
        for a in &snap.agents {
            let st = Status::parse(a.agent_status.as_deref());
            let label = st.label().to_string();
            let prev = since.get(&a.pane_id);
            let changed = prev.map(|(pl, _)| pl.as_str()) != Some(label.as_str());
            let since_ms = match prev {
                Some((pl, ms)) if *pl == label => *ms, // same state: keep the original stamp
                _ => now,
            };
            seen.insert(a.pane_id.clone(), (label.clone(), since_ms));

            if !first && changed && cfg.alert_on.iter().any(|s| s == &label) {
                let ok = last_alert
                    .get(&a.pane_id)
                    .map(|t| now.saturating_sub(*t) >= COOLDOWN_MS)
                    .unwrap_or(true);
                if ok {
                    fired.push((a, st));
                }
            }
        }

        // One detailed toast for a single transition, one summary toast for several.
        match fired.len() {
            0 => {}
            1 => notify_one(&mut client, &cfg, fired[0].0, &ws, fired[0].1),
            _ => notify_summary(&mut client, &cfg, &fired),
        }
        for (a, _) in &fired {
            last_alert.insert(a.pane_id.clone(), now);
        }

        write_since(&agents_file, &seen);
        since = seen;
        first = false;
        std::thread::sleep(Duration::from_millis(cfg.poll_ms));
    }
}

fn read_since(path: &str) -> HashMap<String, (String, u64)> {
    let mut m = HashMap::new();
    if let Ok(c) = std::fs::read_to_string(path) {
        for line in c.lines() {
            let mut it = line.split('\t');
            if let (Some(pane), Some(status), Some(ms)) = (it.next(), it.next(), it.next()) {
                if let Ok(ms) = ms.trim().parse::<u64>() {
                    m.insert(pane.to_string(), (status.to_string(), ms));
                }
            }
        }
    }
    m
}

fn write_since(path: &str, m: &HashMap<String, (String, u64)>) {
    let mut s = String::new();
    for (pane, (status, ms)) in m {
        s.push_str(pane);
        s.push('\t');
        s.push_str(status);
        s.push('\t');
        s.push_str(&ms.to_string());
        s.push('\n');
    }
    let _ = std::fs::write(path, s);
}

fn agent_name(a: &AgentInfo) -> String {
    a.display_agent
        .clone()
        .or_else(|| a.agent.clone())
        .or_else(|| a.name.clone())
        .map(|s| sanitize::clean(&s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent".to_string())
}

fn notify_one(
    client: &mut Client,
    cfg: &Config,
    a: &AgentInfo,
    ws: &HashMap<&str, &str>,
    st: Status,
) {
    let agent = agent_name(a);
    let activity = a
        .title
        .clone()
        .filter(|s| sanitize::meaningful(s))
        .or_else(|| {
            a.terminal_title_stripped
                .clone()
                .filter(|s| sanitize::meaningful(s))
        })
        .map(|s| sanitize::clean(&s))
        .unwrap_or_default();
    let wslabel = ws
        .get(a.workspace_id.as_str())
        .copied()
        .filter(|s| !s.is_empty())
        .unwrap_or("");

    let title = format!("{agent} — {}", st.label().to_uppercase());
    let mut body = wslabel.to_string();
    if !activity.is_empty() {
        body = if body.is_empty() {
            activity
        } else {
            format!("{body} · {activity}")
        };
    }
    let _ = client.notify(&title, &body, &cfg.alert_sound);
}

fn notify_summary(client: &mut Client, cfg: &Config, fired: &[(&AgentInfo, Status)]) {
    let (mut b, mut d, mut w) = (0, 0, 0);
    for (_, st) in fired {
        match st {
            Status::Blocked => b += 1,
            Status::Done => d += 1,
            Status::Working => w += 1,
            _ => {}
        }
    }
    let mut parts = Vec::new();
    if b > 0 {
        parts.push(format!("{b} blocked"));
    }
    if d > 0 {
        parts.push(format!("{d} done"));
    }
    if w > 0 {
        parts.push(format!("{w} working"));
    }
    let title = format!("{} agents updated", fired.len());
    let _ = client.notify(&title, &parts.join(" · "), &cfg.alert_sound);
}
