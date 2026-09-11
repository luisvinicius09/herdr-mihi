# Research: what agent status/lifecycle herdr exposes to a plugin (dashboard groundwork)

**Ticket:** 012 · **Type:** research · **For:** phase-2 non-disruptive agent-status dashboard.
**Primary evidence:** herdr `0.9.0` (2026-09-07) docs + the machine-readable API schema
(`herdrdev/herdr@master:docs/next/api/herdr-api.schema.json`, declares `"protocol": 22`, the
preview/next branch) + two reference plugins mined at source: `hhdebb/herdr-radar` (v1.2.1, Node,
MIT, `min_herdr 0.9.0`) and `thanhdat77/herdr-navigator` (v0.3.6, Rust, MIT, `min_herdr 0.7.3`).
Cross-checked against what ticket 002 measured on herdr 0.8.2.

## Bottom line

herdr exposes a **first-class agent lifecycle** to plugins — enough to build the dashboard with **no
bespoke detection**. Every agent carries an `agent_status` ∈ `{idle, working, blocked, done, unknown}`,
readable from `session.snapshot`, `agent.list`, or an event stream, and **herdr already computes the
one state a "come back to me" dashboard cares about most: `done` = "idle and not yet *seen*"**. There
are **two viable, non-disruptive architectures**, both proven by a reference plugin:

1. **Projection into herdr's built-in Agents view** via `agent.view.set` (declarative filter+sort, no
   pane, moves nothing) — the radar model, non-disruptive *by construction*.
2. **Our own read-only TUI** in a **`popup`** (the least-disruptive pane placement: no focus change, no
   pane id, no lifecycle events) that renders the snapshot and **jumps to an agent on select** via
   `herdr agent focus <pane_id>` — the navigator model.

The single sharpest gotcha: **`agent focus` takes a herdr *pane id* (`w3:p1`), not a terminal id** —
a shipped bug in navigator (#13). And the single sharpest nuance: **focusing/viewing an agent marks it
"seen," flipping `done`→`idle`** — so the act of looking at the dashboard can clear the very "needs
attention" signal it exists to show. Both must shape the design. The design itself (which architecture,
what it shows, how you reach it) is **not decided here** — it feeds the prototype ticket.

---

## 1. Agent status / lifecycle model

### Two enums — one herdr *exposes*, one a plugin may *report*

- **`AgentStatus`** (what herdr computes and hands to plugins): `idle · working · blocked · done · unknown`.
- **`PaneAgentState`** (what an integration may *set* via `pane.report_agent`): `idle · working · blocked · unknown` — **no `done`**. `done` is **derived by herdr, not settable**.

Semantics (docs `agents.mdx`):
- `working` — actively processing.
- `blocked` — "live bottom-buffer snapshot matches a known visible approval, question, or permission
  UI." Detection is *deliberately strict*. **This is the "needs your input" state.**
- `idle` — ready.
- **`done` — "idle but not yet marked *seen*."** Derived. **This is the "finished while you were away"
  state.** Focusing/viewing the agent marks it seen → it flips `done`→`idle`.
- `unknown` — present but can't be classified confidently.

Detection source: **full-lifecycle integration hooks are authoritative when installed**
(`herdr integration install <agent>`; `AgentInfo.screen_detection_skipped=true` signals this), else
**screen-manifest detection** (foreground process + bottom-buffer TOML matching).

