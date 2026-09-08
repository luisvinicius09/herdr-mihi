# Research: how herdr-auto-title maps herdr surfaces to behavior

**Ticket:** 002 · **Type:** research · **Source:** [`kryptamine/herdr-auto-title`](https://github.com/kryptamine/herdr-auto-title)
@ `899ee4e` (2026-09-08), **Go 1.24, MIT, v0.5.0**, `min_herdr_version = 0.8.2`.
Primary evidence: the repo's own `docs/architecture/*` (measured against a live Herdr 0.8.2, protocol
20; Windows against 0.8.2-preview, protocol 22) plus the source under `internal/`.

## Bottom line

Auto Title is a **single long-lived Go binary** that Herdr starts once via a `[[startup]]` hook, then
**polls the whole session over the Herdr socket twice a second** and renames tabs whose computed title
has drifted. It uses **exactly one manifest surface — `[[startup]]` — and no `[[events]]` at all**
(the event stream is deliberately rejected; see below). It talks to Herdr over **raw newline-delimited
JSON on `HERDR_SOCKET_PATH`, never the `herdr` CLI** — confirmed: no `os/exec`, no `HERDR_BIN_PATH` use
anywhere. Only **three socket methods** are used: `session.snapshot`, `pane.process_info`, `tab.rename`.

