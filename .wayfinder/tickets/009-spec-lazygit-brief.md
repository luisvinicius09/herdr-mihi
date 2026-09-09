---
id: 009
title: "Spec: lazygit plugin brief (done better)"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [003, 006, 007]
---

## Question

Write the **per-plugin brief** for the `lazygit` plugin — the "done better" version — as the
handoff artifact a builder implements from. Grounded in the reference research (ticket 003) and
the conventions (ticket 006):

- The exact behavior: how the user opens lazygit, which **pane type**, and how it's **scoped**
  (current worktree / workspace / cwd), pane reuse vs new.
- Which herdr surfaces it uses (`[[actions]]`, `[[panes]]`, keybinding) and missing-binary handling.
- **Config options** and defaults.
- **Chosen language** (per ticket 007 heuristic) and why.
- The **specific improvements** over Crokily/herdr-lazygit (from ticket 003's weakness list).
- Acceptance criteria — what "done" looks like.

Output: a self-contained brief (as a linked asset) ready to hand to a build phase.

## Resolution

Brief written: **`herdr-mihi.lazygit`** — thin, **pure-bash** herdr↔lazygit glue. A single `open` action toggles a
**popup** running the user's **own** lazygit (on `PATH`) with their **own config untouched**, worktree-scoped
(`flock`-guarded, readable-error-on-failure). Trust cuts vs the reference: no python3, no bundled/downloaded binaries,
no network, and **AI-commit removed** (staged-diff exfiltration). Dropped the `open-tab` action (the user promotes the
popup to a tab manually) and all config layering (nothing to inject once the extra verbs are cut). Capability
declaration: spawns lazygit/git/herdr, `network: none`. Full brief:
[../assets/009-lazygit-brief.md](../assets/009-lazygit-brief.md).
