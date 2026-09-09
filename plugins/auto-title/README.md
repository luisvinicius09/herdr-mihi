# auto-title

**Platforms: macOS · Linux**

Tab titles that follow the work in each tab — deterministic, **no network, no LLM, no subprocess**.
A single long-lived Rust daemon herdr starts once; it polls the session over herdr's socket and renames
tabs whose computed title has drifted.

## Install
```sh
herdr plugin install luisvinicius09/herdr-mihi --ref auto-title-latest
```

## Setup (optional)
```sh
cp .env.example "$(herdr plugin config-dir herdr-mihi.auto-title)/.env"
# then restart the herdr server so the daemon re-reads config
```

## Behavior
- Runs on `[[startup]]`; polls `session.snapshot` every `POLL_MS` (default 500 ms) and renames a tab only
  when its computed title changes (deduped, so the tab bar stays quiet).
- **Agent panes** → `agent › what it's doing` (e.g. `claude › Add observation fields`). **Other panes** →
  `project › activity` (just `project` when idle; the project is dropped when it's merely the current
  workspace's name, since herdr already shows the workspace above the tabs). No git branch / transcript / LLM in v1.
- **Manual-rename protection:** if you rename a tab yourself, auto-title backs off and stops touching it.
- All terminal-derived text is treated as hostile (ANSI/control stripped, truncated by column width); noise
  titles — idle `user@host` prompts, bare paths, plain shell names — are ignored.

## Actions
- **reset** — hand a tab you renamed by hand back to auto-title. It re-adopts the **focused** tab and resumes
  naming it; invoked over the CLI (no focused tab) it resets **all** tabs.
  ```sh
  herdr plugin action invoke herdr-mihi.auto-title.reset
  ```
  Or bind it in your herdr `config.toml`:
  ```toml
  [[keys.command]]
  key = "prefix R"
  type = "plugin_action"
  action = "herdr-mihi.auto-title.reset"
  ```

## Capability declaration (trust)
- **Spawns:** nothing. No subprocess, no shell.
- **Network:** none. Talks to herdr over its **local unix socket** (`$HERDR_SOCKET_PATH`) only — never the network, never the `herdr` CLI.
- **Files:** reads `$HERDR_PLUGIN_CONFIG_DIR/.env`; writes small state under `$HERDR_PLUGIN_STATE_DIR`
  (`daemon.pid`, `owned.tsv`, transient `reset.request`) and a diagnostic log (`auto-title.log` in the config dir).
- **No** LLM, **no** telemetry, **no** bundled binaries beyond the plugin's own.

## Requirements
`herdr ≥ 0.8`. macOS + Linux. (Built from source needs a Rust toolchain; prebuilt binaries otherwise.)

## Config (`.env`)
See `.env.example`: `POLL_MS`, `MAX_LENGTH`, `DEBUG`.

## Dependencies (audited)
`serde`, `serde_json` (JSON), `unicode-width` (column-accurate truncation). No networking crates —
enforced by `cargo-deny` in CI.
