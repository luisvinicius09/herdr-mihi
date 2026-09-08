---
id: 005
title: "Decide: repo topology & install-footprint mechanism"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [001]
---

## Question

Given how herdr actually fetches a subdir install (ticket 001), how do we deliver "a plugin of
plugins the user picks from" while *strongly preferring* not to bring unchosen code onto their
machine?

Decide between (and settle the details of) the viable options:

- **Monorepo of subdir-plugins** — one repo, each plugin a subdir, `herdr plugin install
  you/repo/<plugin>`. Simplest; footprint depends on ticket 001's finding.
- **Separate repos per plugin** — guarantees minimal footprint; heavier to maintain/release.
- **Monorepo + meta-installer plugin** — an umbrella plugin that presents a picker and fetches
  only the chosen plugins.
- Some hybrid (e.g. monorepo for dev + per-plugin release tags/branches).

Output: the chosen topology, *why*, and the concrete install/selection UX the user will have.
This is the linchpin — it shapes tickets 006 and 007.

## Findings from ticket 001 (now unblocked)

herdr shallow-clones the **whole repo** per subdir install and stores a **full copy per plugin-id** — no sparse
checkout (see [../assets/001-herdr-install-footprint.md](../assets/001-herdr-install-footprint.md)). Consequences:

- A plain **monorepo brings all unchosen code, × N copies**; a meta-installer over it does **not** fix footprint.
- To honor the footprint preference, each installed plugin must map to a git ref whose tree is only that plugin:
  **separate repos**, or **per-plugin release branches/tags (subtree split)** from a dev monorepo.
- Leads to check before deciding: herdr's `workers/plugin-marketplace/` (discovery vs install channel?); prior art
  `cloudmanic/herdr-plus` (ships a plugin-of-plugins) and `yigitkonur/awesome-herdr`.

## Resolution

**Dev monorepo of independently-installable plugins, published as per-plugin release refs, selected via a picker
plugin.** Full ADR: [../assets/005-topology-decision.md](../assets/005-topology-decision.md).

- **Topology** — one repo `you/herdr-mihi`, one plugin per `plugins/<name>/`, each free to use go/rust/bash.
- **Publish** — release CI runs `git subtree split --prefix=plugins/<name>` → immutable tag `<name>-vX.Y.Z` +
  moving `<name>-latest`. herdr installs the manifest at a ref's **root**, so the ref lands only that plugin.
- **Install** — `herdr plugin install you/herdr-mihi --ref <name>-latest` (or pin `-vX.Y.Z`); `herdr plugin link
  plugins/<name>` for dev.
- **Selection UX (v1)** — a **picker plugin** (bootstrapped via `--ref picker-latest`) lists the collection from a
  machine-readable **catalog** and installs picks; README carries copy-paste commands as fallback.

**Rejected:** plain monorepo (brings all code ×N — ticket 001), separate-repo-per-plugin (fragments maintenance),
single umbrella plugin (forfeits per-plugin language freedom).

**Surfaced:** new v1 plugin → the picker (ticket 010). **Handed to:** ticket 006 (catalog format + versioning +
subtree-split-friendly layout), ticket 007 (compiled-binary build/dist within the tag CI).
