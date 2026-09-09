//! Config from `$HERDR_PLUGIN_CONFIG_DIR/.env` (fallback `./.env` for dev), env vars override.
//! Read once at startup. No dotenv crate — a tiny hand-rolled KEY=VALUE parser.
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub poll_ms: u64,
    pub max_length: usize,
    pub debug: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            poll_ms: 500,
            max_length: 50,
            debug: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let mut file: HashMap<String, String> = HashMap::new();
        let candidates = [
            std::env::var("HERDR_PLUGIN_CONFIG_DIR")
                .ok()
                .map(|d| format!("{d}/.env")),
            Some(".env".to_string()),
        ];
        for path in candidates.into_iter().flatten() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for raw in content.lines() {
                    let line = raw.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = line.split_once('=') {
                        file.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
                break; // first existing file wins
            }
        }
        let get = |k: &str| std::env::var(k).ok().or_else(|| file.get(k).cloned());
        let d = Config::default();
        Config {
            poll_ms: get("POLL_MS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(d.poll_ms),
            max_length: get("MAX_LENGTH")
                .and_then(|s| s.parse().ok())
                .unwrap_or(d.max_length),
            debug: get("DEBUG")
                .map(|s| !matches!(s.to_lowercase().as_str(), "0" | "false" | "no" | "off"))
                .unwrap_or(d.debug),
        }
    }
}
