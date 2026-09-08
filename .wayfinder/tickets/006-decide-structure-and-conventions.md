---
id: 006
title: "Decide: repo structure & shared plugin conventions"
labels: [wayfinder:grilling]
status: open
assignee: null
blocked_by: [005, 004]
---

## Question

Given the topology (ticket 005) and the extracted patterns (ticket 004), define the **clean,
easy-to-follow repo structure** and the **shared conventions** every plugin in the collection
follows — the backbone that makes this modular and maintainable:

- Directory layout (where plugins live, shared tooling, docs, a plugin template/scaffold).
- `herdr-plugin.toml` **template** and manifest conventions (id/naming scheme, versioning).
- Config vs state handling conventions (`HERDR_PLUGIN_CONFIG_DIR` / `HERDR_PLUGIN_STATE_DIR`).
- Per-plugin **README** requirements — **must include a "Platforms: macOS · Linux" line** —
  plus what each README documents (install command, config, keybindings).
- Testing / CI approach at the collection level.

Output: a written structure spec + conventions checklist a new plugin must satisfy.
(Feeds tickets 008 and 009 and every later-phase plugin.)

## Inputs from ticket 005 (topology decided)

Topology = dev monorepo + per-plugin release refs + a picker plugin (see
[../assets/005-topology-decision.md](../assets/005-topology-decision.md)). So this ticket must also define:

- The **collection catalog** format — a machine-readable list of plugins (id, description, latest ref, language,
  platforms) consumed by the **picker (ticket 010)**, the README table, and the release CI.
- The **versioning scheme** behind the `<name>-vX.Y.Z` tags and the `<name>-latest` pointer.
- A `plugins/<name>/` layout + shared tooling that is **subtree-split-friendly** (each plugin dir self-contained so
  its split ref is installable on its own).
