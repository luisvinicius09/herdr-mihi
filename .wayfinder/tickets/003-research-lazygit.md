---
id: 003
title: "Research: how herdr-lazygit works, and where it falls short"
labels: [wayfinder:research]
status: closed
assignee: luisvinicius0906@gmail.com
blocked_by: []
---

## Question

Study [Crokily/herdr-lazygit](https://github.com/Crokily/herdr-lazygit). The user wants this
one **done better**, so capture both how it works and its weaknesses:

- How does it launch lazygit — `[[actions]]`? `[[panes]]` (which pane type: popup/overlay/
  split/zoomed)? Is it scoped to the current worktree / workspace / cwd?
- Manifest shape, language, how it locates the `lazygit` binary, config options, keybinding.
- Concrete **limitations / rough edges** — what would "done better" mean? (e.g. worktree
  awareness, reuse of an existing pane, missing-binary handling, config, multi-repo.)

Produce a markdown summary as a linked asset, ending with a bullet list of the improvements to
make. (Feeds ticket 009.)

## Resolution

Findings written up in [../assets/003-lazygit-reference.md](../assets/003-lazygit-reference.md).

**Bottom line:** `Crokily/herdr-lazygit` (v0.3.0, MIT) is **not a weak plugin to easily beat** — it's a
~3,500-line, two-language (bash + **python3 ≥ 3.7**) reference with a `DESIGN.md` "constitution",
free-key occupancy analysis, worktree-scoped idempotent toggling, 5-layer config, hot-reload, AI commit
generation, an fzf settings pane, and a SHA-pinned private lazygit+fzf runtime. **"Done better" is a
_scope & simplicity_ argument, not a bug-fix one.**

- **Launch:** `[[panes]]` (`placement = "split"`, a 42-col "Git" sidebar) + two `[[actions]]` — `open`
  (split) and `open-tab` (own tab); **no events**. The action shells `open-lazygit.sh`, which opens the
  pane imperatively via `herdr plugin pane open --entrypoint lazygit …` then narrows it over the socket.
  `U` ("Expand") retoggles lazygit's *internal* layout, not the herdr pane type.
- **Scoping (genuinely good):** cwd from `HERDR_PLUGIN_CONTEXT_JSON`; idempotent open/focus/close toggle;
  pane reuse requires a **git-worktree identity match** (`rev-parse --show-toplevel` both sides) or it
  degrades to opening fresh. `flock` guards action-invoke races.
- **Runtime/deps:** downloads pinned **lazygit 0.63.0 + fzf 0.74.0** (SHA-verified, no PATH/brew/sudo) per
  install; **python3 mandatory**; keybinding is **NOT auto-registered** — the user hand-edits `config.toml`.

**Improvements for ticket 009** (full list in the asset): ship a **lean core** (no python3, no bundled
binaries; AI-commit/settings as an optional add-on — fits the "plugin of plugins" thesis); **prefer the
user's own lazygit**, download only as fallback; **auto-register the keybinding** if herdr permits it;
keep the proven bits (worktree-scoped toggle, `flock`, readable-error-on-failure, user-config inheritance).

**Cross-cutting leads surfaced** (not new tickets — routed to existing frontier):
- *Can a herdr manifest declare a default keybinding, or must every plugin document a manual `config.toml`
  edit?* → **ticket 006** (conventions). Recurs for every plugin, incl. `auto-title`.
- *Can a plugin tag/query its **own** panes by a stable id* (vs. `process-info` polling)? → **ticket 006**.
- python3 + two-binary footprint is heavy for a git pane → informs **ticket 007** (build/distribution) and
  the footprint call in **ticket 005**.
