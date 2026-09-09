# Brief: lazygit plugin (`herdr-mihi.lazygit`)

**Status:** Ready to build (2026-09-08). Grounds: ticket 003 (reference research), 006 (conventions), 007 (language/build), the **trust-first** map principle.
**Driver:** a lean, fully-auditable git-in-a-pane you can trust — thin `herdr ↔ lazygit` glue, nothing more.

## Why / trust posture (adapted — this plugin *must* spawn a subprocess)
lazygit is the opposite of auto-title: its whole purpose is to launch an external program, so "no subprocess" can't
apply. Instead:
- **Use *your own* lazygit** (on `PATH`) with **your own lazygit config, untouched**. Bundle/download **nothing**.
- **Pure bash. No python3.** Readable end to end.
- **No network. AI-commit removed** — the reference ships your staged diffs to an LLM; that's the exfiltration we
  refuse. (A separate opt-in add-on could add it later — the "pick what you use" thesis.)

**Capability declaration** (README + manifest comment):
> Spawns: `lazygit` (your binary, in the target dir) · `git` (`rev-parse` for worktree identity) · `herdr` CLI (open/query panes). Files: reads `$HERDR_PLUGIN_CONFIG_DIR/.env`; **writes no lazygit config**. `network: none` · no bundled/downloaded binaries.

## Behavior (v1)
- **One action — `herdr-mihi.lazygit.open`** — toggles a **popup** pane running lazygit, scoped to the current worktree.
  - **Placement: `popup`** (floating over your work; dismiss to return). Size configurable (default ~80% × 80%) via the CLI's `--width`/`--height`. Want it in a tab? Promote the pane in herdr — no separate action needed.
  - **Idempotent toggle:** no lazygit popup for this worktree → open; open but unfocused → focus; focused → close.
  - **Worktree-scoped reuse:** match `git rev-parse --show-toplevel` of the target cwd against a candidate pane's dir; a different repo / a different worktree of the same repo / any resolution failure → **open a fresh popup** (never reuse the wrong one). Bare/non-git dirs get a fallback identity.
  - **cwd resolution** from `HERDR_PLUGIN_CONTEXT_JSON`: `focused_pane_cwd` → `workspace_cwd` → `$HOME`.
  - **`flock` race guard** so key auto-repeat can't open duplicate popups.
  - **Readable error on failure:** if lazygit can't start, the pane **stays open showing the error** (`trap … EXIT` → "press enter to close"), never a flash-and-vanish.
- **lazygit binary:** the user's own, found on `PATH` (or `LAZYGIT_BIN` override). **Absent → fail the action with a readable error + install hint** (`brew install lazygit`). No bundled download, no pinned runtime.

## Herdr surfaces
- `[[actions]]` `open` → `herdr/run.sh open` (actions run with cwd = plugin dir; wrapper still resolves via `$HERDR_PLUGIN_ROOT`).
- `[[panes]]` `lazygit` → command `bash -c 'exec bash "$HERDR_PLUGIN_ROOT/herdr/run.sh" pane'` (**absolute** path — herdr spawns the pane with cwd = target dir, not plugin root; documented gotcha).
- The action opens the pane imperatively: `herdr plugin pane open --plugin herdr-mihi.lazygit --entrypoint lazygit --placement popup --width <W> --height <H> --cwd <target> --focus`.
- **No** `[[events]]`, `[[startup]]`, `[[link_handlers]]`. Dropped the reference's `open-tab` action.

## Config (herdr-native, per 006) & state
- **Config:** `$HERDR_PLUGIN_CONFIG_DIR/.env` (+ committed `.env.example`). Keys: `POPUP_WIDTH=80%`, `POPUP_HEIGHT=80%`, `LAZYGIT_BIN=` (empty → `PATH`). Deliberately tiny.
- **State:** **none.** Popup identity is discovered live (pane query + worktree match); nothing persisted. (Even leaner than auto-title.)

## Language & build (per 007)
- **bash** — thin glue; **no `[[build]]` step** (nothing compiles). Cheapest and most auditable — the trust-friendly choice here.
- Runtime deps: `git`, the user's `lazygit`, the `herdr` CLI, `flock`. No python, no bundled binaries, no `curl`/`tar`.

## Keybinding
- herdr has **no manifest keys surface** — keys live in the user's `config.toml`. Document one copy-paste block (defaults `prefix+g` → `herdr-mihi.lazygit.open`) as `type = "plugin_action"`, then `herdr server reload-config`.
- **herdr-remote caveat:** in default `--remote` mode herdr omits `plugin_action` bindings, so `prefix+g` silently no-ops — document `--remote-keybindings server`.

## Improvements over `Crokily/herdr-lazygit`
~3,500 lines of bash+python3 + two bundled binaries → a **lean pure-bash plugin**: no python, no downloads, your own lazygit + your own config untouched, **AI-commit (diff exfiltration) removed**. Keeps the proven core (worktree-scoped toggle, `flock`, readable-error-on-failure). A plugin you can read in one sitting.

## Deferred / opt-in later
AI-commit (as a separate opt-in add-on) · fzf settings pane · config hot-reload · the "expand" layout gimmick · an `open-tab` action.

## Acceptance criteria
- `prefix+g` toggles a lazygit **popup** scoped to the current worktree; a second press closes it; a different worktree opens its own.
- Runs the `PATH` lazygit with the user's **own config, untouched**; clear error + hint if lazygit is missing.
- **No python, no bundled/downloaded binaries, no network**; capability declaration accurate.
- `flock` prevents duplicate popups on key-repeat; a lazygit startup error stays visible in the pane.
- Pure bash; `herdr plugin link` works with no build step.
