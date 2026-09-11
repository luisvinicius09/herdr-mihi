# Brief: agent-status dashboard plugin (`herdr-mihi.dashboard`)

**Status:** Ready to build (2026-09-10). Grounds: ticket 012 (what herdr exposes), the approved
prototype [`prototypes/013-dashboard-mock.md`](../prototypes/013-dashboard-mock.md), 005 (topology/refs),
006 (conventions), 007 (Rust/prebuilt), the **trust-first** principle. The phase-2 flagship.
**Driver:** *a place to go while agents work that shows their status without disrupting them* — and tells
you when to come back. Blocked = "needs you now"; done = "finished while you were away."

## Behavior (v1) — from the approved prototype (shape **C, Hybrid**)

### ① Watcher — self-supervising `[[startup]]` daemon
- Tracks each agent's `agent_status` and raises a herdr **`notification.show`** toast on a transition to
  **`blocked`** ("needs you") or **`done`** ("finished"). Uses herdr's own toast delivery — honors
  `ui.toast.delivery` (so `off` disables it) and herdr's rate-limiting; **no OS-notify subprocess.**
- Toast: `title` = `<agent> · <activity>`, `body` = `<workspace> — BLOCKED/DONE` (herdr trims to 80/240
  and sanitizes). `sound` default `none`.
- **Detects transitions by poll+diff of `session.snapshot`** (the auto-title pattern — version-safe;
  see Refresh). Self-supervises: survives a herdr restart, detects supersession, exits clean.

### ② Popup — on-demand read-only ratatui TUI
- **Action `herdr-mihi.dashboard.open`** → **`popup`** pane (least-disruptive: no focus steal, no pane
  id, no lifecycle events).
- **Full roster, attention-sorted**, grouped by workspace: `blocked → done → working → idle` (ties by
  most-recent `state_change_seq`), idle dimmed.
- **Row:** status glyph · `<agent> › <activity>` (from `display_agent` + `terminal_title_stripped`,
  sanitized as hostile input) · state · age. Glyphs: `● blocked  ✓ done  ◐ working  ○ idle`.
