# lazygit

**Platforms: macOS · Linux**

A git popup running **your own** lazygit, scoped to the current git worktree. Thin, pure-bash glue —
lazygit owns all the git UI.

## Install
```sh
herdr plugin install luisvinicius09/herdr-mihi --ref lazygit-latest
```
Requires `lazygit` on your `PATH` (`brew install lazygit`). Nothing is bundled or downloaded.

## Setup (optional)
```sh
cp .env.example "$(herdr plugin config-dir herdr-mihi.lazygit)/.env"
```

## Keybinding
herdr binds no keys by default. Add to your herdr `config.toml`, then `herdr server reload-config`:
```toml
[[keys.command]]
key = "prefix g"
type = "plugin_action"
action = "herdr-mihi.lazygit.open"
```
Over `--remote`, bindings need `--remote-keybindings server` or they silently no-op.

## Behavior
- `open` resolves the current worktree (focused pane's cwd → workspace cwd → `$HOME`, then the git top-level)
  and opens a lazygit **popup** rooted there. Quit lazygit (`q`) to close it.
- If lazygit fails to start (or is missing), the popup **stays open showing the error** instead of vanishing.
- v1 opens a fresh popup each time; it does not yet *summon* an already-open one. (herdr's `pane open` CLI returns
  no pane id, so de-duplicating needs socket pane-querying — a planned enhancement, shared with auto-title's engine.)

## Capability declaration (trust)
- **Spawns:** `lazygit` (yours), `git` (`rev-parse` for the worktree root), `herdr` CLI (`plugin pane open`).
- **Network:** none.
- **Files:** reads `$HERDR_PLUGIN_CONFIG_DIR/.env`. **Writes nothing** (no state).
- **No** python, **no** bundled/downloaded binaries, **no** telemetry, **no** writes to your lazygit config.

## Requirements
`lazygit`, `git`, `bash`, `herdr ≥ 0.8`. macOS + Linux.

## Config (`.env`)
See `.env.example`: `LAZYGIT_BIN`, `POPUP_WIDTH`, `POPUP_HEIGHT`.
