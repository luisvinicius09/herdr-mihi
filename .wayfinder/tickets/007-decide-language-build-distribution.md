---
id: 007
title: "Decide: language & build/distribution conventions"
labels: [wayfinder:grilling]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: [005]
---

## Question

The collection may mix bash / rust / go. Decide the rules so it stays maintainable on
**macOS + Linux**:

- **When to reach for which language** — bash for thin wrappers/glue; rust/go for anything
  stateful, long-running, or performance-sensitive. Write the heuristic down.
- **Build & distribution for compiled plugins** — build-on-install (toolchain required on the
  user's machine, per herdr's install build step) vs **prebuilt per-OS binaries** shipped via
  releases. Consider the footprint decision from ticket 005.
- Shared build tooling / task runner across plugins; how the manifest's build step is declared
  per language.

Output: the language-choice heuristic + the build/distribution convention. (Informs 008/009.)

## Inputs from ticket 005 (topology decided)

Distribution backbone = release CI that `git subtree split`s each changed plugin into an immutable tag
`<name>-vX.Y.Z` and moves `<name>-latest` (see [../assets/005-topology-decision.md](../assets/005-topology-decision.md)).
This ticket decides how **compiled (go/rust) plugins** are built within that: build-on-install (herdr runs the
manifest's build step on the user's machine) vs **prebuilt per-OS binaries** attached to the tag / committed into the
split ref (cf. `herdr-plus` and `herdr-auto-title`, which both ship goreleaser binaries). Must work for macOS + Linux.

## Resolution

Decided: **bash for thin glue; Rust the default compiled language** (Go allowed per-plugin when justified);
**prebuilt per-OS binaries** — the ecosystem norm. `[[build]] = ["bash","herdr/install.sh"]` downloads
`<plugin>-<os>-<arch>` from the plugin's GitHub release (matrix: darwin arm64/amd64 + linux amd64/arm64) and falls
back to source only if the download fails and a toolchain exists. **No build-on-install as the primary path.** Full
spec (install.sh + release.yml + uniform asset naming):
[../assets/007-language-build-distribution.md](../assets/007-language-build-distribution.md).

Evidence: herdr aborts installs on build failure and won't install toolchains (docs); every popular compiled plugin
ships prebuilt binaries (reviewr 629★, file-viewer, herdr-plus, herdr-nvim). **Unblocks tickets 008/009/010.**
