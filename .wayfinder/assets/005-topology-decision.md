# Decision: repo topology & install-footprint mechanism (ticket 005)

**Status:** Decided (2026-09-08). Resolves the open question surfaced by ticket 001.

## Decision

The collection is a **single dev monorepo** of many **independently-installable** plugins,
published as **per-plugin release refs**, and selected via an in-repo **picker plugin**.

### Topology
- One repo `you/herdr-mihi`; one plugin per `plugins/<name>/`; each plugin free to use its own
  language (go / rust / bash).

### Publish mechanism (release CI)
- On release, for each changed plugin, CI runs `git subtree split --prefix=plugins/<name>` →
  a commit whose **root tree = that plugin** — then:
  - pushes an **immutable tag** `<name>-vX.Y.Z`, and
  - moves a **`<name>-latest`** pointer to it.
- herdr installs the manifest at a ref's **root** (verified in `src/cli/plugin.rs`:
  `manifest_root = checkout` when no subdir), so `--ref <name>-vX.Y.Z` lands **only that plugin** —
  honoring the footprint preference from one repo.

### Install UX
- Newest:  `herdr plugin install you/herdr-mihi --ref <name>-latest`
- Pinned:  `herdr plugin install you/herdr-mihi --ref <name>-vX.Y.Z`
- Dev:     `herdr plugin link plugins/<name>` (skips clone/build)

### Selection UX (v1)
- A **picker plugin** (itself in the collection, bootstrapped via `--ref picker-latest`) opens a
  pane listing the collection; ticking plugins runs the right per-ref install (with version pin support).
- Backed by a **collection catalog** — a machine-readable list of plugins (id, description, latest
  ref, language, platforms) that the picker, the README table, and the release CI all consume.
  Catalog format is decided in ticket 006.
- README also carries copy-paste install commands as the no-picker fallback.

## Why (vs the alternatives considered)
- **Plain monorepo** — rejected: ticket 001 proved a subdir install clones the WHOLE repo, ×N copies.
- **Separate repo per plugin** — rejected: minimal footprint but N repos fragments maintenance.
- **Single umbrella plugin** (herdr-plus model) — rejected: simplest, but forfeits per-plugin
  language freedom (go/rust/bash), which the user wants.
- **Chosen** — one maintainable repo + per-plugin language + minimal footprint + versioned installs.

## Consequences / follow-ups
- New v1 plugin: the **picker** → ticket 010.
- Ticket 006 must define the **collection catalog** format + the `<name>-vX.Y.Z` / `<name>-latest`
  versioning scheme + a `plugins/<name>/` layout that is subtree-split-friendly.
- Ticket 007 must define how compiled (go/rust) plugins are built **within** the subtree-split/tag
  CI (build-on-install vs prebuilt binaries), for macOS + Linux.
