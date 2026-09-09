# Research: how herdr-lazygit works, and where it falls short

**Ticket:** 003 · **Type:** research · **Source:** [`Crokily/herdr-lazygit`](https://github.com/Crokily/herdr-lazygit) @ `main` (v0.3.0, MIT).
Read files: `herdr-plugin.toml`, `README.md`, `DESIGN.md`, `scripts/{open-lazygit,open-lazygit-tab,run-lazygit,runtime-env,install-runtime}.sh`, `CHANGELOG.md`, `lazygit-config.yml`.

## Scope of "done better"

We are **not** rebuilding lazygit — lazygit stays the stock external binary and owns all git UI (staging,
diff, history, commit, sync). Our plugin is the thin **herdr↔lazygit glue**: the manifest, the
action/pane, worktree-scoped launch/toggle, keybinding registration, config plumbing, and missing-binary
handling. "Done better" = *that integration layer* is cleaner and lighter than `Crokily/herdr-lazygit`'s.

## Bottom line

This is **not a rough plugin to trivially beat** — it's a ~3,500-line, two-language (bash + python3),
deeply-considered reference with a written "constitution" (`DESIGN.md`), a free-key occupancy analysis,
worktree-scoped idempotent toggling, layered config, hot-reload, AI commit generation, an fzf settings
pane, and a SHA-pinned private runtime. **"Done better" is therefore mostly a _scope & simplicity_
argument, not a bug-fix argument**: our version should keep its best ideas (worktree-scoped toggle,
readable-error handling, user-config inheritance) while shedding the weight that makes it heavy to
install and maintain (python3 dependency, two bundled binaries, ~5,000 total lines) — and fixing the one
genuine UX wart it can't fix itself: **the keybinding is not auto-registered; the user hand-edits
`config.toml`.**

## How it launches lazygit

- **Surfaces used:** `[[panes]]` + `[[actions]]` (no `[[events]]`/`[[startup]]` — appears **only** on an
  explicit action). One pane `id = "lazygit"`, `title = "Git"`, `placement = "split"`. Two actions:
  `open` (split) and `open-tab` (own tab).
- **Pane type = split** (a 42-column sidebar), or its **own tab** via `open-tab`. Not popup/overlay/zoomed.
  "Expand" (`U`) does **not** change the herdr pane type — it toggles *lazygit's internal* layout
  (`sidePanelWidth` 0.99 ↔ 0.3333, width 42 ↔ 110 cols) by rewriting a per-pane config layer + a socket
  `set-width` + a hot-reload nudge.
- **Two-step launch.** The action shells `bash scripts/open-lazygit.sh` (a **bare relative path works for
  actions** because herdr runs actions with cwd = plugin dir). That launcher then opens the pane
  *imperatively* via the herdr CLI:
  `herdr plugin pane open --plugin herdr-lazygit --entrypoint lazygit --placement split --direction right --cwd <target> --focus`,
  then narrows the split to `SIDEBAR_COLS` via `layout-helper.py set-width` over the herdr socket.
- **The pane command must use an absolute path.** The manifest's `[[panes]]` command is
  `bash -c 'exec bash "$HERDR_PLUGIN_ROOT/scripts/run-lazygit.sh"'` — a documented gotcha: herdr spawns
  the *pane* with cwd = the launcher's `--cwd` (the user's work dir), **not** the plugin root, so a bare
  `scripts/...` path dies instantly. herdr injects `HERDR_PLUGIN_ROOT` for this.
- **Scoping is worktree-aware and genuinely good.** cwd resolves from `HERDR_PLUGIN_CONTEXT_JSON`
  (`focused_pane_cwd` → `workspace_cwd` → `$HOME`). The launcher is an **idempotent toggle**:
  - `open` — scoped to the **current tab**: no pane → open split; exists but unfocused → focus; focused → close.
  - `open-tab` — scoped to the **current workspace**: none → new tab; in another tab → switch to it; in
    current tab → focus/close. Never crosses workspaces.
  - **Reuse requires git-worktree identity match**: `git rev-parse --show-toplevel` on both the target cwd
    and each candidate pane's `foreground_cwd`/`cwd`. Different repo, a *different worktree of the same
    repo*, or any resolution failure → degrade to **OPEN** a fresh pane (never reuse the wrong one). Bare
    repos and non-git dirs get fallback identities.
  - Panes are identified by label `"Git"` **+** `herdr pane process-info` confirming lazygit actually runs
    there (retried 4×0.3s for a just-spawned pane) **+** a `SAFE` regex option-injection guard on ids.
  - A `flock` guards against fire-and-forget `action invoke` races (key auto-repeat opening duplicates).