**Rollup (built-in):** a `blocked` agent makes its pane/tab/**workspace** look blocked; a `working`
agent makes the workspace look active; a `done` agent **stays visible until viewed**. herdr also keeps
global **attention counts**. (So some of "the dashboard" already exists in herdr's own rollups — the
question is what we add on top.)

### Where it's exposed — every surface

| Surface | How | Fields |
|---|---|---|
| **`session.snapshot`** (socket) | one request → whole session | `panes[]: PaneInfo{ pane_id, terminal_id, workspace_id, tab_id, focused, agent_status, revision, +agent, agent_session, cwd, foreground_cwd, display_agent, label, title, terminal_title_stripped, state_labels{}, tokens{} }`; also `agents[]: AgentInfo` (richer, below); plus `focused_*_id` |
| **`agent.list` / `agent.get`** (socket **or** CLI `herdr agent list`) | agent-centric list | `AgentInfo{ terminal_id, agent_status, workspace_id, tab_id, pane_id, focused, revision, +interactive_ready, launch_pending, screen_detection_skipped, state_change_seq(uint64 monotonic), name, agent, display_agent, cwd, foreground_cwd, title, terminal_title_stripped, state_labels{}, tokens{}, agent_session }` |
| **Events** (socket `events.subscribe`) | push on change | `pane.agent_status_changed` → `{ pane_id, workspace_id, agent_status, +agent, display_agent, title, state_labels }`; `pane.agent_detected` (agent appeared) |
| **CLI** | scripting | `herdr agent list`, `herdr agent get <t>`, `herdr api snapshot`, `herdr agent wait <pane> --until done\|blocked`, `herdr agent explain` |
| **`HERDR_PLUGIN_CONTEXT_JSON`** | per-invocation only | *may* include an `agent` field "when available"; **exact sub-fields not documented** — use `agent.list`/snapshot for typed data |

**`AgentInfo` is richer than `PaneInfo`** — for a dashboard use `agent.list`/`snapshot.agents[]`, not
`panes[]`. Two fields matter especially: **`state_change_seq`** (monotonic — client-side ordering/dedup
and "most-recently-changed" sort without wall-clock) and **`agent_session`**
(`{source, agent, kind:"id"|"path", value}`, present only when a native session ref is stored).

---

## 2. Agent lifecycle events

- **No `agent.started/finished/blocked/needs_input` events.** Every transition — including
  blocked/needs-input/done — arrives as one event: **`pane.agent_status_changed`**. Plus
  `pane.agent_detected` when an agent first appears.
- **Filterable subscription:** `{"type":"pane.agent_status_changed","pane_id":"w1:p1","agent_status":"blocked"}`
  — both `pane_id` and target `agent_status` are optional filters. So a plugin can subscribe to *only
  blocked transitions across all panes* — ideal for a "raise attention when an agent needs me" feature.
- **Delivery:** socket stream (`events.subscribe`) **or** manifest `[[events]]` hooks
  (`on = "<event>"`, `command = [...]`; the process gets `HERDR_PLUGIN_EVENT` +
  `HERDR_PLUGIN_EVENT_JSON`). Every doc example of `on` uses `worktree.created`, but the field is
  free-form so agent events should work — **verify at build**.
- **Backlog/cursor:** docs (0.9) say lifecycle subscriptions **start at accept time and do not replay
  retained events** — no cursor, no sequence-ack for resume. Correct bootstrap: open `events.subscribe`
  → await ack → **buffer** → `session.snapshot` → install → apply buffered → continue; re-snapshot on
  reconnect.
- **`events.wait` / `herdr agent wait --until done|blocked`** — one-shot semantic wait ("observe
  semantic state, not arbitrary command completion").
- **⚠ Version-dependent reliability.** Ticket 002 measured (herdr **0.8.2**) ~**10 s of event backlog
  on subscribe, no cursor, no timestamps** — decisive enough that auto-title chose polling. The **0.9**
  docs describe **no replay**, and radar (`min 0.9.0`) runs an **event-as-wake-hint** model
  successfully. So event behavior **improved between 0.8.2 and 0.9** — but this must be **re-measured on
  the target herdr version** before committing to event-driven refresh. Safe default: **snapshot poll**,
  optionally event-woken (see radar §5).

---

## 3. Rendering + reaching a view — and what "disrupts"

### `[[panes]]` placements (`PluginPanePlacement`: `overlay · popup · split · tab · zoomed`)

| Placement | Effect | Disruptive? |
|---|---|---|
| **`popup`** | "session-modal terminal popup without changing the tab layout"; **no pane id, does not change plugin focus context, emits no pane lifecycle events, not part of pane/layout/agent APIs**; closed via `popup.close`; returns `ui_busy` if Settings/copy-mode/another popup is open | **Least disruptive** — no focus/layout change |
| `overlay` (default) | "temporary zoomed overlay over the active pane; restores previous focus+zoom on close" | Steals focus while open, restores on close |
| `split` | normal pane splitting an existing one; `plugin.pane.open` takes **`focus:false`** | Adds a pane; focus steal is opt-out |
| `tab` | normal pane in a new tab (can target a workspace) | New tab; focus controllable |
| `zoomed` | normal pane, zoomed | Changes layout/focus |

`plugin.pane.open` accepts **`focus` (bool)** — split/tab can open **without stealing focus**.
(Consistent with ticket 010: the picker uses `popup`.)

### `agent.view.set` / `agent.view.clear` — the no-pane projection primitive

The intended way to render an agent dashboard **without opening anything or moving anything**:
installs **one transient declarative projection** for herdr's **built-in Agents view** (expanded +
collapsed sidebar, mobile list, mouse targets, indexed focus, next/prev navigation). "It does not
change `agent.list`, notifications, detection, or global attention counts."
- `AgentViewSetParams{ source(req), label?, filter?, sort? }`.
- **filter** ops `all/any/not/eq/in/exists` over fields `status, workspace_id, tab_id, pane_id, agent,
  seen, state_change_seq` (+ `{"token":"name"}`), with context values `current_workspace_id`,
  `current_tab_id`.
- **sort** over `attention, status, agent, seen, state_change_seq, workspace_order, tab_order,
  pane_order, {"token":"name"}`, `asc|desc`.
- `agent.view.clear` removes it. **Landed in herdr 0.7.5** (our plugins pin `min 0.8.0` → available).

### Focus / jump-to primitives

`pane.focus{pane_id}`, `pane.focus_direction`, `tab.focus`, `workspace.focus`, **`agent.focus{target}`**,
`plugin.pane.focus/close`. CLI: `herdr agent focus`, `herdr pane focus`, `herdr tab focus`.

- **`herdr agent focus` takes a PANE ID (`w<N>:p<M>`), NOT a terminal id** — navigator issue #13
  (`term_…` → `{"error":{"code":"agent_not_found"}}`), fixed v0.3.4. **Store `pane_id` as the jump
  target;** keep `terminal_id`/title only for search/display.
- **Focus never sends input to the process** — input is *only* delivered by explicit
  `pane.send_text/send_keys/send_input` or `agent.send_keys/prompt`. Focusing moves *you*, it doesn't
  poke the agent. (Not stated verbatim in docs — inferred from the model + absence of any input-on-focus
  behavior; `pane.focused` events are UI-only and don't move other clients.)
- **BUT focus has a side effect: it marks the agent "seen"** → can flip `done`→`idle` and decrement
  attention counts. **Design consequence:** viewing/jumping to a `done` agent clears its "done" flag.
  A dashboard must decide whether reading it should mark-seen (probably *not* — read without focusing).

### `[[actions]]` + `[[link_handlers]]`
`[[actions]]{ id, title, contexts, command }` (invoke via `plugin.action.invoke` or keybind
`type="plugin_action"` → `plugin.id.action`). `[[link_handlers]]{ id, title, pattern(regex), action }`
fire on **Control+click**. Proven pattern (navigator): an **`[[actions]]` entry opens a `[[panes]]`
view** — bind a key → action → pane.

### Env vars (confirmed)
`HERDR_SOCKET_PATH, HERDR_BIN_PATH, HERDR_ENV=1, HERDR_PLUGIN_ID/ROOT/CONFIG_DIR/STATE_DIR,
HERDR_PLUGIN_CONTEXT_JSON, HERDR_WORKSPACE_ID/TAB_ID/PANE_ID (not set for popups),
HERDR_PLUGIN_ACTION_ID, HERDR_PLUGIN_EVENT/_JSON`. No herdr-managed storage API in v1 —
`CONFIG_DIR/STATE_DIR` are path discovery only.

---

## 4. Reference mining

### `hhdebb/herdr-radar` — the *projection* model (overview without a view)

- **Not a separate screen.** It rewrites the **display tokens** of agents already in herdr's native
  sidebar (`pane.report_metadata` / `workspace.report_metadata`, `source=plugin:hhdebb.herdr-radar`):
  per-row title/state-mark/vendor-logo/split-indicator/worktree-tree indent, and workspace roll-ups
  (`space_working/blocked/done/idle`), ordered by minute-grained `sort_key`.
- **Status taxonomy (reusable):** `working · blocked(awaiting you) · done · idle · idle_stale`
  (dormant), with **badge-hold**: completion ticks / question marks held until acknowledged.
- **Refresh = event-as-wake-hint + snapshot-per-frame:** subscribes to many events but **ignores their
  payloads** ("WAKE HINTS ONLY"); each debounced wake takes a *fresh* `agent.list`. Constants worth
  stealing: `WAKE_DEBOUNCE_MS=50`, `FRAME_FLOOR_MS=120`, fallback `POLL_MS=150`.
- **Transport:** socket-first, **CLI fallback** (`spawnSync herdr …`). **Zero runtime/dev deps**
  (Node built-ins only) — strong fit for our security posture.
- **Non-disruptive: yes** — "only writes display tokens … never changes focus or pane order." Its only
  `[[panes]]` is a **`popup`** for *settings*.
- **But invasive in one way:** it writes **three managed blocks into the user's `config.toml`**
  (sidebar/tab-bar/theme) + reloads, and ships a patched font. For *our* trust posture, **prefer our own
  surface over rewriting the user's config**; `unconfigure` cleanly reverts (good hygiene to emulate).
- **No jump-to affordance** (the view is the sidebar).

### `thanhdat77/herdr-navigator` — the *own-TUI + jump* model

- **ratatui + crossterm** fuzzy picker; `[[actions]]` (`open`, `open-side`, `jump-back`) → `[[panes]]`
  with placement **`overlay`** (transient) and **`split`** (persistent side pane).
- **Read-only list** from `herdr agent list` + `workspace list` + `pane list` (CLI only, **no socket**),
  agent rows carry live `agent_status`, `pane_id`, `cwd/foreground_cwd`, stripped title.
- **Jump = `run_herdr(["agent","focus", pane_id])`** / `["workspace","focus", id]`. **List-read is
  cleanly separated from jump-write** — polling never disturbs anything; the only mutation is the focus
  call on explicit user action. **jump-back** records origin workspace for one-key return.
- Deps: `crossterm 0.29, ratatui 0.30, nucleo-matcher, fuzzy-matcher, serde/serde_json, toml`. No
  network, no async, only spawns `herdr` (+ optional `zoxide`). MIT, macOS/Linux.
- **Carry:** the actions→overlay pattern; store `pane_id` as jump target; separate read from write;
  jump-back ergonomics. (We already share the ratatui stack via the picker.)

---

## 5. "Non-disruptive," defined concretely (in herdr terms)

A dashboard is **non-disruptive** iff, for the agents it observes and until the user explicitly acts:

1. **Read-only observation** — only `session.snapshot`/`agent.list`/`events.subscribe` (or the CLI
   equivalents). Never `send_text/send_keys/send_input/prompt`.
2. **No focus steal** — render in a **`popup`** (no focus/layout change, no pane id) *or* project via
   **`agent.view.set`** (no pane at all) *or* open a split/tab with **`focus:false`**. Never a
   focus-stealing `overlay`/`zoomed` unless the user asked to open it.
3. **No pane/layout mutation** of the observed agents — no split/move/zoom/close/rename of their panes.
4. **Don't clear attention by looking** — reading the snapshot does **not** mark-seen, but **focusing an
   agent does**. So "peek at the list" must avoid `agent.focus`; only an explicit "jump to this agent"
   should focus (and thereby, intentionally, mark it seen).
5. **No host-config mutation** (unlike radar's `config.toml` blocks) unless opt-in and reversible.
6. **Close-clean** — `popup.close` / `agent.view.clear` leaves no residue; a startup daemon must
   self-supervise and exit on supersession (the auto-title pattern, ticket 002/008).

---

## 6. What's buildable — three architectures (for the prototype to choose among)

All three are **feasible today** on `min_herdr 0.8.0` (our current pin); `agent.view.set` needs ≥0.7.5,
`session.snapshot` ≥0.7.2 — both satisfied.

- **A — Projection (`agent.view.set`, radar-style):** a `[[startup]]` daemon (or an action) that
  installs a filtered/sorted view of herdr's built-in Agents panel (e.g. *blocked first, then done, then
  working, dim idle*). Cheapest, most native, zero custom UI, non-disruptive by construction. **Ceiling:**
  limited to what herdr's Agents view can render — we style/sort/filter, we don't invent layout.
- **B — Own popup TUI (navigator/picker-style):** an `[[actions]]`→`[[panes]] popup` ratatui view that
  reads the snapshot, renders our own dashboard (group by workspace, status glyphs, last-activity,
  tokens), and **jumps via `agent focus <pane_id>`** on select. Full layout control; reuses our picker's
  ratatui stack. **Cost:** we own the rendering + refresh loop; a popup is modal (one at a time,
  `ui_busy` if another is open) and on-demand (not ambient).
- **C — Hybrid:** background daemon subscribed to `pane.agent_status_changed` filtered to `blocked`/
  `done` (attention signal, maybe a notification), **plus** an on-demand popup (B) to survey + jump, and
  optionally a projection (A) as the ambient always-on layer. Most capable, most surface area.

**Refresh:** default to **snapshot poll**, optionally **event-woken** (radar's wake-hint + fresh
snapshot) — but **re-measure event backlog on the target herdr version** first (002 saw 10 s on 0.8.2;
0.9 docs say none).

---

## 7. Leads for the prototype ticket (013) — not decided here

- **Pick the architecture** (A projection / B own popup / C hybrid) — the core fork. It's a
  "how should it look + how do you reach it" question → **prototype (HITL)**, likely via `/prototype`.
- **What it shows:** which of `working/blocked/done/idle/idle_stale`, grouping (workspace → worktree →
  agent, per radar), sort (attention/status/`state_change_seq`), what per-row info (agent name, cwd,
  last activity, tokens).
- **How you reach it:** keybind → action → popup? Or ambient projection? What "non-disruptive" means for
  *this* UI (see §5) — especially the **mark-seen-on-focus** decision.
- **Refresh model:** poll vs event-woken — carry an explicit re-measurement task if event-driven.
- **Reuse:** the picker's ratatui stack + `herdr/run.sh` shim + capability-declaration discipline; the
  auto-title self-supervising `[[startup]]` daemon if the design is ambient.
- **Trust cuts to pre-decide in the brief:** read-only observation, `pane_id` as jump target, no
  host-config mutation, no bundled font, socket-first with CLI fallback, minimal audited deps.

---

## Sources
- herdr docs: https://herdr.dev/docs/plugins/ · /socket-api/ · /agents/ · /agent-automation/ · /cli-reference/
- herdr schema (authoritative): `herdrdev/herdr@master:docs/next/api/herdr-api.schema.json` (`protocol 22`)
- herdr CHANGELOG: `herdrdev/herdr@master:CHANGELOG.md`
- `hhdebb/herdr-radar` @ `main` — herdr-plugin.toml, package.json, README, bin/agent-state.js, bin/agent-view.js, lib/{herdr,daemon,subscribe,frame}.js
- `thanhdat77/herdr-navigator` @ `main` — herdr-plugin.toml, README, src/{sources,herdr,app,update,navigator_state}.rs, CHANGELOG, Cargo.toml, issues/13
- Cross-reference: ticket 002 asset (auto-title, measured on herdr 0.8.2)
