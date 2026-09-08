---
id: 009
title: "Spec: lazygit plugin brief (done better)"
labels: [wayfinder:grilling]
status: open
assignee: null
blocked_by: [003, 006]
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