## Manifest / language / binary / config / keybinding

- **Manifest:** TOML, `min_herdr_version = "0.7.0"`, `platforms = ["linux", "macos"]`. One `[[build]]`,
  one `[[panes]]`, two `[[actions]]`. **No keybinding surface in the manifest.**
- **Language:** **bash + python3 (≥ 3.7, a hard requirement)**. ~3,500 lines of scripts + ~1,250 lines of
  tests across 16 scripts. python3 does JSON parsing, socket pane-geometry (`layout-helper.py`), locking,
  and the free-key analysis (`free-keys.py`).
- **Binary location — pinned private runtime.** The `[[build]]` step (`install-runtime.sh`) downloads the
  **official lazygit 0.63.0 + fzf 0.74.0** archives for the OS/arch, verifies **repo-pinned SHA-256**
  digests, and installs both under the managed checkout's `bin/`. Never uses `PATH`, Homebrew, or `sudo`;
  atomic publish with rollback. Escape hatch: `RUNTIME_LAZYGIT_BIN` / `RUNTIME_FZF_BIN` (absolute paths in
  `panel.conf`, version-warned + "unsupported"). Mirror overrides
  (`HERDR_LAZYGIT_*_BASE_URL`) for firewalled installs. **`herdr plugin link` does not run `[[build]]`**,
  so linked dev checkouts must run the installer by hand.
- **Config — layered `LG_CONFIG_FILE` (comma list, later wins), up to 5 layers:**
  0. the user's *own* lazygit config (from `lazygit --print-config-dir`; opt out with
     `INHERIT_USER_CONFIG=0`) → 1. bundled `lazygit-config.yml` (gui only: `mouseEvents`, `showRandomTip`)
     → 2. `generated.yml` (plugin keys/customCommands, built from `keys.conf`) → 3. `layout-<pid>-<epoch>.yml`
     (per-pane sidebar/expanded state) → 4. `lazygit-user.yml` (user override, always wins).
  **A missing file is a fatal lazygit error**, so `run-lazygit.sh` creates each layer before `exec`.
  Plugin state (`$HERDR_PLUGIN_CONFIG_DIR`, fallback `~/.config/herdr-lazygit`): `keys.conf`, `panel.conf`,
  `ai-backend.conf`, `prompt.txt`.
- **Keybinding — NOT auto-installed.** The user must hand-add
  `prefix+g` → `herdr-lazygit.open` and `prefix+shift+g` → `herdr-lazygit.open-tab` to their herdr
  `config.toml` (`type = "plugin_action"`) and run `herdr server reload-config`. The README even ships a
  copy-paste **AI-agent prompt** to do this idempotently — a tell that this step is genuinely awkward.
  Inside lazygit there are exactly **three plugin verbs** — `C` (AI commit), `U` (expand), `;` (settings) —
  chosen by `free-keys.py` occupancy analysis so they never shadow a commonly-used lazygit built-in; all
  remappable from the settings pane.
- **Extra scope (well beyond "a git pane"):** AI commit-message generation (claude/codex/opencode/gemini
  or a custom CLI, with staged-diff budget sampling), an fzf settings pane, and a
  filesystem-is-the-bus **hot-reload-on-focus** config model.

## What it already does well (don't regress these)

