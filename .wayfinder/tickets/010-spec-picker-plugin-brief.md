---
id: 010
title: "Spec: picker plugin brief (install/selection UX)"
labels: [wayfinder:prototype]
status: open
assignee: null
blocked_by: [006]
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
