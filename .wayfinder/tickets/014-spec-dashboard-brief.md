---
id: 014
title: "Spec: agent-status dashboard brief"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [012, 013]
---

## Question

Consolidate the research (012) and the approved prototype (013) into a **handoff-ready brief** for
`herdr-mihi.dashboard` — the same shape as the auto-title / lazygit / picker briefs (008/009/010): a
spec someone can build from without further decisions. Most choices are already locked in
[../prototypes/013-dashboard-mock.md](../prototypes/013-dashboard-mock.md); this ticket writes them up
and settles the remaining brief-level details:

- **Manifest shape** — `[[startup]]` (watcher), `[[actions]]` (open) + `[[panes]] popup` (dashboard),
  `[[build]]`, `min_herdr_version` (confirm the floor that has `notification.show` + `agent.view.set` +
  `agent focus` by pane id), platforms macOS·Linux, id `herdr-mihi.dashboard`.
- **Socket/CLI surface used** — `session.snapshot`/`agent.list` (read), `pane.agent_status_changed`
  (optional), `notification.show` (alerts), `agent.focus` (jump); socket-first with CLI fallback.
- **`.env` config surface** — which transitions alert, sound, poll interval, idle-dim threshold, max
  rows, glyph set (`ICONS` like the picker), watcher on/off.
- **Title/glyph grammar** — status glyphs + Nerd-Font/unicode/ascii fallback; row format
  (`agent › activity`, state, age); hostile-input sanitization (reuse auto-title's approach).
- **The 5 security guarantees + capability declaration** (mirror 008/009/010): read-only observation,
  only-jump-focuses, alerts only via `notification.show` (no OS subprocess), no network, no
  host-config mutation, minimal audited deps.
- **Watcher refresh** — v1 ships **poll+diff of `session.snapshot`** (version-safe); note
  `events.subscribe` as an optimization gated on a small event-backlog re-measurement (012 flagged the
  0.8.2-vs-0.9 discrepancy) — not a v1 blocker.
- **Shared-code opportunity** — the socket client / snapshot poll-loop / sanitizer are now used by
  auto-title, (maybe) the picker, and the dashboard: decide whether to factor a shared internal crate
  (touches conventions, tickets 005/006) or copy.

Output: a linked brief asset. This is the last ticket toward the dashboard destination — when it lands,
the dashboard is buildable in phase 1's execution style.

## Resolution

Wrote the handoff-ready brief: **[../assets/014-dashboard-brief.md](../assets/014-dashboard-brief.md)** —
`herdr-mihi.dashboard`, Rust, hybrid (watcher + popup), min_herdr 0.8.0 (verify), with manifest shape,
socket/CLI surface, `.env` config, glyph grammar, the 5 security guarantees + capability declaration, and
acceptance criteria, all consolidated from tickets 012/013.

**The one open decision (shared Rust code) is settled:** extend the bash-shim convention (ticket 006) to
Rust — canonical modules in **`shared/rust/`** (socket/snapshot client, poll loop, sanitizer), copied
into each plugin's `src/` via `just sync-shared`, **CI drift-checked**. Chosen over a vendored shared
crate and over independent copies because it keeps release trees self-contained (honors ticket 005's
subtree-free refs), adds no git/crates.io dep (honors `deny.toml`), and reuses a proven pattern.
**Execution consequence (not a decision):** auto-title's `socket.rs`/`snapshot.rs`/`sanitize.rs` get
retrofitted to the canonical copies, and `sync-shared`/`check-sync`/`scripts/ci.sh` extend to
`shared/rust/` — done at build time.

With this the **dashboard is fully specified and buildable**. No further dashboard decisions remain;
what's left is execution (phase-1 style) and the later-phase plugin briefs (still fog).