- **Worktree-scoped idempotent toggle** (open/focus/close, refuses to reuse the wrong worktree's pane).
- **Failure is visible:** a missing/broken runtime fails the *action* with a readable error in
  `herdr plugin log list`; a lazygit startup error (e.g. an inherited config key the pinned version rejects)
  **leaves the pane open showing the error + the config layers**, instead of closing instantly.
- **`flock`** against action-invoke races.
- **Layered config** with user-override-always-wins **and** inheriting the user's own lazygit theme/config.
- **Runtime integrity:** SHA-pinned, no-sudo, no-brew, atomic-with-rollback download.

## Limitations / rough edges — where "done better" lives

1. **Keybinding isn't auto-registered** (the biggest wart). Install wires no key; the user hand-edits
   `config.toml` (or runs an AI prompt to). *Open question this raises for our conventions: **can a herdr
   manifest declare a default keybinding**, or must every plugin document a manual edit?* → lead for **ticket 006**.
2. **Heavy install footprint & dependencies** for what is core-wise "open lazygit in a pane":
   **python3 ≥ 3.7** is mandatory, and every install **downloads two binaries** (lazygit + fzf, tens of MB)
   needing `curl`/`wget` + `tar` + `sha256sum`. `fzf` exists **only** for the settings pane. → lead for **ticket 007** (build/distribution) and the footprint call in **ticket 005**.
3. **Runs pinned lazygit 0.63.0, not the user's.** Great for reproducibility, but the user's own (newer,
   extended) lazygit isn't what runs, and overriding it is "unsupported." Key/config generation is welded
   to one exact version.
4. **Python-per-launch latency & dependency.** Each `open` shells out to python3 several times (context
   parse, decision engine, socket geometry, lock). A compiled (rust/go) or leaner-bash launcher removes both.
5. **Indirect pane identification.** Reuse detection polls `herdr pane process-info` per candidate
   (up to 4×0.3s) and string-matches `"lazygit"` in foreground-process names — works, but latency-prone and
   fragile. *Open question: can a plugin tag/query its **own** panes by a stable id?* → lead for **ticket 006**.
6. **Herdr Remote foot-gun (inherited from herdr).** In default `--remote` mode herdr omits
   `[[keys.command]]` `plugin_action` bindings, so `prefix+g` **silently does nothing**; the user must
   attach with `--remote-keybindings server`. Worth a documented note in our brief.
7. **Complexity/maintenance surface:** ~3,500 script + ~1,250 test lines, two languages, for a git pane.
   Most of the weight is AI-commit + settings + reproducible-runtime machinery, not the launch/scope core.
8. **Context resolves from the UI-focused pane**, not a background process — so the actions must be
   triggered by a foreground keybinding. A design constraint to carry, not a defect.

## Improvements to make (feeds ticket 009 — the "done better" lazygit brief)

- **Auto-register the keybinding if herdr allows it.** First resolve the ticket-006 question "can a manifest
  declare a default keybinding?" If yes, ship it; if no, make the manual step a single documented one-liner
  (not an AI prompt) and state the default keys prominently.
- **Ship a lean core, make the heavy parts opt-in.** A minimal plugin that nails *launch + worktree-scoped
  toggle + expand* with **no python3 and no bundled binaries**. Fold AI-commit / settings-pane / fzf into an
  optional add-on — which is exactly the collection's "plugin of plugins, pick what you use" thesis.
- **Prefer the user's own lazygit by default** (detect on `PATH` / check `lazygit --version`), falling back
  to a pinned download only when absent — inverting herdr-lazygit's "always private, override unsupported."
  Reconsider bundling **fzf at all** if the settings pane is dropped from core.
- **Cut the python dependency** — do JSON/geometry either in the chosen compiled launcher (ticket 007) or
  with the herdr CLI's own output; don't require a python3 interpreter for a git pane.
- **Keep the proven ideas verbatim:** worktree-identity pane reuse, the open/focus/close toggle, the
  `flock` race guard, readable-error-on-startup-failure, user-config inheritance, and the "never shadow a
  common lazygit built-in" key rule.
- **Decide our default pane type deliberately** (split sidebar vs popup/overlay vs tab) rather than
  inheriting the 42-col sidebar by default — a ticket-009 UX call to make against how *we* want git to feel.
- **Document the herdr-remote keybinding caveat** in the plugin README so `prefix+g` "doesn't work over
  remote" never becomes a mystery.
