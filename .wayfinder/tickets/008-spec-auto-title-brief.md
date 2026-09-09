---
id: 008
title: "Spec: auto-title plugin brief"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [002, 006, 007]
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

## Resolution

Brief written: **`herdr-mihi.auto-title`** — a **trust-first** Rust reimplementation of the reference's core
(`[[startup]]` daemon polling `session.snapshot`; deterministic confidence-ladder titles; manual-rename protection;
hostile-input sanitization). The spine is **five verifiable security guarantees** — no network, no subprocess/shell,
minimal audited deps, source-first trustable binary, deterministic/reliable — plus a **capability declaration** stating
exactly what it touches. The four "do better" levers (multi-agent, hot-reload, friendly names, custom grammar) are
deferred to keep v1 small and auditable. Full brief:
[../assets/008-auto-title-brief.md](../assets/008-auto-title-brief.md).

Surfaced a **collection-wide trust-first principle** (added to the map Notes): every plugin self-authored, minimal,
provably safe, shipping a capability declaration. Applies to tickets 009/010 and the phase-2 dashboard.
