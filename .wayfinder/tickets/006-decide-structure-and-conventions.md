---
id: 006
title: "Decide: repo structure & shared plugin conventions"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
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

## Resolution

Decided: **shared code = ① canonical `/shared` synced into each plugin** (fix-once, CI drift-check), and
**config/state = herdr-native** (`$HERDR_PLUGIN_CONFIG_DIR/.env` + `$HERDR_PLUGIN_STATE_DIR`). All other conventions
ratified from ticket 004. Full spec + new-plugin checklist:
[../assets/006-structure-and-conventions.md](../assets/006-structure-and-conventions.md).

Highlights: layout `plugins/<name>/` + canonical `/shared`; ids `herdr-mihi.<plugin>`; wrapper-shim manifests
(absolute via `$HERDR_PLUGIN_ROOT`, never cwd-relative); one-artifact argv dispatch; exit-0-when-not-applicable +
defensive JSON parsing; mandatory "Platforms: macOS · Linux" README line; `catalog.json` generated from manifests;
per-plugin semver → `<plugin>-vX.Y.Z`/`-latest`; `justfile` CI.

Handed to ticket 007: `/shared/install.sh` contents, `release.yml` matrix, language heuristic. **Re-wired tickets
008/009/010 to also block on 007** (a brief needs the language/build convention before it can name a plugin's language).
