---
id: 008
title: "Spec: auto-title plugin brief"
labels: [wayfinder:grilling]
status: open
assignee: null
blocked_by: [002, 006]
---

## Question

Write the **per-plugin brief** for `auto-title` — the handoff artifact a builder implements
from. Grounded in the reference research (ticket 002) and the conventions (ticket 006):

- The **gap it fills** and the exact behavior (what titles, from what signals, when they update).
- Which herdr surfaces it uses (`[[events]]` / `[[startup]]` — which events) and how it renames.
- **Config options** and defaults; state, if any.
- **Chosen language** (per ticket 007 heuristic) and why.
- How it **improves on** kryptamine/herdr-auto-title.
- Acceptance criteria — what "done" looks like.

Output: a self-contained brief (as a linked asset) ready to hand to a build phase.
