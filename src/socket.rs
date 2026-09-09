//! Minimal herdr socket client: newline-delimited JSON, one request per connection
//! (herdr closes the connection after answering). macOS + Linux unix socket only.
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

    fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> std::io::Result<serde_json::Value> {
        self.seq += 1;
        let req = serde_json::json!({
            "id": format!("auto-title:{}", self.seq),
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

    /// Fetch the whole session in one request; returns the raw response envelope
    /// ({"id":…, "result":…} or {"id":…, "error":…}) so the caller can log/parse it.
    pub fn snapshot_raw(&mut self) -> std::io::Result<serde_json::Value> {
        self.request("session.snapshot", serde_json::json!({}))
    }

    /// Set a tab's label.
    pub fn rename(&mut self, tab_id: &str, label: &str) -> std::io::Result<()> {
        let v = self.request(
            "tab.rename",
            serde_json::json!({ "tab_id": tab_id, "label": label }),
        )?;
        if let Some(err) = v.get("error") {
            return Err(std::io::Error::other(format!("tab.rename error: {err}")));
        }
        Ok(())
    }
}
