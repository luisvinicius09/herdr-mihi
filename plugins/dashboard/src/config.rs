//! Config from `$HERDR_PLUGIN_CONFIG_DIR/.env` (env vars override; a bad value keeps the default).
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct Icons {
    pub blocked: &'static str,
    pub done: &'static str,
    pub working: &'static str,
    pub idle: &'static str,
    pub unknown: &'static str,
}

pub fn icons(kind: &str) -> Icons {
    match kind {
        // herdr's own glyphs (src/client/shell.rs `status_icon`): match its UI exactly.
        "symbols" => Icons {
            blocked: "×",
            done: "✓",
            working: "◐",
            idle: "○",
            unknown: "·",
        },
        "dots" => Icons {
            blocked: "●",
            done: "●",
            working: "●",
            idle: "○",
            unknown: "·",
        },
        "ascii" => Icons {
            blocked: "!",
            done: "*",
            working: ">",
            idle: "-",
            unknown: "?",
        },
        "unicode" => Icons {
            blocked: "●",
            done: "✓",
            working: "◐",
            idle: "○",
            unknown: "·",
        },
        "nerd" => Icons {
            blocked: "\u{f071}", // warning triangle
            done: "\u{f00c}",    // check
            working: "\u{f021}", // refresh
            idle: "\u{f10c}",    // circle-o
            unknown: "\u{f128}", // question
        },
        // fallback = herdr's default ("dots")
        _ => Icons {
            blocked: "●",
            done: "●",
            working: "●",
            idle: "○",
            unknown: "·",
        },
    }
}

