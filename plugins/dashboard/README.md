# dashboard

**Platforms: macOS · Linux**

A **non-disruptive agent-status dashboard** for herdr — a place to go while your agents work that shows
their status and tells you when to come back, **without disturbing them**. Two parts:

- a background **watcher** that raises a herdr toast when an agent goes **blocked** (needs you) or
  **done** (finished while you were away);
- an on-demand **popup** that lists every agent, attention-first, and lets you **jump** to one.

## Install
```sh
herdr plugin install luisvinicius09/herdr-mihi --ref dashboard-latest
```

## Behavior
- **Watcher** (`[[startup]]`): polls `session.snapshot`, and on a transition **into** `blocked`/`done`
  raises `notification.show` (title `<agent> — BLOCKED/DONE`, body `<workspace> · <activity>`). It honors
  your `ui.toast.delivery` (so `off` disables toasts) and herdr's rate-limiting — **no OS-notify
  subprocess**. Per-pane cooldown avoids spam; several transitions in one poll collapse into a single
  summary toast. It also stamps each agent's state+since to `STATE_DIR/agents.tsv` (powers durations).
  Which transitions + the sound are tunable in `.env`; set `WATCHER=off` to disable it.
- **Popup** (`open` action): an ambient **agent screen** — a centered card floating in a native
  **effect field** (screensaver-style). Top: a **"▲ N need you" hero** + counts. Body: each agent is a
  **bordered tile** (its border glows in the status color for blocked/done), sized to the panel:
  - **top** — status glyph · agent name · `· STATUS dur` (e.g. `· BLOCKED 4m`);
  - **middle** — what it's doing;
  - **meta** — **project ⎇branch** + **sync** (`✓` level with upstream / `↕` diverged) + **git state**
    (`⚠ rebasing`/`merging`/`cherry-pick`/…) + **stash** (`⌥2`) + token chips + the pane id;
  - **footer** — the **workspace**, right-aligned on the box.

  Tiles are ordered by workspace then attention. It **auto-refreshes** (`REFRESH_MS`) and **dims after
  `IDLE_DIM_S`** of no keypress (true screensaver). Composable panel stack — more panels slot in.
- **Navigate:** `↑/↓`(`j/k`) move · **`⏎` jump** · **`⇥` next attention** (hop to the next blocked/done)
  · **`/` filter** (type; enter/esc to apply) · **`a`** attention-only · **`s`** sort
  (attention/recent/name) · **`e`** effect · `r` refresh · `q`/`Esc` close.
- **Effects:** **`e`** cycles the field — `none / rain / stars / beam / snow / wave / life / fireworks`
  (remembered). Event flair: a **fireworks burst when an agent finishes**, a **red flash when one
  blocks**, over whatever ambient effect is running. All **native Rust** — no python/subprocess/network.
- **Theme:** by default it **matches your herdr `[theme]`** (reads the theme *name* from your herdr
  config and maps it to a palette — dracula, tokyonight, catppuccin, nord, gruvbox, solarized, onedark,
  rose-pine, kanagawa, everforest; unknown → herdr blue). Override with `THEME` / `CANVAS_BG` / `ACCENT`.
- **Jump = `herdr agent focus <pane_id>`** — the *only* thing that changes anything, and only on your
  explicit Enter. herdr treats a focus as "seen" (so jumping to a `done` agent clears its done marker);
  merely opening/reading/filtering the list marks nothing.

## Keybinding
**Quickest:** `herdr plugin action invoke herdr-mihi.dashboard.setup` adds the binding (default
`prefix+d`) to your `config.toml` and reloads — idempotent, warns instead of clobbering if the key is taken.

Or by hand — herdr binds no keys by default; add to your `config.toml` then `herdr server reload-config`:
```toml
[[keys.command]]
key = "prefix+d"
command = "herdr plugin action invoke herdr-mihi.dashboard.open"
```

## Capability declaration (trust)
- **Reads only:** `session.snapshot` (agent status/roster) over herdr's socket. Never reads scrollback,
  transcripts, or files beyond its own config/state.
- **Writes:** the **only** state change is an explicit **jump** (`agent.focus`, on your Enter). Alerts go
  through herdr's own `notification.show` — **no OS-notify subprocess, no network, no telemetry**.
- **Spawns:** the `herdr` CLI only (fallback for the socket). No other subprocess.
- **Files:** reads `$HERDR_PLUGIN_CONFIG_DIR/.env`, your herdr `config.toml` (only `[theme] name` +
  `[ui] status_indicators`, read-only, to match its colors + glyphs), and each agent's **`.git` files** (`HEAD`, `refs`/`packed-refs`, `config`,
  `logs/refs/stash`, and in-progress markers) for branch/sync/state/stash — **read-only, never shells
  out to `git`**. (Dirty-tree status is intentionally omitted; it would need a `git` subprocess.) Writes
  only `$HERDR_PLUGIN_STATE_DIR` (daemon pid, chosen effect, per-pane state+since). **No** host-config
  mutation, **no** bundled font.
- All agent-derived text is sanitized as hostile input (ANSI/control/bidi stripped, column-truncated).

## Requirements
`herdr ≥ 0.8` (needs `notification.show`, `agent focus` by pane id, `session.snapshot`). macOS + Linux.

## Config (`.env`)
See `.env.example`: `THEME` (auto/named/herdr-mihi), `CANVAS_BG` + `ACCENT` (auto or hex override),
`POPUP_WIDTH/HEIGHT`, `REFRESH_MS` (auto-refresh cadence; 0 = manual), `EFFECT`
(none/rain/stars/beam/snow/wave/life/fireworks), `FRAME_MS` (animation speed), `IDLE_DIM_S` (dim after
N idle seconds; 0 = never), `WATCHER`, `ALERT_ON`, `ALERT_SOUND`, `POLL_MS`, `MAX_ROWS`, `ICONS`
(`auto` mirrors herdr's `ui.status_indicators` — symbols/dots — or force symbols/dots/nerd/unicode/ascii).

## Dependencies (audited)
`ratatui` (TUI), `serde`/`serde_json` (snapshot), `unicode-width` (column truncation). No networking crates.
Shares the socket/snapshot/sanitize modules with auto-title via the collection's canonical `shared/rust/`.
