# Spec: repo structure & shared conventions (ticket 006)

**Status:** Decided (2026-09-08). Builds on ticket 005 (topology) and ticket 004 (patterns).
The backbone every plugin conforms to. Compiled-binary build details are ticket 007.

## Decisions taken this ticket
- **Shared code → ① canonical `/shared` + sync** (fix-once, self-contained plugins, CI drift-check).
- **Config/state → herdr-native** (`$HERDR_PLUGIN_CONFIG_DIR` / `$HERDR_PLUGIN_STATE_DIR`).
- Everything else ratified from ticket 004's research (below).

## Repo layout

```
herdr-mihi/
  README.md                      # index; install table generated from catalog.json
  catalog.json                   # GENERATED from manifests (just catalog); consumed by picker + README
  justfile                       # single task runner (ci, sync-shared, check-sync, new-plugin, catalog)
  .githooks/commit-msg           # conventional-commits enforcement
  .github/workflows/
    ci.yml                       # push/PR -> just ci
    release.yml                  # per-plugin subtree-split -> tag <p>-vX.Y.Z + move <p>-latest (ticket 007)
  shared/                        # CANONICAL shared bits — EDIT HERE ONLY
    run.sh                       # PATH-fixing wrapper (generic; derives target from $HERDR_PLUGIN_ID)
    install.sh                   # prebuilt-binary downloader + source fallback (compiled plugins)
    templates/
      README.tmpl.md
      herdr-plugin.toml.tmpl
  plugins/
    <name>/                      # one plugin; self-contained; subtree-split-friendly
      herdr-plugin.toml          # manifest (see conventions)
      herdr/
        run.sh                   # SYNCED copy of /shared/run.sh — DO NOT edit here
        install.sh               # SYNCED copy — compiled plugins only
      src/ | main.go | *.sh      # implementation (language-specific; NOT shared)
      README.md                  # first line: "Platforms: macOS · Linux"
      .env.example               # committed, if configurable (.env is gitignored)
      .gitignore                 # bin/ target/ .env
```

## Conventions checklist (a new plugin must satisfy all)

**Manifest (`herdr-plugin.toml`)**
- [ ] `id = "herdr-mihi.<plugin>"`; also set `name`, `version` (semver), `min_herdr_version`, `description`, `platforms = ["macos","linux"]`.
- [ ] Every `command` points at the wrapper, absolute via `$HERDR_PLUGIN_ROOT`:
      `["sh","-c","exec bash \"$HERDR_PLUGIN_ROOT/herdr/run.sh\" <subcmd>"]` — never cwd-relative, never bare interpreter names (herdr runs shell-less with a minimal PATH).
- [ ] Compiled plugins: `[[build]] command = ["bash","herdr/install.sh"]`. Bash plugins: no build step.
- [ ] Actions/panes/events declared with fully-qualified refs (`herdr-mihi.<plugin>.<action>`).

**Entrypoint**
- [ ] One artifact, **argv-subcommand dispatch** (one binary/script, `match args[1]`), not one script per action.
- [ ] The manifest names **only** the wrapper; implementation can move behind it.

**Config & state — herdr-native**
- [ ] Config read from `$HERDR_PLUGIN_CONFIG_DIR/.env` (dotenv), falling back to plugin-root `./.env` for `herdr plugin link` dev. Commit **`.env.example`**; gitignore `.env`.
- [ ] Setup doc line: `cp .env.example "$(herdr plugin config-dir herdr-mihi.<plugin>)/.env"`.
- [ ] State written under `$HERDR_PLUGIN_STATE_DIR`, **atomic** (temp file → `rename()`), keys sanitized to safe path components.
- [ ] Tests redirect paths by setting `$HERDR_PLUGIN_CONFIG_DIR/$_STATE_DIR` themselves (no custom user-facing override var — we chose plain herdr-native).
- [ ] `.env` flags: `0|false|no|off` (case-insensitive) = false, else true.

**Robustness (adopt wholesale from 004)**
- [ ] **Exit 0 when not applicable** (missing config, non-actionable event, unsupported URL, missing optional tool) after a one-line stderr note. Non-zero only on genuine failure; distinct codes for distinct meanings.
- [ ] Diagnostics to **stderr, prefixed with the plugin id**; user-facing pane output to stdout.
- [ ] **Defensive** parsing of `$HERDR_PLUGIN_CONTEXT_JSON` / `$HERDR_PLUGIN_EVENT_JSON` — fields drift across herdr versions; never assume a field; malformed JSON → `{}`, not a throw.
- [ ] Config parse failure → defaults + one warning, never panic.

**README (every plugin)**
- [ ] **First line: `Platforms: macOS · Linux`** (our mandated addition — neither reference repo has it).
- [ ] Then: one-sentence purpose (+ language), **Setup** (install command + config-dir `.env`), **keybind block** (herdr binds no keys by default — always show `[[keys.command]]`), **Behavior**, **Requirements** (runtime versions + external tools).

**Shared code (option ①)**
- [ ] Universal glue lives canonically in `/shared`; `just sync-shared` copies it into each `plugins/*/herdr/`; `just check-sync` (in CI) fails on drift. Never edit the synced copies.

## Collection catalog (`catalog.json`)
- **Generated** from every plugin's manifest by `just catalog` (CI regenerates and fails if the committed copy is stale — single source of truth = the manifests).
- Per plugin: `id`, `name`, `description`, `language`, `platforms`, `latest` ref (`<plugin>-latest`), and available `versions`.
- Consumers: the **picker plugin (ticket 010)** and the **README install table**.

## Versioning
- **Per-plugin semver** via conventional commits scoped to the plugin (`feat(auto-title): …`).
- Release automation produces per-plugin tags `<plugin>-vX.Y.Z` and moves `<plugin>-latest` (mechanism = ticket 005; CI implementation = ticket 007). release-please (multi-package) keeps `herdr-plugin.toml` `version` in sync.

## Tooling / CI
- **`justfile`** is the single task runner shared by humans and CI: `just ci` (fmt/lint/test per plugin + `check-sync` + catalog-staleness check), `just new-plugin <name>` (scaffold from `/shared/templates` + `sync-shared`), `just sync-shared`, `just catalog`.
- `.githooks/commit-msg` enforces conventional commits. `ci.yml` on push/PR runs `just ci`.

## Handoffs
- **Ticket 007** — the exact contents of `/shared/install.sh` (prebuilt-binary download + source fallback), the `release.yml` matrix, and the language-choice heuristic.
- **Tickets 008/009/010** — write each plugin's brief against this checklist.
