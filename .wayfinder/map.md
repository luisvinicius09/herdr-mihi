<!-- wayfinder:map -->
# Map: herdr plugin collection ("plugin of plugins")

## Destination

A **handoff-ready spec** (plan only — no code written during the map) for a modular
herdr plugin collection the user can pick from. The spec locks: the **repo topology &
install-footprint mechanism**, a **clean repo structure & shared conventions**,
**language + build/distribution rules** (rust / go / bash), and a **per-plugin brief**
for each plugin — starting with `auto-title` and `lazygit`. The map is done when
someone could open it and build any phase-1 plugin without further decisions.

## Notes

**Domain — herdr plugins** (source: https://herdr.dev/docs/plugins/)
- herdr is a terminal multiplexer (tmux/zellij alternative), v0.9.0, Apache 2.0, runs on macOS · Linux · Windows (Windows beta).
- A plugin = a `herdr-plugin.toml` manifest + one or more executables in **any argv language** (bash, JS, Lua, rust, go…). There is **no SDK** — the whole `herdr` CLI *is* the plugin API.
- herdr runs the executable as a subprocess (no shell), cwd = plugin dir, and injects env vars: `HERDR_BIN_PATH`, `HERDR_SOCKET_PATH`, `HERDR_PLUGIN_ID`, `HERDR_PLUGIN_ROOT` (read-only source), `HERDR_PLUGIN_CONFIG_DIR` (user config), `HERDR_PLUGIN_STATE_DIR` (durable state), `HERDR_PLUGIN_CONTEXT_JSON` (workspace/tab/pane/worktree/agent/selection/URL), and event/action ids.
- Manifest surfaces: `[[startup]]`, `[[events]]` (e.g. `worktree.created`, delivered via `HERDR_PLUGIN_EVENT_JSON`), `[[actions]]`, `[[panes]]` (overlay/popup/split/tab/zoomed), `[[link_handlers]]`.
- Install: `herdr plugin install owner/repo/subdir` (git clone → preview → build → store) or `herdr plugin link /path` (dev, no build). **No managed storage API** — plugins own their state.
- Official cookbook: `ogulcancelik/herdr-plugin-examples` (`agent-telegram-notify`, `github-link-preview`, `dev-layout-bootstrap`).

**Constraints & decided scope**
- **Modular** — a "plugin of plugins" where the user chooses what to use.
- **Install footprint** — *strongly prefer* not bringing code the user didn't choose onto their machine (not absolute; a simpler monorepo tradeoff is acceptable if research shows it's far easier to maintain). Drives ticket 005.
- **Platforms: macOS + Linux for v1.** Windows out of scope (see below). The repo/plugin READMEs **must state "Platforms: macOS · Linux"**.
- **Languages** — rust / go / bash allowed, chosen per plugin. rust/go cross-compile to Windows for ~free, keeping that door open later.
- **Clean, easy-to-follow repo structure** is a first-class goal.

**Skills every session should consult**
- `grill-me` (grilling) + `domain-modeling` — the default for decision tickets.
- `research` — for the AFK research tickets.
- `prototype` — when "how should it look/behave" is the key question (esp. the dashboard).

