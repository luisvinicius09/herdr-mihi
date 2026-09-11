---
id: 012
title: "Research: what agent status/lifecycle herdr exposes to a plugin (dashboard groundwork)"
labels: [wayfinder:research]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: []
---

## Question

Phase 2's flagship is a **non-disruptive agent-status dashboard** — a place to go *while
agents work* that surfaces their status **without disrupting the agents' current state**.
It's fog: we can't yet say what it shows, how you reach it, or what "non-disruptive" means
concretely. Before any of that can be prototyped, we need to know what herdr actually
**exposes** to a plugin. This ticket answers, from evidence (docs + herdr source + the
reference plugins + what auto-title already measured):

- **Agent status/state model** — what states does herdr track for an agent (idle / working /
  blocked / waiting-input / done?), and *where* are they exposed to a plugin: `session.snapshot`
  (`PaneInfo.agent` / `status` / `agent_session`), `HERDR_PLUGIN_CONTEXT_JSON`,
  `pane.report_metadata`, CLI? Which fields, what shape, how reliable?
- **Agent lifecycle events** — are there `[[events]]` for agent transitions (needs-input /
  blocked / finished)? What's their cost/latency (auto-title measured ~10 s of event backlog —
  does that rule events out for a dashboard, or is "an agent just finished" worth the lag)?
- **How a plugin renders + reaches a view non-disruptively** — the `[[panes]]` placements
  (overlay / popup / split / tab / zoomed): which of these change the *focused agent's* pane
  state or steal input, and which don't? Is there a jump-to-pane primitive (the navigator
  pattern) and does jumping disturb the agent you jump to?
- **Reference mining** — what do `hhdebb/herdr-radar` (overview) and `thanhdat77/herdr-navigator`
  (jumping) do with these surfaces, and what's worth carrying?
- **Define "non-disruptive" concretely** in herdr terms (read-only snapshot polling, no focus
  steal, no pane mutation, close-clean).

Produce a markdown summary as a linked asset. Feeds the phase-2 dashboard prototype ticket
(currently fog). Does **not** decide the dashboard's design — only what's buildable.

## Resolution

herdr exposes a **first-class agent lifecycle** — enough to build the dashboard with **no bespoke
detection**. Every agent has `agent_status ∈ {idle, working, blocked, done, unknown}` (a plugin may
only *report* `idle/working/blocked/unknown` via `pane.report_agent`; **`done` is derived by herdr**),
readable from `session.snapshot`, `agent.list`/`herdr agent list`, or the event stream. The two states
a "come back to me" dashboard cares about are already computed: **`blocked` = needs your input**, and
**`done` = "idle but not yet *seen*"** (finished while you were away). `AgentInfo` (richer than
`PaneInfo`) carries `pane_id`, `state_change_seq` (monotonic sort/dedup), `agent_session`, cwd, stripped
title, tokens; herdr also rolls status up to tab/workspace and keeps global attention counts.

**Two non-disruptive architectures, each proven by a reference plugin — the fork the prototype must
settle:**
- **(A) Projection** via **`agent.view.set`** (declarative filter+sort of herdr's built-in Agents view,
  no pane, moves nothing) — the **radar** model, non-disruptive by construction (radar: event-as-wake-
  hint + fresh `agent.list` per frame, zero deps; but it invasively writes `config.toml` — we shouldn't).
- **(B) Own read-only TUI** in a **`popup`** (least-disruptive placement: no focus change, no pane id,
  no lifecycle events) that renders the snapshot and **jumps on select** via `herdr agent focus <pane_id>`
  — the **navigator** model (ratatui, CLI-only, list-read cleanly separated from jump-write).
- **(C) Hybrid** — background `pane.agent_status_changed` subscription filtered to `blocked`/`done` for
  attention, plus an on-demand popup to survey+jump.

**Two sharp gotchas that must shape the design:** ① **`agent focus` takes a PANE id (`w3:p1`), not a
terminal id** (navigator's shipped bug #13) — store `pane_id` as the jump target. ② **focusing/viewing
an agent marks it "seen," flipping `done`→`idle`** — so *looking* can clear the attention signal; a
"peek" must read without focusing. **Refresh:** default to snapshot poll, optionally event-woken, but
**re-measure event backlog on the target herdr version** (ticket 002 saw ~10 s on 0.8.2; the 0.9 docs
say no replay, and radar's `min 0.9.0` runs event-driven). All three architectures are feasible on our
`min_herdr 0.8.0` pin (`agent.view.set` ≥0.7.5, `session.snapshot` ≥0.7.2).

Full write-up, field/method tables, and a concrete definition of "non-disruptive":
[../assets/012-dashboard-agent-status.md](../assets/012-dashboard-agent-status.md). Surfaces the
**dashboard prototype** ticket (013), now on the frontier.