/// Read herdr's `[ui] status_indicators` ("dots" | "symbols") so we can mirror its glyphs.
fn herdr_status_indicators() -> Option<String> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
    let content = std::fs::read_to_string(format!("{base}/herdr/config.toml")).ok()?;
    let mut in_ui = false;
    for line in content.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_ui = l == "[ui]";
            continue;
        }
        if in_ui {
            if let Some(rest) = l.strip_prefix("status_indicators") {
                if let Some((_, v)) = rest.split_once('=') {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

/// Resolve the icon set: "auto" mirrors herdr's `status_indicators`; else a named set.
fn resolve_icons(sel: &str) -> Icons {
    match sel.trim().to_lowercase().as_str() {
        "auto" => icons(&herdr_status_indicators().unwrap_or_else(|| "dots".to_string())),
        other => icons(other),
    }
}

/// A resolved color palette (RGB triples; the TUI converts to ratatui colors).
#[derive(Clone, Copy)]
pub struct Theme {
    pub accent: (u8, u8, u8),
    pub canvas: (u8, u8, u8),
    pub blocked: (u8, u8, u8),
    pub done: (u8, u8, u8),
    pub working: (u8, u8, u8),
    pub idle: (u8, u8, u8),
    pub dim: (u8, u8, u8),
}

/// The herdr-mihi default (herdr blue on navy) — also the fallback for unknown themes.
fn default_theme() -> Theme {
    Theme {
        accent: (0x07, 0x97, 0xff),
        canvas: (0x0d, 0x15, 0x26),
        blocked: (0xff, 0x5c, 0x5c),
        done: (0x4a, 0xde, 0x80),
        working: (0xfb, 0xbf, 0x24),
        idle: (0x5a, 0x66, 0x78),
        dim: (0x5a, 0x66, 0x78),
    }
}

/// Curated palettes for popular herdr/terminal themes, keyed by herdr's `[theme] name`.
pub fn theme_for(name: &str) -> Theme {
    let key = name.trim().to_lowercase().replace([' ', '_'], "-");
    match key.as_str() {
        "dracula" => Theme {
            accent: (0xbd, 0x93, 0xf9),
            canvas: (0x28, 0x2a, 0x36),
            blocked: (0xff, 0x55, 0x55),
            done: (0x50, 0xfa, 0x7b),
            working: (0xf1, 0xfa, 0x8c),
            idle: (0x62, 0x72, 0xa4),
            dim: (0x62, 0x72, 0xa4),
        },
        "tokyonight" | "tokyo-night" => Theme {
            accent: (0x7a, 0xa2, 0xf7),
            canvas: (0x1a, 0x1b, 0x26),
            blocked: (0xf7, 0x76, 0x8e),
            done: (0x9e, 0xce, 0x6a),
            working: (0xe0, 0xaf, 0x68),
            idle: (0x56, 0x5f, 0x89),
            dim: (0x56, 0x5f, 0x89),
        },
        "catppuccin" | "catppuccin-mocha" | "mocha" => Theme {
            accent: (0xcb, 0xa6, 0xf7),
            canvas: (0x1e, 0x1e, 0x2e),
            blocked: (0xf3, 0x8b, 0xa8),
            done: (0xa6, 0xe3, 0xa1),
            working: (0xf9, 0xe2, 0xaf),
            idle: (0x6c, 0x70, 0x86),
            dim: (0x6c, 0x70, 0x86),
        },
        "nord" => Theme {
            accent: (0x88, 0xc0, 0xd0),
            canvas: (0x2e, 0x34, 0x40),
            blocked: (0xbf, 0x61, 0x6a),
            done: (0xa3, 0xbe, 0x8c),
            working: (0xeb, 0xcb, 0x8b),
            idle: (0x4c, 0x56, 0x6a),
            dim: (0x4c, 0x56, 0x6a),
        },
        "gruvbox" | "gruvbox-dark" => Theme {
            accent: (0x83, 0xa5, 0x98),
            canvas: (0x28, 0x28, 0x28),
            blocked: (0xfb, 0x49, 0x34),
            done: (0xb8, 0xbb, 0x26),
            working: (0xfa, 0xbd, 0x2f),
            idle: (0x92, 0x83, 0x74),
            dim: (0x92, 0x83, 0x74),
        },
        "solarized" | "solarized-dark" => Theme {
            accent: (0x26, 0x8b, 0xd2),
            canvas: (0x00, 0x2b, 0x36),
            blocked: (0xdc, 0x32, 0x2f),
            done: (0x85, 0x99, 0x00),
            working: (0xb5, 0x89, 0x00),
            idle: (0x58, 0x6e, 0x75),
            dim: (0x58, 0x6e, 0x75),
        },
        "onedark" | "one-dark" => Theme {
            accent: (0x61, 0xaf, 0xef),
            canvas: (0x28, 0x2c, 0x34),
            blocked: (0xe0, 0x6c, 0x75),
            done: (0x98, 0xc3, 0x79),
            working: (0xe5, 0xc0, 0x7b),
            idle: (0x5c, 0x63, 0x70),
            dim: (0x5c, 0x63, 0x70),
        },
        "rose-pine" | "rosepine" => Theme {
            accent: (0xc4, 0xa7, 0xe7),
            canvas: (0x19, 0x17, 0x24),
            blocked: (0xeb, 0x6f, 0x92),
            done: (0x9c, 0xcf, 0xd8),
            working: (0xf6, 0xc1, 0x77),
            idle: (0x6e, 0x6a, 0x86),
            dim: (0x6e, 0x6a, 0x86),
        },
        "kanagawa" => Theme {
            accent: (0x7e, 0x9c, 0xd8),
            canvas: (0x1f, 0x1f, 0x28),
            blocked: (0xe8, 0x24, 0x24),
            done: (0x98, 0xbb, 0x6c),
            working: (0xe6, 0xc3, 0x84),
            idle: (0x72, 0x71, 0x69),
            dim: (0x72, 0x71, 0x69),
        },
        "everforest" => Theme {
            accent: (0xa7, 0xc0, 0x80),
            canvas: (0x2d, 0x35, 0x3b),
            blocked: (0xe6, 0x7e, 0x80),
            done: (0x83, 0xc0, 0x92),
            working: (0xdb, 0xbc, 0x7f),
            idle: (0x85, 0x92, 0x89),
            dim: (0x85, 0x92, 0x89),
        },
        _ => default_theme(),
    }
}

/// Read herdr's active theme name from `~/.config/herdr/config.toml` ([theme] name = "...").
fn herdr_theme_name() -> Option<String> {
    let base = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap_or_default()));
    let content = std::fs::read_to_string(format!("{base}/herdr/config.toml")).ok()?;
    let mut in_theme = false;
    for line in content.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_theme = l == "[theme]";
            continue;
        }
        if in_theme {
            if let Some(rest) = l.strip_prefix("name") {
                if let Some((_, v)) = rest.split_once('=') {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

/// Resolve a theme selection: "auto" matches herdr; a name picks a palette; else the default.
fn resolve_theme(sel: &str) -> Theme {
    match sel.trim().to_lowercase().as_str() {
        "auto" => herdr_theme_name()
            .map(|n| theme_for(&n))
            .unwrap_or_else(default_theme),
        "herdr-mihi" | "herdr" | "default" => default_theme(),
        name => theme_for(name),
    }
}

#[derive(Clone)]
pub struct Config {
    pub popup_width: String,
    pub popup_height: String,
    pub watcher_on: bool,
    pub alert_on: Vec<String>,
    pub alert_sound: String,
    pub poll_ms: u64,
    pub max_rows: usize,
    pub icons: Icons,
    /// Resolved color theme (matched to herdr's `[theme]` unless overridden).
    pub theme: Theme,
    /// Auto-refresh cadence for the open popup, in ms (0 = only on `r`).
    pub refresh_ms: u64,
    /// Initial background effect: none | rain | stars | beam (cycle live with `e`).
    pub effect: String,
    /// Animation frame interval in ms while an effect is active.
    pub frame_ms: u64,
    /// Dim the card after this many seconds without a keypress (0 = never dim).
    pub idle_dim_s: u64,
}

impl Config {
    pub fn load() -> Config {
        let m = load_env();
        Config {
            popup_width: get(&m, "POPUP_WIDTH", "100%"),
            popup_height: get(&m, "POPUP_HEIGHT", "100%"),
            watcher_on: get(&m, "WATCHER", "on") != "off",
            alert_on: get(&m, "ALERT_ON", "blocked,done")
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            alert_sound: get(&m, "ALERT_SOUND", "none"),
            poll_ms: get(&m, "POLL_MS", "1000").parse().unwrap_or(1000),
            max_rows: get(&m, "MAX_ROWS", "200").parse().unwrap_or(200),
            icons: resolve_icons(&get(&m, "ICONS", "auto")),
            theme: {
                let mut t = resolve_theme(&get(&m, "THEME", "auto"));
                let cb = get(&m, "CANVAS_BG", "auto");
                if cb != "auto" {
                    t.canvas = parse_hex(&cb);
                }
                let ac = get(&m, "ACCENT", "auto");
                if ac != "auto" {
                    t.accent = parse_hex(&ac);
                }
                t
            },
            refresh_ms: get(&m, "REFRESH_MS", "2000").parse().unwrap_or(2000),
            effect: get(&m, "EFFECT", "rain"),
            frame_ms: get(&m, "FRAME_MS", "80").parse().unwrap_or(80),
            idle_dim_s: get(&m, "IDLE_DIM_S", "60").parse().unwrap_or(60),
        }
    }
}

/// Parse `#rrggbb` (or `rrggbb`) into an RGB triple; falls back to the default canvas navy.
fn parse_hex(s: &str) -> (u8, u8, u8) {
    let h = s.trim().trim_start_matches('#');
    if h.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&h[0..2], 16),
            u8::from_str_radix(&h[2..4], 16),
            u8::from_str_radix(&h[4..6], 16),
        ) {
            return (r, g, b);
        }
    }
    (0x0d, 0x15, 0x26)
}

fn load_env() -> HashMap<String, String> {
    let mut m = HashMap::new();
    if let Ok(dir) = std::env::var("HERDR_PLUGIN_CONFIG_DIR") {
        if let Ok(c) = std::fs::read_to_string(format!("{dir}/.env")) {
            for line in c.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = l.split_once('=') {
                    m.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    m
}

fn get(m: &HashMap<String, String>, key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .or_else(|| m.get(key).cloned())
        .unwrap_or_else(|| default.to_string())
}
