#![allow(dead_code)] // shared module: not every consumer uses every method
//! Minimal herdr socket client: newline-delimited JSON, one request per connection
//! (herdr closes the connection after answering). macOS + Linux unix socket only.
//! Canonical: `shared/rust/socket.rs` — synced into each plugin, CI drift-checked.
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;

pub struct Client {
    path: String,
    seq: u64,
}

impl Client {
    pub fn new(path: String) -> Self {
        Client { path, seq: 0 }
    }

    /// One request → the raw response envelope (`{"id":…,"result":…}` or `{"id":…,"error":…}`),
    /// so the caller can log/parse it. Each request is its own short-lived connection.
    pub fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> std::io::Result<serde_json::Value> {
        self.seq += 1;
        let req = serde_json::json!({
            "id": format!("herdr-mihi:{}", self.seq),
            "method": method,
            "params": params,
        });
        let mut stream = UnixStream::connect(&self.path)?;
        let mut line = serde_json::to_string(&req).unwrap_or_default();
        line.push('\n');
        stream.write_all(line.as_bytes())?;
        stream.flush()?;

        let mut reader = BufReader::new(stream);
        let mut resp = String::new();
        reader.read_line(&mut resp)?;
        serde_json::from_str::<serde_json::Value>(resp.trim())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Fetch the whole session in one request; returns the raw response envelope.
    pub fn snapshot_raw(&mut self) -> std::io::Result<serde_json::Value> {
        self.call("session.snapshot", serde_json::json!({}))
    }

    /// Set a tab's label (auto-title).
    pub fn rename(&mut self, tab_id: &str, label: &str) -> std::io::Result<()> {
        self.ok_or_err(
            "tab.rename",
            serde_json::json!({ "tab_id": tab_id, "label": label }),
        )
    }

    /// Focus the agent in a pane — herdr's `agent focus` takes a **pane id** (`w:p`),
    /// never a terminal id (dashboard "jump"). Marks the agent "seen".
    pub fn agent_focus(&mut self, pane_id: &str) -> std::io::Result<()> {
        self.ok_or_err("agent.focus", serde_json::json!({ "target": pane_id }))
    }

    /// Raise a user notification through herdr's configured toast delivery.
    /// herdr honors `ui.toast.delivery` (so `off` disables it) and rate-limits.
    pub fn notify(
        &mut self,
        title: &str,
        body: &str,
        sound: &str,
    ) -> std::io::Result<serde_json::Value> {
        self.call(
            "notification.show",
            serde_json::json!({ "title": title, "body": body, "sound": sound }),
        )
    }

    fn ok_or_err(&mut self, method: &str, params: serde_json::Value) -> std::io::Result<()> {
        let v = self.call(method, params)?;
        if let Some(err) = v.get("error") {
            return Err(std::io::Error::other(format!("{method} error: {err}")));
        }
        Ok(())
    }
}
