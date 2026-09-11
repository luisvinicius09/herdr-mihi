---
id: 013
title: "Prototype: the non-disruptive agent-status dashboard (architecture + look + reach)"
labels: [wayfinder:prototype]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [012]
---

## Question

Ticket 012 established what herdr exposes (agent lifecycle `idle/working/blocked/done/unknown`, the
`done`="finished & unseen" and `blocked`="needs input" signals, `agent.view.set` projection, `popup`
rendering, `agent focus <pane_id>` jump). Now make it concrete enough to write a brief. This is a
**HITL prototype** ("how should it look / how should it behave") — build a cheap rough artifact
(likely via `/prototype`) to react to, then lock the shape. Decide:

1. **Architecture** (the core fork from 012 §6) — **(A)** projection into herdr's built-in Agents view
   via `agent.view.set`, **(B)** our own read-only `popup` TUI that jumps on select, or **(C)** hybrid
   (background attention watcher + on-demand popup). Weigh against the trust posture and "how much do we
   want our own surface vs. lean on herdr's."
2. **What it shows** — which states, grouping (workspace → worktree → agent?), sort (attention / status
   / `state_change_seq`), per-row info (agent name, cwd, last activity, tokens), and how "done"/"blocked"
   are made visually obvious.
3. **How you reach it** — keybind → action → popup? ambient projection? both? And the concrete
   **"non-disruptive" contract** for this UI (012 §5) — especially: does reading the dashboard
   **mark-seen** (probably not), and only an explicit jump focuses (and intentionally marks seen)?
4. **Refresh model** — snapshot poll vs event-woken; if event-driven, spawn a re-measurement task for
   event backlog on the target herdr version (012 flagged the 0.8.2-vs-0.9 discrepancy).
5. **Language/reuse** — reuse the picker's ratatui stack + `herdr/run.sh` shim + capability-declaration
   discipline; auto-title's self-supervising `[[startup]]` daemon if the design is ambient.

Output: an approved prototype + the decisions above, feeding a `spec: dashboard brief` ticket (fog).
Reference: [../assets/012-dashboard-agent-status.md](../assets/012-dashboard-agent-status.md).

## Resolution

Approved shape: **(C) Hybrid** — a self-supervising `[[startup]]` **watcher** that toasts via herdr's
native **`notification.show`** when an agent flips to **blocked or done** (transitions + sound tunable
in `.env`), plus an on-demand **`popup`** ratatui TUI showing the **full roster, attention-sorted**
(`blocked → done → working → idle`, grouped by workspace), where **Enter jumps** via
`herdr agent focus <pane_id>`. Non-disruptive contract: **only an explicit jump marks-seen** — reading
the list mutates nothing. Refresh: watcher detects transitions by **poll+diff of `session.snapshot`**
(auto-title-proven, version-safe; event-subscription is a later optimization pending a backlog
re-measurement); popup snapshots on open. Reuse: the picker's ratatui stack + `herdr/run.sh` shim +
auto-title's daemon; read-only, no OS-notify subprocess, no network, no host-config mutation.

Eight locked decisions + the approved layout + pre-decided trust cuts:
[../prototypes/013-dashboard-mock.md](../prototypes/013-dashboard-mock.md). Surfaces the
**Spec: agent-status dashboard brief** ticket (014), now on the frontier.
