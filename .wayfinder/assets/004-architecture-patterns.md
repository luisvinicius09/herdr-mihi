# Research: architecture patterns from herdr-nvim & the official cookbook

**Ticket:** 004 · **Type:** research
**Sources (shallow clone, primary code read directly):**
- [ChmaraX/herdr-nvim](https://github.com/ChmaraX/herdr-nvim) @ `v1.0.0` — one polished single-plugin repo (Rust core + Lua nvim plugin).
- [ogulcancelik/herdr-plugin-examples](https://github.com/ogulcancelik/herdr-plugin-examples) @ herdr 0.7 — the official cookbook: **four** standalone example plugins (the notes listed three; there is also `rust-release-check`), one per language (JS, Bash, Lua, Rust).

Feeds ticket 006 (structure & conventions) and ticket 007 (language & build/distribution).

## Bottom line

The two repos show **two ends of the maturity spectrum** and the gap between them *is* our convention set:

- **The cookbook** = minimal, cwd-relative, interpreter-on-PATH, copy-don't-share. Good for a 40-line demo; not enough backbone for a maintained collection.
- **herdr-nvim** = the production template: a **PATH-normalizing wrapper script** in front of a **single multi-command binary**, **prebuilt per-OS binaries** downloaded at install (toolchain-free), release-please + a build matrix, and a `just ci` task runner. This is the shape ticket 006/007 should standardize on.

The two **disagree on config/state**, which is the one convention we must consciously pick (below).

---

## 1. Per-plugin layout & manifest conventions

**Cookbook (flat, minimal):** each plugin is a subdir = `herdr-plugin.toml` + entrypoint file(s) at the plugin root. Commands are **cwd-relative** (`node toggle.mjs`, `bash open.sh`, `lua setup.lua`, `./target/release/rust-release-check`) — this works only because herdr sets **cwd = plugin dir**.

**herdr-nvim (structured):** language dirs kept apart — `src/` (Rust), `lua/herdr-nvim/`, `plugin/`, `doc/`, `tests/`, and a dedicated **`herdr/` dir holding the herdr-facing shims** (`run.sh`, `install.sh`). Every manifest `command` is `["sh","-c","exec bash \"$HERDR_PLUGIN_ROOT/herdr/run.sh\" <subcmd>"]` — **absolute via `$HERDR_PLUGIN_ROOT`**, never cwd-relative.

**Manifest schema confirmed across all five plugins** (extends ticket 001's field list):
- Top-level: `id`, `name`, `version`, `min_herdr_version`, `description`, `platforms = ["macos","linux"]`.
- `[[actions]]`: `id`, `title`, `command`, optional `contexts = ["pane","workspace"]`.
- `[[panes]]`: `id`, `title`, `placement` (`popup` | `split`), `command`.
- `[[events]]`: `on` (e.g. `pane.agent_status_changed`), `command`.
- `[[link_handlers]]`: `id`, `title`, `pattern` (regex), `action`.
- `[[build]]`: `command` (array).
- `[[startup]]` exists per the map notes but is **unused** in either repo — no live example to copy.

**`id` naming = `<namespace>.<plugin>`**: herdr-nvim → `chmarax.herdr-nvim`; cookbook → `examples.<name>`. The action/keybind reference is the fully-qualified `<id>.<action-id>` (`chmarax.herdr-nvim.toggle`).

### Adopt
- **`<owner-or-collection>.<plugin>` id scheme**, one namespace prefix for the whole collection (e.g. `herdr-mihi.auto-title`).
- **Separate the herdr-facing shim from the implementation** (`herdr/run.sh` pattern) — the manifest points at one stable wrapper per plugin; implementation can move behind it.
- Always set `platforms`, `min_herdr_version`, `version`, `description` on every manifest.

### Avoid / watch
- **cwd-relative commands + bare interpreter names** (`node`, `lua`, `gh`): herdr runs commands with **no shell and a minimal PATH** (see §4). The cookbook gets away with it; herdr-nvim explicitly does not trust it. Prefer the wrapper.

---

## 2. Config & state handling — THE decision point (feeds 006)

The two repos take **opposite approaches**; we must standardize on one for consistency across the collection.

| | Config | State |
|---|---|---|
| **Cookbook** (`agent-telegram-notify`) — *herdr-native* | `HERDR_PLUGIN_CONFIG_DIR/.env` (dotenv), **falls back to plugin-root `./.env`** for dev. Ships a committed **`.env.example`**. | A file in `HERDR_PLUGIN_STATE_DIR` (e.g. `enabled`), **XDG_STATE_HOME fallback** → `~/.local/state/<repo>/<plugin>/…`. |
| **herdr-nvim** — *own XDG namespace* | `~/.config/herdr-nvim/config.toml` (TOML), override via `HERDR_NVIM_CONFIG`. **Ignores `HERDR_PLUGIN_CONFIG_DIR`.** | `XDG_STATE_HOME/herdr-nvim/<tab>.json`, override via `HERDR_NVIM_STATE_DIR`. **Ignores `HERDR_PLUGIN_STATE_DIR`.** |

**Recommendation for 006: default to the herdr-native pattern** (`HERDR_PLUGIN_CONFIG_DIR` / `HERDR_PLUGIN_STATE_DIR`). It's what herdr injects, works with `herdr plugin config-dir <id>` (the documented way users find where to drop `.env`), and keeps each plugin's data under herdr's managed tree. herdr-nvim's own-namespace choice makes sense *only because* its nvim half also reads `~/.config/herdr-nvim` — a special case we won't usually have.

### Adopt (regardless of which dir)
- **Committed `.env.example`; never commit `.env`** (gitignore it). Setup doc = `cp .env.example "$(herdr plugin config-dir <id>)/.env"`.
- **Config-dir `.env` first, plugin-root `.env` as dev fallback** — lets `herdr plugin link` dev work without touching the managed dir.
- **Atomic state writes**: serialize → temp file → `rename()` (herdr-nvim `state::save`). Never write in place.
- **Env-var overrides for config/state paths** so tests can redirect them (both repos do this; enables the hermetic test guards in `config.rs`/`state.rs`).
- **State keyed by a sanitized id** (herdr-nvim replaces `:` in tab ids → `_` for a safe single path component).
- **`.env` flag parsing convention**: treat `0|false|no|off` (case-insensitive) as false, everything else true (`envFlag`).

---

## 3. Entrypoint & dispatch patterns

- **Single artifact, argv subcommand dispatch.** herdr-nvim ships **one binary** with a `match args[1]` dispatcher (`toggle|sidebar|doctor|pick-file|picker|open-link`) in `main.rs`; each manifest entry passes a different subcommand. `agent-telegram-notify` does the same in JS (`toggle.mjs on|off`). **One entrypoint, many actions** beats one script per action.
- **Wrapper script normalizes the environment** (`herdr/run.sh`): sets a sane `PATH`, resolves `$HERDR_PLUGIN_ROOT/bin/<binary>`, `exec`s it with `"$@"`. The manifest only ever names the wrapper.
- **Programmatic panes**: open a declared `[[panes]]` entrypoint at runtime with `herdr plugin pane open --plugin <id> --entrypoint <pane-id> --placement split --direction right --env KEY=VALUE --focus` (github-link-preview `open.sh`, herdr-nvim toggle). Pass data to the pane via `--env`.
- **Call back into herdr via `HERDR_BIN_PATH`** (fallback literal `herdr`): `herdr pane split/rename/run`, `herdr terminal title set/clear`, etc. Every language example does `${HERDR_BIN_PATH:-herdr}`.

---

## 4. Error handling & logging conventions (strong, consistent — adopt wholesale)

- **Exit 0 = "not my job right now."** Missing config, non-actionable event/status, an unsupported URL, or a missing optional tool (`gh`) all **`exit 0` after a one-line stderr note** — never fail the herdr event/action. Seen in *every* cookbook plugin (`notify.mjs` exits 0 on missing token or uninteresting status; `preview.sh` exits 0 if `gh` absent).
- **Non-zero only on genuine failure.** herdr-nvim's `run()` wrapper: `Ok(()) → 0`, `Err(e) → eprintln!("herdr-nvim: {e:#}") ; 1`. `rust-release-check` uses `exit 1` for "check failed" vs `exit 2` for "couldn't run" — distinct codes for distinct meanings.
- **Diagnostics to stderr, prefixed with the plugin name.** User-facing pane output to stdout.
- **Defensive parsing of herdr's JSON env** (`HERDR_PLUGIN_CONTEXT_JSON` / `HERDR_PLUGIN_EVENT_JSON`): fields are **unstable across herdr versions** — `notify.mjs` tries ~6 fallback field names for agent status; malformed JSON returns `{}` rather than throwing. **Never assume a context field exists.**
- **Config parse failure → defaults + one warning, never panic** (herdr-nvim `config::load`). Malformed TOML must not brick the plugin.
- **Interactive panes trap EXIT** to hold the pane open (`preview.sh` `trap finish EXIT` → "press enter to close").

---

## 5. Build & distribution (feeds 007) — two strategies, one clearly better

**A. Build-on-install (`rust-release-check`):** manifest `[[build]] command = ["cargo","build","--release"]`; action runs `./target/release/<bin>`. **Requires the toolchain (`cargo`) on the user's machine.** Simple to author, poor install UX. Note: `herdr plugin link` (dev) does **not** run build commands — you must `cargo build` yourself first (documented in the cookbook README).

**B. Prebuilt-binary download with source fallback (herdr-nvim `herdr/install.sh`) — the "reviewr pattern":**
```
[[build]] command = ["bash", "herdr/install.sh"]
```
`install.sh`: reads version from the manifest → maps `uname -s/-m` → Rust target triple → `curl` the matching asset from `github.com/<repo>/releases/download/v<ver>/herdr-nvim-<target>` into `bin/`. **Falls back to `cargo build --release` only if the download fails and `cargo` exists**, else errors clearly. **Toolchain-free for the common path.**

Supporting machinery around (B):
- **Release build matrix** (`release.yml`): `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu` → `cargo build --release --target …` → attach `herdr-nvim-<target>` to the GH release (`softprops/action-gh-release`).
- **release-please** (`release-type: rust`) automates version bump + changelog + tag, and **keeps `herdr-plugin.toml` version in sync with `Cargo.toml`** via `extra-files` (`jsonpath: $.version`). This solves the manifest-version-drift problem for free.
- `[profile.release] lto = true, strip = true`; `bin/` and `target/` gitignored (built at install, never committed).

### For 007
- **Compiled plugins → strategy B** (prebuilt download + source fallback). Build-on-install is acceptable only for a plugin that genuinely needs a local build, or as the fallback tier.
- This **reinforces ticket 005/001's footprint finding**: prebuilt binaries live on **GitHub Releases**, so a plugin's install can pull just its own binary regardless of repo topology — the release asset, not the repo tree, carries the weight.
- **bash plugins need no build step at all** — the cheapest distribution. Good default for thin glue (matches the map's language heuristic).

---

## 6. Testing / CI

- **`justfile` as the task runner** (herdr-nvim): `just ci` = `cargo fmt --check` + `cargo test` + headless nvim Lua suite (`nvim --headless -l tests/run.lua`); `just setup` wires the git hooks path. Single entry point CI and humans share.
- **`ci.yml`** on push + PR: toolchain + neovim + just → `just ci`. **`release.yml`** re-runs the full test suite on each target *before* building. Extra guards: **pr-title-lint** + a **commit-msg githook** (conventional commits, required by release-please).
- **Test hermeticity**: env-var-overridable config/state paths + a `Mutex`-guarded env swap (`ConfigEnvGuard`, `StateDirGuard`) so tests don't touch real dirs and don't race. Fixtures under `tests/fixtures/` (sample agent session logs, layout JSON).
- **Cookbook plugins have no tests/CI** — reference demos only.

### For 006
- A **collection-level task runner** (`justfile`) with a `ci` recipe spanning all plugins, plus per-language recipes, fits the "clean, easy-to-follow" goal.
- Adopt **conventional-commits + release-please** if we want automated per-plugin versioning/tagging — especially valuable if 005 lands on per-plugin release refs (release-please supports multi-package/monorepo).

---

## 7. Organizing multiple plugins vs one

- **Cookbook = flat monorepo of standalone plugins**: each subdir fully self-contained with its own manifest; **root README indexes them** and gives a per-subdir install command (`herdr plugin install owner/repo/<subdir>`). Explicit stance: *"Copy what you need into your own plugin rather than depending on this repo"* — **no shared library; duplication over coupling.**
- **herdr-nvim = one repo, one plugin** — the deep, polished single-plugin template.
- **Neither demonstrates shared code across plugins in a monorepo.** This is an open question for **006** and interacts with **001/005**: herdr stores a *full copy per plugin-id*, so any shared code must be **vendored into each plugin's own tree** (or built into each plugin's binary) — a monorepo `shared/` dir that plugins import at dev time would need a bundling/vendoring step before release. The cookbook's "just copy it" is the path of least resistance given herdr's install model.

### README conventions (every plugin, per both repos)
Lead line states **language + one-sentence purpose**; then **Setup** (install command, `.env` copy via `herdr plugin config-dir`), **keybind example** (herdr binds *no* keys by default — always show the `[[keys.command]]` block), **Behavior**, **Requirements** (runtime versions + external tools, e.g. `nvim ≥ 0.10 · herdr ≥ 0.7.4`, `gh`, `Node ≥ 18`, `Lua 5.4`). Badges (CI/release/license) on the flagship.
- **Gap to close in 006:** *neither* repo has an explicit **"Platforms: macOS · Linux"** line — they encode platforms only in the manifest. Our convention (per the map) **requires that line in every README**; it's an addition, not something to copy.

---

## Patterns to adopt / avoid — quick list

**Adopt**
1. Wrapper shim (`herdr/run.sh`) that fixes PATH + resolves `$HERDR_PLUGIN_ROOT/bin/<bin>`; manifest names only the wrapper.
2. One artifact per plugin, argv-subcommand dispatch (not one script per action).
3. `<collection>.<plugin>` id namespace; always set `platforms`/`min_herdr_version`/`version`.
4. Config = `HERDR_PLUGIN_CONFIG_DIR/.env` + committed `.env.example` + plugin-root dev fallback.
5. State = file under `HERDR_PLUGIN_STATE_DIR`, **atomic temp+rename**, env-overridable path, sanitized keys.
6. Error convention: **exit 0 when not applicable**, non-zero only on real failure; distinct codes; stderr-prefixed diagnostics; **defensive** context/event JSON parsing; config errors → defaults, never panic.
7. Compiled dist: **prebuilt per-OS binary download + source fallback** (`install.sh`), GH Releases matrix build, release-please syncing manifest version.
8. `justfile` task runner + push/PR CI + conventional commits; hermetic tests via env-overridable paths.
9. README template with mandatory **"Platforms: macOS · Linux"** line (our addition) + keybind block + Requirements.

**Avoid / decide**
1. cwd-relative commands + bare interpreter names — brittle under herdr's minimal, shell-less PATH.
2. Build-on-install requiring a toolchain as the *primary* path (make it the fallback tier).
3. herdr-nvim's own-`~/.config` namespace as the default — prefer herdr-injected dirs unless a plugin has a non-herdr consumer.
4. No-shared-code "copy it" duplication is the cookbook's stance; whether our collection vendors shared helpers vs duplicates is an **open 006 question**, constrained by herdr's full-copy-per-plugin install (001/005).

## Notes for downstream tickets
- **`rust-release-check`** is a fourth cookbook example (build-on-install demo) not in the map's Notes — worth listing there if the reference set is ever revised.
- **`herdr plugin config-dir <id>`** is the documented, user-facing way to locate a plugin's config dir (recent herdr builds) — the canonical setup instruction for any `.env`-based plugin. Complements ticket 001's on-disk path finding.
- **`[[startup]]`** has no live example in either repo — if a phase-1/2 plugin needs it (e.g. the dashboard), that's uncharted and may want its own quick research.
