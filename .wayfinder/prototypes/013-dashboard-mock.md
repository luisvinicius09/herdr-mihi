# Prototype: non-disruptive agent-status dashboard

**Ticket:** 013 · **Type:** prototype (HITL) · Approved by Luis via reaction to the mockups below.
Grounded in [../assets/012-dashboard-agent-status.md](../assets/012-dashboard-agent-status.md).

## Approved shape: **(C) Hybrid** — a background watcher + an on-demand popup

### ① Watcher (ambient) — a self-supervising `[[startup]]` daemon

Subscribes to herdr's `pane.agent_status_changed` and raises a native toast when an agent flips to a
"come back" state. Uses herdr's own **`notification.show`** (CLI `herdr notification show`) — no
OS-notify subprocess, and it honors the user's `ui.toast.delivery` (so `off` disables it) and herdr's
rate-limiting.

```
🔔 claude · Add observation fields → BLOCKED (needs you)     [agrotroca]
🔔 codex  · release pipeline       → DONE                    [herdr-mihi]
```

- **Alerts on:** `blocked` **and** `done` (both are "come back" moments). Which transitions + the
  sound (`none`/`done`/`request`, default `none`) are **tunable via `.env`**.
- Self-supervises like auto-title (survive herdr restart, detect supersession, exit clean).

### ② Popup (on-demand) — our own read-only ratatui TUI

Opened by a keybind → action → `[[panes]] placement = "popup"` (least-disruptive: no focus steal, no
pane id, no lifecycle events). **Full roster, attention-sorted, grouped by workspace.** Enter jumps.

```
┌ herdr-mihi · agents ── 1 blocked  1 done ────────────────┐
│                                                          │
│ agrotroca                                                │
│  ● claude › Add observation fields          blocked  ⚠   │
│  ◐ codex  › refactor api                    working  2m  │
│ herdr-mihi                                               │
│  ✓ claude › release pipeline                done         │
│  ○ shell                                    idle         │
│                                                          │
│ ↑↓ move   ⏎ jump   r refresh   q close                   │
└──────────────────────────────────────────────────────────┘
  ● blocked   ✓ done   ◐ working   ○ idle
  ⏎ = herdr agent focus <pane_id>
```

- **Sort:** attention-first — `blocked → done → working → idle` (ties by most-recent
  `state_change_seq`), grouped under their workspace.
- **Row:** status glyph · `agent › activity` (from `display_agent` + stripped title, sanitized as
  hostile input per auto-title) · state · age.
- **Jump:** Enter → `herdr agent focus <pane_id>` (**pane id `w:p`, never terminal id** — navigator #13).

## Locked decisions (feed the brief, ticket 014)

| # | Decision | Choice | Why |
|---|---|---|---|
| 1 | Architecture | **Hybrid** (watcher + popup) | Most capable; matches "a place to go" + "tell me when to come back" |
| 2 | Popup scope | **Full roster, attention-sorted** | Overview *and* attention in one view; watcher covers pure alerts |
| 3 | Seen contract | **Only an explicit jump marks-seen** | Reading never mutates herdr state; attention persists until you act (non-disruptive) |
| 4 | Alert surface | herdr **`notification.show`** | Native toast, respects `ui.toast.delivery`, rate-limited; no OS subprocess |
| 5 | Alert transitions | **blocked + done**, tunable in `.env` (sound default `none`) | Both are "come back" moments; full opt-out |
| 6 | Refresh | Watcher: detect transitions by **poll+diff of `session.snapshot`** (auto-title-proven, version-safe), event-subscription as an optimization; popup: snapshot-on-open + light poll while open | Avoids the 0.8.2 event-backlog risk (012); events re-measured before relying on them |
| 7 | Reach | keybind → `[[actions]]` → `[[panes]] popup`; watcher auto-starts via `[[startup]]` | Proven navigator/picker pattern |
| 8 | Language / reuse | Rust; reuse the picker's **ratatui** stack + `herdr/run.sh` shim; watcher reuses auto-title's self-supervising daemon | Consistency, minimal new surface |

## Trust cuts (pre-decided; the brief formalizes them in a capability declaration)

- **Read-only observation** — only `session.snapshot`/`agent.list` (+ optional `events.subscribe`);
  never `send_text/keys/input/prompt`.
- **Only jump focuses** (and intentionally marks-seen); reading the list mutates nothing.
- Alerts only via herdr `notification.show` — **no OS-notify subprocess, no network**.
- **No host-config mutation** (unlike radar's `config.toml` blocks), **no bundled font**.
- Socket-first with **CLI fallback**; minimal pinned+audited deps (the picker/navigator set already
  covers ratatui/crossterm/serde).

## Open for the brief (not decided here)
- Exact `.env` surface (which transitions, sound, poll interval, idle-dim threshold, max rows).
- Title/glyph grammar details + Nerd-Font/unicode/ascii glyph fallback (mirror the picker's `ICONS`).
- Whether to *also* offer an optional `agent.view.set` projection (architecture A) as a cheap ambient
  layer — a possible later add, out of scope for v1.
- **Event-backlog re-measurement** on the target herdr version — a small optimization task; the v1
  watcher ships poll+diff regardless, so this only unlocks switching to `events.subscribe`.
