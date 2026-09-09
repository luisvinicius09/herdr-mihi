---
id: 010
title: "Spec: picker plugin brief (install/selection UX)"
labels: [wayfinder:prototype]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [006, 007]
---

## Question

The topology decision (ticket 005) made the **picker** a v1 plugin: a herdr plugin whose pane lists
the collection and installs chosen plugins via their per-plugin release refs. Write its brief — and
because the key question is "how should the pane look & behave," `/prototype` a rough version to
react to first.

- **Behavior**: open a pane (which placement?), list the collection from the catalog (format =
  ticket 006), let the user select one/many, run `herdr plugin install you/herdr-mihi --ref
  <name>-latest` per pick (support pinning `<name>-vX.Y.Z`). Show installed vs available; allow uninstall?
- **herdr surfaces**: `[[actions]]` to open it, `[[panes]]` for the UI; how it shells out to the
  `herdr` CLI / socket API. Bootstrapping (the picker installs itself first via `--ref picker-latest`).
- **Catalog source**: read the catalog from the installed picker's own files, or fetch the latest
  from the repo at run time?
- **Language** (per ticket 007 heuristic) and why. References worth mining: `herdr-plus` (fuzzylist,
  install.sh), `herdr-navigator` (fuzzy navigation).
- Acceptance criteria.

Output: a rough prototype linked as an asset + a self-contained brief ready to hand to build.

## Resolution

Prototype built ([../prototypes/010-picker-mock.sh](../prototypes/010-picker-mock.sh) — throwaway, runnable, prints
commands only, no herdr needed) and reacted to; brief written from it:
[../assets/010-picker-brief.md](../assets/010-picker-brief.md).

**`herdr-mihi.picker`** (Rust TUI) — the collection's trust-visible front door: a **popup** listing the catalog with
installed/available/update states, **multi-select install/update/uninstall**, and a **confirm screen showing the EXACT
`herdr plugin install … --ref <plugin>-latest` commands + the resolved commit SHA** before anything runs. Installs
**only** from the pinned `luisvinicius09/herdr-mihi`; network only to it. User approved: layout, confirm-as-trust-anchor
(+ resolved SHA), install/update/uninstall scope, and popup placement (promotable to a tab).