A title is computed by a **deterministic confidence ladder** of eight sources (agent title → terminal
title → transcript → process → ssh → git branch → cwd → `Shell` fallback), assembled as
`<position> · <context> › <branch> › <agent> › <activity>`. **No LLM, no external service, no
scrollback scanning.** Config is eight `HERDR_AUTO_TITLE_*` vars read once at startup from a
`config.env` file (not Herdr's plugin config dir). State kept between polls is minimal and all
reconstructable. It is a genuinely strong reference — well-measured, well-documented — and the main
things to carry are its *architecture* (poll-not-events, deterministic ladder, manual-rename
protection, hostile-input sanitization) rather than its exact title grammar.

---

## 1. Herdr surfaces used (and how it renames)

**Manifest (`herdr-plugin.toml`) declares only two kinds of entry:**

- `[[build]]` — `go build -o herdr-auto-title ./cmd/herdr-auto-title` (a `.exe` twin for Windows).
  Built at install time; **Go is the only build dependency**.
- `[[startup]]` — `./herdr-auto-title`. **A one-shot launch, not a supervised daemon.** Herdr
  spawns it and forgets it (no handle kept), so the *process itself* must stay alive, survive a
  Herdr outage, and detect its own supersession.

**No `[[events]]`, `[[actions]]`, `[[panes]]`, or `[[link_handlers]]`.** This is the single most
important mapping decision: rather than subscribe to events, it **polls `session.snapshot`**.

**How it renames:** via the socket method **`tab.rename {tab_id, label}`** — *not* a `herdr` CLI
command. The whole plugin issues only three methods (`internal/herdr/protocol.go`):

| Method | Role | Measured cost |
|---|---|---|
| `session.snapshot` | whole session in one request (workspaces, tabs+labels, panes+dirs+titles+agent+status) | 0.47 ms / 6 KB for 6 panes |
| `pane.process_info` | foreground process group of **one** pane (name, argv, cwd), deepest-first | ~0.11–0.17 ms, **per pane** |
| `tab.rename` | set a tab's label | 0.16 ms median |

Transport: newline-delimited JSON, `{"id","method","params"}` → `{"id","result"|"error"}`, **one
request per connection** (Herdr closes after answering), so there is *no* reconnect logic. On Windows
the socket is a named pipe at `\\.\pipe\<HERDR_SOCKET_PATH>`. A label is **one line** — the tab bar
renders a single row.

### Why polling, not events (a measured decision worth inheriting)

`events.subscribe` **replays ~10 s of per-pane backlog before any live event**, paces it ~10/s, gives
no cursor to skip it, and events carry no timestamp/sequence. A change made 2 s after subscribing was
observed arriving 13 s later. A snapshot instead "describes the present," costs one request, and
carries every field the resolver needs. The repo says in bold: **do not reintroduce a subscription
without re-measuring.**

---

## 2. How a title is computed

### The poll loop (`internal/app/app.go`)

Every `POLL_MS` (default 500 ms): `session.snapshot` → note which panes' **revisions** advanced →
`Manual.Retain` (drop bookkeeping for gone tabs, release locks whose label moved) → assemble tabs from
the snapshot → for each **unlocked** tab: pick its context pane, read that one pane, resolve a title,
and **rename only if the label differs** (dedup keeps the loop quiet). The interval *is* the rename
rate: a tab changes name at most once per poll.

**Decide from freshly read state**: each poll reads the whole session and throws it away. Only four
things are carried between polls, each because a snapshot cannot express it:
1. **when each pane last changed** (from monotonic revisions) — to name a multi-pane tab after the
   pane that moved last;
2. **what each pane was running** (cache of `pane.process_info`, refreshed when revision moves or
   every 2 s) — avoids a per-pane process request every poll;
3. **how far each agent transcript has been read** (path + byte offset + topic) — transcripts are
   append-only, so only new bytes are read; a *not-found* session is re-searched only every 10 s;
4. **what Auto Title last named each tab** — the basis of manual-rename protection.

### The confidence ladder (`internal/resolver/resolver.go`)

Sources declare their own confidence; the resolver sorts by it. A title has four fields
`Parts{Context, Branch, Agent, Activity}`. **A source never overrides a field a higher one already
filled, but a lower one can complete the other half** — so a cwd (30) fills the *context* of a title
whose *activity* came from an agent (90).

| Conf | Source | Fills | What it contributes |
|---:|---|---|---|
| 90 | Agent title (`PaneInfo.title`) | Activity | agent's own reported topic (usually null for Claude Code → arrives at 80) |
| 80 | Terminal title (`terminal_title_stripped`) | Activity | richest source in practice; carries most agent context |
| 75 | Session transcript | Activity | reads Claude Code's transcript when it never titled its terminal |
| 70 | Foreground process | Activity | a **lone** program's kind (`nvim`); a shell = no activity |
| 60 | SSH | **Context** | `ssh › prod-01` — the *host*, parsed from argv, not the launch dir |
| 40 | Git branch | **Branch** | qualifies context; read from `.git` files, never by running git |
| 30 | Working directory | Context | basename of the pane's dir (project name) |
| 10 | Generic fallback | whole name | `Shell` |

**Which pane speaks for a multi-pane tab** (`SelectContextPane`, `internal/state/tab.go`): focused
pane → else a pane whose agent is `working`/`blocked` → else the pane that changed most recently (ties
break on recency then pane ID, so it's deterministic).

Notable per-source cleverness:
- **Git branch is read from files under `.git`** (`HEAD`, refs), *not* by shelling `git rev-parse`
  (12.37 ms vs 0.019 ms). Trunk contributes nothing (trunk read from `refs/remotes/origin/HEAD`, not a
  hardcoded `main`); an issue key is extracted when a branch is too long
  (`bugfix-…-mc-13675` → `MC-13675`); detached HEAD shows the short hash *except* mid-rebase; worktrees
  and submodules are followed through `gitdir:`.
- **cwd** prefers the *foreground process's own* cwd (from `pane.process_info`) over the snapshot's
  `cwd`/`foreground_cwd`, because both snapshot fields lie in real cases (a subshell strands `cwd`; an
  agent's MCP server strands `foreground_cwd` in the server's dir).
- **Transcript** reads Claude Code's `ai-title` (last wins) or, failing that, the **first prompt the
  user actually typed** (`origin.kind:"human"`), turning slash-command-only sessions from a permanent
  `claude` into a real name. Reachable only because `herdr integration install claude` writes a
  `SessionStart` hook that reports the session id into `PaneInfo.agent_session` — *not* something the
  plugin installs. Format is undocumented → treated as best-effort; if it breaks, the source declines.
- **Repetition & framing:** the workspace name is dropped from a tab in that workspace (Herdr already
  shows it above the tabs); the tab **position** (not `TabInfo.number`) leads the title as a `·`
  decorator and is counted against `MaxLength`, not added to it, because truncation cuts the tail.

### Sanitization — every candidate value is treated as hostile (`sanitize.go`)

Everything comes from terminal output / agent-written files, so it is stripped of ANSI escapes,
control/format chars (RTL overrides, zero-width spaces, bidi isolates — the ZWJ is kept for emoji),
collapsed whitespace/separators, and truncated **by terminal column width via grapheme clusters**
(`rivo/uniseg`), not rune count. `Meaningful` rejects bare paths, program/shell names, and shell
prompts as "saying nothing." Iron rule: **nothing derived from terminal output is ever passed to a
shell — the plugin runs no subprocess at all.**

---

## 3. Config, state, dependencies

**Config — eight `HERDR_AUTO_TITLE_*` vars, read once at startup** (`internal/app/config.go`) from a
`config.env` file loaded into the process env via `joho/godotenv` (env wins over file). The file lives
in `os.UserConfigDir()/herdr-auto-title/config.env` (macOS `~/Library/Application Support/…`, Linux
`~/.config/…`, Windows `%APPDATA%\…`) — **deliberately NOT** Herdr's `HERDR_PLUGIN_CONFIG_DIR`, because
that only exists when the server starts the plugin (so `make run` would see a different dir) and would
split the plugin's files. **Not reread while running** — a settings change needs `herdr server stop`.
A bad value warns and keeps the default; a single bad *line* voids the whole file.

| Setting | Default | Meaning |
|---|---|---|
| `DEBUG` | `false` | log at DEBUG |
| `POLL_MS` | `500` | poll interval (also the rename rate & calm knob) |
| `MAX_LENGTH` | `50` | longest title, in **columns** |
| `BRANCH_MAX` | `12` | longest branch in a title; `0` = no branches |
| `POSITION` | `true` | prefix each tab's position |
| `MANUAL_FILE` | `manual-names.json` next to `config.env` | where manual renames persist; empty = memory-only |
| `TRANSCRIPT` | `true` | read an agent's transcript when it hasn't titled its terminal |
| `AGENT_NAME` | `true` | show the agent's name in front of its activity |

**State handling** — all under `internal/state/`, all rebuilt from each snapshot (closed tabs/panes
drop out for free). The one *persisted* piece is **manual-rename protection** (`manual.go`): a tab is
"the user's" when its label moved to something the resolver neither set nor would set. Locks persist to
`manual-names.json` (temp-file + rename) so a mid-session plugin restart doesn't lose manual names, but
**each lock records the label it was taken with** and releases if the tab no longer carries it (Herdr
tab ids are per-session and can be reused). Clearing a tab's name hands it back on the next poll.

**Dependencies — only two, both tiny and transitive-free:** `rivo/uniseg` (grapheme width) and
`joho/godotenv` (env file). That's the whole `go.mod`.

**Robustness worth noting:** no poll failure is fatal (Herdr's socket can lag a just-launched plugin);
failure logging backs off exponentially; and each instance learns the socket's identity
(dev+inode on Unix, pid+start-time marker on Windows) and **exits cleanly when a new server takes the
socket** — because Herdr's one-shot startup hook means a stopped server leaves the old plugin running
and the new server starts a fresh one.

---

## 4. What's good vs weak — and what to carry into our version

**Strong (carry these — they are architecture, not cosmetics):**
- **Poll `session.snapshot`, don't subscribe to events** — measured, decisive, and it makes the whole
  design stateless and restart-safe. This is the core mapping pattern for any "keep X in sync" plugin.
- **`[[startup]]` one-shot + self-supervision** (survive outages, detect supersession via socket
  identity, exit 0 on handoff). Any long-lived herdr plugin needs this exact shape.
- **Rename over the socket (`tab.rename`), never the CLI** — no subprocess, nothing shellable.
- **Deterministic confidence ladder** with independent field-filling — clean, testable, extensible
  (the numeric gaps leave room for new sources). Great model for our agent-status dashboard too.
- **Treat all terminal-derived text as hostile** (ANSI/format-char stripping, column-width truncation)
  — a reusable concern across the whole plugin collection; candidate for a **shared internal package**.
- **Manual-rename protection from successive polls** (label-guarded persisted locks) — the non-obvious
  correctness work; a "don't fight the user" pattern we'll want anywhere we mutate user-visible state.
- **Cost discipline:** per-tab (not per-pane) reads, revision-gated process cache, in-poll memoization,
  dedup-before-rename. And **exemplary docs** — every decision paired with the measurement that settled
  it; a bar worth matching for ticket 006's conventions.

**Weak / gaps / opportunities to do better (for ticket 008):**
- **Transcript reading is Claude-Code-only** and depends on undocumented internals (`ai-title`,
  `origin.kind`). Herdr ships integration hooks for 17 agents but only Claude's transcript format is
  parsed. Room to support more agents, or lean on `pane.report_metadata` instead.
- **Config isn't reread** — every change needs a full `herdr server stop`. A control-channel `reset`
  is *specified but not built*; likewise a `command → friendly name` map (`yarn dev` → `Dev`).
- **On Windows the process & ssh sources go silent** (Herdr lists only shell/agent there), so
  `nvim › file` and `ssh › host` titles don't appear. Our macOS/Linux-only v1 sidesteps this, but note
  the platform-shaped behavior lives in one place.
- **License is MIT**, ours will differ (herdr ecosystem trends Apache-2.0) — a fresh implementation,
  not a fork; mine the *architecture*, write our own code.
- **Title grammar is opinionated** (`·`/`›`, position-first, workspace elision). Good defaults, but
  our brief should decide consciously whether to match or diverge — it's the most visible surface and
  the cheapest place to differentiate.

---

## Leads for ticket 008 (auto-title brief) — not decided here

- Decide our **language** against ticket 007's heuristic — Go is a strong fit here (single static
  binary, trivial `[[build]]`, cross-compiles to Windows for free) and matches the reference, easing
  comparison; but confirm against our own rule.
- Decide **how far to match vs improve** the title grammar, and whether to broaden agent-transcript
  support beyond Claude Code or rely on `pane.report_metadata`.
- Consider factoring **sanitization** and the **socket client / poll-loop skeleton** as shared code, if
  the dashboard (phase 2) and other plugins will also poll-and-render — a topology/conventions question
  that touches tickets 005/006.
