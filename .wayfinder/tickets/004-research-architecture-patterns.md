---
id: 004
title: "Research: architecture patterns from herdr-nvim & the official cookbook"
labels: [wayfinder:research]
status: closed
assignee: luisvinicius0906@gmail.com
blocked_by: []
---

## Question

Study [ChmaraX/herdr-nvim](https://github.com/ChmaraX/herdr-nvim) (the user flagged it as
well-structured / "logically makes sense") and the official cookbook
[ogulcancelik/herdr-plugin-examples](https://github.com/ogulcancelik/herdr-plugin-examples).
Extract **reusable patterns** to inform our repo structure & conventions:

- Per-plugin directory layout and `herdr-plugin.toml` conventions.
- How config (`HERDR_PLUGIN_CONFIG_DIR`) and state (`HERDR_PLUGIN_STATE_DIR`) are handled.
- Entrypoint patterns (argv command, wrapper scripts), error handling, logging.
- Build steps declared in the manifest; any testing.
- Anything about how these repos organize *multiple* plugins vs one.

Produce a markdown summary as a linked asset — a "patterns to adopt / avoid" list.
(Feeds tickets 006 and 007.)

## Resolution

Both repos cloned and read directly ([findings](../assets/004-architecture-patterns.md)). The
cookbook has **four** plugins, not three — `rust-release-check` (JS/Bash/Lua/Rust, one each).
The two repos sit at opposite ends of a maturity spectrum, and the gap between them *is* our
convention set.

Patterns to **adopt** (→ 006/007):
- **Wrapper shim** (`herdr/run.sh`): fixes PATH + resolves `$HERDR_PLUGIN_ROOT/bin/<bin>`; the
  manifest names only the wrapper. herdr runs commands **shell-less with a minimal PATH**, so
  bare `node`/`lua`/`gh` + cwd-relative paths (the cookbook's style) are brittle.
- **One artifact per plugin, argv-subcommand dispatch** (`main.rs match args[1]`), not one
  script per action.
- **id = `<collection>.<plugin>`**; always set `platforms`/`min_herdr_version`/`version`.
- **Error convention (uniform, strong):** `exit 0` when not applicable (missing config,
  uninteresting event, missing optional tool), non-zero only on real failure; **defensive**
  parsing of `HERDR_PLUGIN_CONTEXT_JSON`/`EVENT_JSON` (fields unstable across versions); config
  errors → defaults, never panic.
- **Compiled distribution → prebuilt per-OS binary download + source fallback** (`install.sh`
  "reviewr pattern"), backed by a GH-Releases build matrix + **release-please** that syncs the
  manifest `version` to `Cargo.toml`. Beats build-on-install (`rust-release-check`), which
  needs `cargo` on the user's machine. **bash plugins need no build at all.** (→ 007)
- **`justfile` task runner** + push/PR CI + conventional commits; hermetic tests via
  env-overridable config/state paths.

The **one live decision** for 006: config/state placement. The cookbook uses the herdr-injected
`HERDR_PLUGIN_CONFIG_DIR/.env` (+ committed `.env.example`) and `HERDR_PLUGIN_STATE_DIR`;
herdr-nvim **ignores both** and rolls its own `~/.config/herdr-nvim` + `XDG_STATE_HOME`.
**Recommend defaulting to the herdr-native dirs** (works with `herdr plugin config-dir <id>`);
herdr-nvim's choice is justified only by its non-herdr nvim consumer. Also confirmed: **neither
repo has the "Platforms: macOS · Linux" README line** the map mandates — it's our addition, and
**shared-code-across-plugins is unsolved** (cookbook says "just copy it"; herdr's full-copy
install means any shared code must be vendored per plugin — an open 006 question tied to 005).
