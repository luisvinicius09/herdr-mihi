# picker

**Platforms: macOS · Linux**

Browse the herdr-mihi collection and install/update/uninstall plugins from a popup — **showing you the
exact commands before it runs anything.** The collection's trust-visible front door.

## Install (bootstrap)
```sh
herdr plugin install luisvinicius09/herdr-mihi --ref picker-latest
```
Then open it (bind a key, below) and install the rest.

## Keybinding
herdr binds no keys by default. Add to your herdr `config.toml`, then `herdr server reload-config`:
```toml
[[keys.command]]
key = "prefix p"
type = "plugin_action"
action = "herdr-mihi.picker.open"
```
Over `--remote`, bindings need `--remote-keybindings server`.

## Behavior
- Opens a **popup** listing the catalog with each plugin's state: `o available`, `* installed vX`, or
  `^ update vX->vY`.
- Navigate ↑/↓ (or j/k), `space` to select, `a` for all, `enter` to continue.
- A **confirm screen shows the exact `herdr plugin install … --ref <name>-latest` / `uninstall` commands**
  and runs **nothing** until you press `y`. Install/update for available/outdated plugins, uninstall for
  installed ones.
- Then it runs each via the herdr CLI and reports the result.

## Capability declaration (trust)
- **Spawns:** the `herdr` CLI only — `plugin list` (state), `plugin install`/`uninstall` (your picks),
  `plugin pane open` (to open itself). No other subprocess.
- **Network:** only what `herdr plugin install` fetches from the **pinned** `luisvinicius09/herdr-mihi`.
  Never an arbitrary owner/repo/URL.
- **Files:** reads `catalog.json` (bundled, or the repo root in dev) and `$HERDR_PLUGIN_CONFIG_DIR/.env`.
- **No** AI/LLM, **no** telemetry. It shows every command before running it.

## Requirements
`herdr ≥ 0.8`. macOS + Linux.

## Config (`.env`)
See `.env.example`: `POPUP_WIDTH`, `POPUP_HEIGHT`, and `ICONS` — the state glyphs default to **Nerd Font**
icons; set `ICONS=unicode` (or `ascii`) if your terminal font isn't a Nerd Font and you see boxes.

## Dependencies (audited)
`ratatui` + `tui-big-text` (TUI + banner), `serde` / `serde_json` (catalog). No networking crates.
