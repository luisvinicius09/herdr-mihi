---
id: 011
title: "Decide: how to test & develop plugins against herdr"
labels: [wayfinder:research]
status: open
assignee: null
blocked_by: [008, 009, 010]
---

## Question

Plugins run *inside* herdr (socket, panes, events, `HERDR_PLUGIN_CONTEXT_JSON`), so "does it work" can't
be answered by unit tests alone. Produce the collection's **testing & dev-loop convention + concrete,
followable instructions**, grounded in how the reference plugins and herdr itself support testing — and
covering all three phase-1 plugin shapes (daemon/poller, action+popup, installer).

- **Dev loop:** `herdr plugin link <dir>` (runs no `[[build]]` — so build/`sync-shared` by hand first) →
  exercise → read `herdr plugin log list`. How to iterate fast; `herdr server reload-config` for keys.
- **Unit tests:** pure logic per language — Rust `cargo test`, bash `bats` — hermetic via env-overridable
  `$HERDR_PLUGIN_CONFIG_DIR`/`$HERDR_PLUGIN_STATE_DIR`; fixtures for `CONTEXT_JSON` / `EVENT_JSON` /
  `session.snapshot` (herdr-nvim's `ConfigEnvGuard`/`StateDirGuard` pattern).
- **Mock-herdr tests:** can we stand up a fake socket server returning canned `session.snapshot` /
  `pane.process_info` and asserting `tab.rename` (auto-title), or stub `herdr plugin install` (picker)?
  Investigate what herdr exposes — is there a headless/test mode or a documented socket contract to mock?
- **Integration / manual checklist:** verify each shape live — daemon (auto-title titles track work),
  action+popup (lazygit toggles, worktree-scoped), installer (picker runs the right `--ref` install) —
  including the **herdr-remote keybinding caveat**.
- **CI reality:** herdr won't be in CI → CI runs unit + mock + lint + the **trust assertions**
  (no-network, no-subprocess where promised, `cargo-audit`/`deny`); integration is a documented *local*
  checklist a human/agent follows.

Output: a `TESTING.md` convention + per-plugin-shape test recipes, as a linked asset.
