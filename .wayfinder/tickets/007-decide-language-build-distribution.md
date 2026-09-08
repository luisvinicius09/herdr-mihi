---
id: 007
title: "Decide: language & build/distribution conventions"
labels: [wayfinder:grilling]
status: open
assignee: null
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
