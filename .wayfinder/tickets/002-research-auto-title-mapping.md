---
id: 002
title: "Research: how herdr-auto-title maps herdr surfaces to behavior"
labels: [wayfinder:research]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: []
---

## Question

Study [kryptamine/herdr-auto-title](https://github.com/kryptamine/herdr-auto-title) and
extract its **mapping logic** — the thing the user wants to replicate:

- Which herdr surfaces does it use — `[[events]]`? `[[startup]]`? which events (pane/tab/
  workspace/agent/worktree)? How does it call *back* into herdr to rename (which `herdr` CLI
  command)?
- How does it **compute** a title (cwd, running process, git branch, agent state)? Where's the
  config, and what's configurable?
- Manifest shape, language, dependencies, state handling.
- What's good vs weak, and what we'd carry into our version.

Produce a markdown summary as a linked asset. (Feeds ticket 008.)

## Resolution

`kryptamine/herdr-auto-title` is a **single long-lived Go binary** (Go 1.24, MIT, v0.5.0, min herdr
0.8.2) with a **strong, well-measured architecture** worth mining. The mapping logic:

- **Surfaces:** `[[startup]]` only — a one-shot launch, **no `[[events]]`**. It rejects the event
  stream (measured ~10 s of backlog on subscribe, no cursor) and instead **polls `session.snapshot`
  twice a second**, renaming tabs whose computed label drifted.
- **Calls back into herdr via the socket, not the CLI:** raw newline-delimited JSON on
  `HERDR_SOCKET_PATH`. **No `os/exec`, no `HERDR_BIN_PATH`.** Exactly three methods:
  `session.snapshot`, `pane.process_info`, and **`tab.rename {tab_id, label}`** to rename.
- **Title computation:** a deterministic **confidence ladder** — agent title (90) → terminal title
  (80) → Claude-Code transcript (75) → foreground process (70) → ssh host (60) → git branch (40) →
  cwd (30) → `Shell` (10) — assembled as `<pos> · <context> › <branch> › <agent> › <activity>`.
  No LLM. Git branch read from `.git` files (not `git` subprocess); transcript reachable only via
  `herdr integration install claude`; all terminal-derived text sanitized as hostile input.
- **Config:** eight `HERDR_AUTO_TITLE_*` vars read **once at startup** from `config.env` in
  `os.UserConfigDir()/herdr-auto-title/` (deliberately not herdr's plugin config dir). **State:**
  minimal, snapshot-rebuilt; the one persisted piece is label-guarded **manual-rename protection**.
  **Deps:** two, transitive-free (`rivo/uniseg`, `joho/godotenv`).

**Carry into our version:** poll-not-events, `[[startup]]`+self-supervision, socket-rename, the
deterministic ladder, hostile-input sanitization, and manual-rename protection — plus its bar for
per-decision measured docs. **Do better on:** multi-agent transcript support (Claude-only today),
no-restart config reload, and consciously choosing our own title grammar. Fresh implementation, not
a fork (MIT vs our license).

Full write-up + source refs: [../assets/002-auto-title-mapping.md](../assets/002-auto-title-mapping.md).
Feeds ticket 008 (auto-title brief), still blocked by ticket 006 (conventions).