- **Keys:** `↑/↓`(`j/k`) move · **`⏎` jump** · `r` refresh · `q` close.
- **Jump = `herdr agent focus <pane_id>`** — a herdr **pane id (`w:p`), never a terminal id**
  (navigator #13). Jumping is the *only* mutating action and is the intended, explicit way to act.

### Non-disruptive contract (concrete, per 012 §5)
Read-only observation only (`session.snapshot`/`agent.list`; never `send_text/keys/input/prompt`); render
in a `popup` (no focus steal); **only an explicit jump marks-seen** — reading the list never clears a
`done`/attention; no pane/layout mutation of observed agents; no host-config mutation; close-clean.

## Herdr surfaces (manifest shape)
- `[[startup]]` → the watcher (`herdr/run.sh` with no arg, or a `watch` subcommand).
- `[[actions]]` `open` → opens the popup.
- `[[panes]]` `dashboard` (`placement = "popup"`, roomy default, size configurable).
- `[[build]]` = `bash herdr/install.sh` (prebuilt + source fallback, per 007).
- `min_herdr_version` = **0.8.0** (collection standard) — **verify at build** that `notification.show`,
  `agent.focus`-by-pane-id, `session.snapshot`, and `pane.agent_status_changed` all exist at 0.8.0; bump
  the floor if any landed later. Platforms macOS·Linux. id `herdr-mihi.dashboard`.
- **No** `[[link_handlers]]`. `[[events]]` deferred (see Refresh).

## Socket / CLI surface used
Read: `session.snapshot` (or `agent.list`). Alert: `notification.show`. Jump: `agent.focus`. Transport
**socket-first with `herdr` CLI fallback** (`HERDR_BIN_PATH`). Optional later: `events.subscribe`
filtered to `pane.agent_status_changed`.

## Refresh model
- **v1: poll+diff of `session.snapshot`** in the watcher (auto-title-proven, version-independent). The
  popup snapshots on open + light poll while open.
- `events.subscribe` is an **optimization, deferred** — ticket 012 measured ~10 s event backlog on herdr
  0.8.2 while the 0.9 docs claim no replay; switching to events requires a small **re-measurement on the
  target herdr version** first. v1 ships poll+diff regardless, so this never blocks the build.

## Trust posture / capability declaration
> **Read-only** agent observation (`session.snapshot`/`agent.list`); the **only** state change is an
> explicit user **jump** (`agent.focus`, which herdr treats as "seen"). Alerts **only** via herdr
> `notification.show` — **no OS-notify subprocess, no network, no telemetry**. Spawns: the `herdr` CLI
> only (fallback for the socket). Files: reads `$HERDR_PLUGIN_CONFIG_DIR/.env`; writes only its own
> `$HERDR_PLUGIN_STATE_DIR` (daemon pid / last-seen bookkeeping). **No host-config mutation** (unlike
> radar's `config.toml` blocks), **no bundled font**. All agent-derived text sanitized as hostile input.

The **5 guarantees** (mirroring 008/009): no network · no unexpected subprocess (only `herdr`) · minimal
pinned+audited deps · source-first trustable binary (prebuilt + source fallback) · deterministic/read-only.

## Config (herdr-native, per 006) & state
- **`.env`:** `POPUP_WIDTH`/`POPUP_HEIGHT`; `ALERT_ON=blocked,done` (subset to tune); `ALERT_SOUND=none|done|request`;
  `POLL_MS` (watcher diff cadence); `IDLE_DIM_MIN` (idle-dim threshold); `MAX_ROWS`; `ICONS=nerd|unicode|ascii`
  (mirror the picker's glyph fallback); `WATCHER=on|off`.
- **State:** `$HERDR_PLUGIN_STATE_DIR` — daemon pid (supersession) + last-seen `agent_status` per pane
  (for transition diffing). All rebuildable from a snapshot.

## Language, build & shared code (per 007 + ticket 014 decision)
- **Rust** — interactive ratatui TUI + a long-lived daemon. Prebuilt binaries + `install.sh` source
  fallback. Reuses the **picker's ratatui/crossterm/serde stack** (already audited in `deny.toml`).
- **Shared Rust modules live canonically in `shared/rust/`** (socket/snapshot client, poll loop,
  hostile-input sanitizer), **copied into each plugin's `src/` via `just sync-shared`, CI drift-checked**
  — the same fix-once pattern as the bash shims (extends 006 to Rust). **Consequence (execution):**
  auto-title's existing `socket.rs`/`snapshot.rs`/`sanitize.rs` get retrofitted to consume the canonical
  copies, and `scripts/ci.sh` + `check-sync` + `sync-shared` extend to `shared/rust/`.

## Keybinding
Documented, opt-in (no manifest keys surface): default `prefix+d` → `herdr-mihi.dashboard.open`, added
via a `setup` action like lazygit (conflict-safe). Note the herdr-remote caveat (`plugin_action`
bindings omitted in default `--remote`).

## Role / references
Mines **herdr-radar** (status taxonomy `working/blocked/done/idle/idle_stale`, event-wake+snapshot
discipline, non-disruptive-by-token approach — but *not* its `config.toml` mutation) and
**herdr-navigator** (the `[[actions]]→[[panes]]` open pattern, `agent focus <pane_id>` jump, read/write
separation, jump-back ergonomics). Shares the auto-title daemon + sanitizer via `shared/rust/`.

## Deferred (post-v1)
Optional `agent.view.set` projection as an ambient always-on layer (architecture A) · event-driven
refresh (after re-measurement) · jump-back history · fuzzy filter if the roster grows · per-agent tokens
(context/summary) in rows · any AI/LLM (never).

## Acceptance criteria
- Watcher toasts via `notification.show` on `blocked`/`done` transitions (tunable in `.env`); honors
  `ui.toast.delivery=off`; self-supervises (survives restart, exits on supersession).
- Popup opens as a `popup`; shows the full roster grouped by workspace, attention-sorted; rows show
  status glyph + `agent › activity` + state + age; agent-derived text sanitized.
- `⏎` jumps via `agent focus <pane_id>` (pane id, not terminal id); reading the list marks nothing seen.
- Read-only otherwise; no network; only `herdr` spawned; capability declaration accurate; Rust; prebuilt
  + source fallback; shared modules sourced from `shared/rust/` and drift-checked in CI.
