#![allow(dead_code)] // shared module: not every consumer uses every helper
//! Treat every terminal/agent-derived string as hostile: strip ANSI/OSC escapes, control and
//! bidi/zero-width format chars, collapse whitespace, and truncate by terminal COLUMN width.
//! Canonical: `shared/rust/sanitize.rs` — synced into each plugin, CI drift-checked.
use unicode_width::UnicodeWidthChar;

/// Strip escapes + control/format chars and collapse whitespace to single spaces.
pub fn clean(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // ANSI escape: CSI (ESC [ … final) or OSC (ESC ] … BEL/ST); drop the whole sequence.
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    while let Some(&n) = chars.peek() {
                        chars.next();
                        if n.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(&n) = chars.peek() {
                        chars.next();
                        if n == '\u{07}' {
                            break;
                        }
                        if n == '\u{1b}' {
                            chars.next(); // ST is ESC \
                            break;
                        }
                    }
                }
                _ => {
                    chars.next();
                }
            }
            continue;
        }
        if c.is_control() {
            out.push(' ');
            continue;
        }
        if is_format_char(c) {
            continue; // drop zero-width / bidi controls (keep ZWJ for emoji)
        }
        out.push(c);
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_format_char(c: char) -> bool {
    matches!(c,
        '\u{200b}' | '\u{200c}' | '\u{200e}' | '\u{200f}'
        | '\u{202a}'..='\u{202e}'
        | '\u{2066}'..='\u{2069}'
        | '\u{feff}')
}

/// Truncate to at most `max` terminal columns, appending '…' if it had to cut.
pub fn truncate_cols(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let total: usize = s.chars().map(|c| c.width().unwrap_or(0)).sum();
    if total <= max {
        return s.to_string();
    }
    let mut width = 0;
    let mut out = String::new();
    for c in s.chars() {
        let cw = c.width().unwrap_or(0);
        if width + cw > max.saturating_sub(1) {
            break;
        }
        width += cw;
        out.push(c);
    }
    out.push('…');
    out
}

/// Does this string actually say something? Rejects empties, bare shells/programs, prompts.
pub fn meaningful(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    // a bare filesystem path says nothing new (the cwd is already the context)
    if t.starts_with('/') || t.starts_with('~') {
        return false;
    }
    // an idle shell prompt like `user@host` or `user@host: ~/path` — nothing is happening
    if t.split_whitespace().next().is_some_and(|w| w.contains('@')) {
        return false;
    }
    // bare shell / program names
    !matches!(
        t.to_lowercase().as_str(),
        "bash" | "zsh" | "sh" | "fish" | "-bash" | "-zsh" | "shell" | "node" | "python" | "python3"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi_and_collapses_ws() {
        assert_eq!(clean("\u{1b}[31mred\u{1b}[0m   text"), "red text");
    }

    #[test]
    fn drops_control_and_zero_width() {
        assert_eq!(clean("a\u{200b}b\tc"), "ab c");
    }

    #[test]
    fn truncates_by_width_with_ellipsis() {
        assert_eq!(truncate_cols("abcdef", 4), "abc…");
        assert_eq!(truncate_cols("abc", 10), "abc");
    }

    #[test]
    fn meaningful_rejects_noise() {
        assert!(!meaningful("  "));
        assert!(!meaningful("bash"));
        assert!(!meaningful("luisvinicius@Luiss-MacBook-Air"));
        assert!(!meaningful("luisvinicius@host: ~/coding"));
        assert!(!meaningful("~/coding/herdr-mihi"));
        assert!(!meaningful("/usr/local/bin"));
        assert!(meaningful("editing main.rs"));
    }
}
