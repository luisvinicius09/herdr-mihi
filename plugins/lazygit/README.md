# lazygit

**Platforms: macOS · Linux**

A git **popup** running **your own** lazygit, rooted at the current git worktree. Thin, pure-bash
glue — lazygit owns all the git UI.

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
**Quickest:** `herdr plugin action invoke herdr-mihi.lazygit.setup` adds the binding (default `prefix+g`) to
your `config.toml` and reloads — idempotent, prints exactly what it added, and warns instead of clobbering if
the key is already taken.

Or do it by hand — herdr binds no keys by default, so add to your `config.toml` then `herdr server reload-config`:
```toml
[[keys.command]]
key = "prefix+g"
command = "herdr plugin action invoke herdr-mihi.lazygit.open"
```
(herdr `[[keys.command]]` entries run a **shell command** — use the absolute path to `herdr`, e.g.
`/opt/homebrew/bin/herdr`, if your keybinds run with a minimal PATH.) Over `--remote`, bindings need
`--remote-keybindings server` or they silently no-op.

## Behavior
- `open` resolves the current worktree (focused pane's cwd → workspace cwd → `$HOME`, then the git top-level)
  and opens a lazygit **popup** rooted there — a floating overlay that covers the workspace **without
  rearranging your split**. Quit lazygit (`q`) to close it. Because it's rooted at the current worktree,
  opening it in workspace A acts on A's repo, in B on B's.
- If lazygit fails to start (or is missing), the popup **stays open showing the error** instead of vanishing.
- **Choose how it opens** (`PLACEMENT` in `.env`, default `popup`; or bind the **`open-side`** action):
  - `popup` — floating overlay; covers the workspace, doesn't rearrange your split; **session-modal**
    (one at a time, not persistent — a herdr limitation).
  - `side` — a **split** pane beside your work; workspace-tied and persists when you switch away.

## Capability declaration (trust)
- **Spawns:** `lazygit` (yours), `git` (`rev-parse` for the worktree root), `herdr` CLI (`plugin pane open`).
- **Network:** none.
- **Files:** reads `$HERDR_PLUGIN_CONFIG_DIR/.env`; writes nothing at runtime. The opt-in `setup` action
  appends a keybind to your herdr `config.toml` (shown first, idempotent) and reloads. With `ESC_QUIT=on`
  it sets `LG_CONFIG_FILE` to layer a **read-only** overlay (`esc-quit.yml`) onto your lazygit config.
- **No** python, **no** bundled/downloaded binaries, **no** telemetry, **no** writes to your lazygit config.

## Requirements
`lazygit`, `git`, `bash`, `herdr ≥ 0.8`. macOS + Linux.

## Config (`.env`)
See `.env.example`: `LAZYGIT_BIN`, `PLACEMENT` (`popup` or `side`), `ESC_QUIT` (Esc quits at the top
level, like `q`), and `POPUP_WIDTH`/`POPUP_HEIGHT` (popup only).

**`ESC_QUIT=on`** makes **Esc** close lazygit at the top level (same as `q`); Esc still cancels/backs out
inside submenus. It works by *layering* a tiny read-only overlay onto your lazygit config via
`LG_CONFIG_FILE` (lazygit's own merge mechanism) — **your lazygit config file is never modified**.