**Reference plugins to mine** (the user's "mapping logic")
- Phase 1 focus: [herdr-auto-title](https://github.com/kryptamine/herdr-auto-title) · [herdr-lazygit](https://github.com/Crokily/herdr-lazygit) (do it better)
- Architecture reference: [herdr-nvim](https://github.com/ChmaraX/herdr-nvim) (well-structured, "logically makes sense")
- Later phases: [herdr-radar](https://github.com/hhdebb/herdr-radar) · [space-colors](https://github.com/ferretorres/herdr-plugin-space-colors) · [herdr-navigator](https://github.com/thanhdat77/herdr-navigator)

**Phasing**
1. `auto-title`, `lazygit`, `picker` (install/selection UX — ticket 010)
2. Non-disruptive agent-status dashboard (flagship idea — expand later)
3. `nvim`, `radar`, `space-colors`, `navigator`

**Tracker** — local-markdown (no GitHub remote yet). Map + tickets live under `.wayfinder/`; see `.wayfinder/README.md` for how frontier / claim / blocking work here. Migrate to GitHub Issues once the repo is pushed.

## Decisions so far

<!-- the index — one line per closed ticket: enough to judge relevance, then open the link for detail -->

- [Research: how herdr fetches & stores a subdir plugin install](tickets/001-research-herdr-install-footprint.md) — herdr shallow-clones the **whole repo** per subdir install and stores a **full copy per plugin-id** (no sparse checkout); a monorepo brings all unchosen code × N. Honoring the footprint preference needs **separate repos or per-plugin release refs**, not a plain monorepo. ([findings](assets/001-herdr-install-footprint.md))
- [Research: how herdr-auto-title maps herdr surfaces to behavior](tickets/002-research-auto-title-mapping.md) — a Go binary that uses **`[[startup]]` only (no events)**, **polls `session.snapshot`** twice a second, and renames via the socket method **`tab.rename`** (never the `herdr` CLI). Title = a deterministic **confidence ladder** (agent→terminal→transcript→process→ssh→git→cwd→`Shell`). Carry: poll-not-events, self-supervising startup, socket-rename, the ladder, hostile-input sanitization, manual-rename protection. Do better: multi-agent transcripts, no-restart config, our own title grammar. ([findings](assets/002-auto-title-mapping.md))
- [Decide: repo topology & install-footprint mechanism](tickets/005-decide-repo-topology.md) — **dev monorepo of independently-installable plugins**, published as **per-plugin release refs** (CI subtree-split → immutable tag `<name>-vX.Y.Z` + moving `<name>-latest`), installed via `herdr plugin install you/herdr-mihi --ref <name>-latest`. Selection via an in-repo **picker plugin** (now a v1 deliverable → ticket 010). One repo, per-plugin language, minimal footprint. ([decision](assets/005-topology-decision.md))
- [Research: how herdr-lazygit works, and where it falls short](tickets/003-research-lazygit.md) — `Crokily/herdr-lazygit` is a polished ~3,500-line bash+**python3** reference: `[[panes]]` split + `open`/`open-tab` `[[actions]]`, worktree-scoped idempotent toggle, pinned lazygit+fzf runtime, AI-commit + fzf settings pane, **keybinding NOT auto-registered**. "Done better" = **simpler**: lean core (no python3, no bundled binaries; AI-commit optional), prefer the user's own lazygit, auto-register the key if herdr allows — keep its worktree-scoped toggle & readable-error handling. ([findings](assets/003-lazygit-reference.md))
- [Research: architecture patterns from herdr-nvim & the official cookbook](tickets/004-research-architecture-patterns.md) — the cookbook (4 plugins, 1 per language) is minimal; **herdr-nvim is the production template**. Adopt: **PATH-fixing wrapper shim** (`herdr/run.sh`) since herdr runs commands shell-less with a minimal PATH; one-artifact + argv-subcommand dispatch; `<collection>.<plugin>` ids; **exit-0-when-not-applicable** error convention + defensive context-JSON parsing; **prebuilt-binary-download-with-source-fallback** dist (→007, beats build-on-install) + release-please syncing manifest version; `justfile` CI. **Open for 006:** config/state placement — cookbook uses herdr-injected `HERDR_PLUGIN_CONFIG_DIR/.env` + `_STATE_DIR` (recommended default) vs herdr-nvim's own `~/.config` namespace; plus the mandated "Platforms: macOS · Linux" README line and shared-code-vs-copy are unsolved. ([findings](assets/004-architecture-patterns.md))

## Not yet specified

<!-- in-scope fog: real but not yet sharp enough to ticket; graduates as the frontier advances -->

- **Non-disruptive agent-status dashboard** (phase 2, flagship idea) — a place to go *while waiting for agents to work* that surfaces their status **without disrupting the agents' current state**. Not yet sharp: what it shows, how you reach it (pane? overlay?), what "non-disruptive" means concretely. Will likely graduate into (a) research on what agent lifecycle/status herdr exposes to plugins (events, `HERDR_PLUGIN_CONTEXT_JSON` agent fields, CLI) and (b) a `/prototype` of the layout. May borrow from `herdr-radar` (overview) and `herdr-navigator` (jumping) patterns.
- **Later-phase plugin briefs** — `nvim`, `radar`, `space-colors`, `navigator`. Each graduates into its own research + brief once phase 1 and the dashboard are settled.

## Out of scope

<!-- ruled beyond the destination; closed, never graduates unless the destination is redrawn -->

- **Windows support** — herdr runs on Windows (beta), but v1 targets **macOS + Linux only**. Keeps bash viable and avoids Windows shims / cross-platform binary machinery. Revisit only if wide sharing to Windows users becomes a goal (rust/go plugins could add it later with near-zero rework).
