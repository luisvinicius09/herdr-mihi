# Brief: picker plugin (`herdr-mihi.picker`)

**Status:** Ready to build (2026-09-08). Grounds: ticket 005 (topology/refs), 006 (conventions/catalog), 007 (Rust/prebuilt), the **trust-first** principle, and the approved prototype [`prototypes/010-picker-mock.sh`](../prototypes/010-picker-mock.sh).
**Driver:** the collection's **trust-visible front door** — the concrete expression of "pick what you use" *and* "see exactly what runs." It's the one plugin that installs code, so it's the whole thesis in miniature.

## Behavior (v1) — from the approved prototype
- **Action `herdr-mihi.picker.open`** opens a **popup** pane (roomy; size configurable; promotable to a tab like lazygit).
- **Browse:** list the catalog — each row: selection checkbox · status (`● installed vX` / `○ available` / `⤴ update vX→vY`) · name · description. Navigate ↑/↓ or j/k; `space` select; `a` all.
- **Per-row action** by state: not-installed → **install**; installed with a newer catalog version → **update**; installed → **uninstall** (a distinct mark, e.g. `x`). Multi-select across all three.
- **Confirm (the trust centerpiece)** — shows the EXACT commands that will run, one per selection:
  - install / update → `herdr plugin install luisvinicius09/herdr-mihi --ref <plugin>-latest`  → commit `<sha7>`
  - uninstall → `herdr plugin uninstall herdr-mihi.<plugin>`
  - footer: *source pinned to `luisvinicius09/herdr-mihi` · nothing runs until you say yes · network only to the pinned repo.* `[y] yes / [n] back / [q] cancel`.
- **Run:** execute each via the herdr CLI, streaming progress → **Done** summary (installed / updated / removed / failed).
- **Resolved commit SHA:** each `-latest` displays the commit it currently points to, so you approve an exact commit, not a moving pointer.

## Catalog (per 006) + freshness
- Reads `catalog.json` (generated from manifests at release; per plugin: id, name, description, language, platforms, `latest` ref, latest version, **latest commit SHA**).
- **Freshness:** fetch `catalog.json` from the **pinned repo** on open (the only picker-initiated network, and only to the collection repo), with the bundled copy as an **offline fallback**. Installed state + versions come from `herdr plugin list`.

## Trust posture / capability declaration
> Installs/uninstalls **only** plugins in the pinned collection, via refs from the catalog — **never** an arbitrary owner/repo/URL. Shows the exact commands + resolved SHA; **nothing runs before explicit confirm**. Spawns: `herdr` CLI (`plugin list` / `plugin install --ref` / `plugin uninstall`). Network: **only** `luisvinicius09/herdr-mihi` (catalog fetch + herdr's own clone). Files: reads catalog (cached) + `$HERDR_PLUGIN_CONFIG_DIR/.env`. No arbitrary URLs, no telemetry.

## Herdr surfaces
- `[[actions]]` `open` (opens the pane). `[[panes]]` `picker` (`placement = popup`, roomy default, configurable). `[[build]]` = `bash herdr/install.sh` (prebuilt + source fallback). **No** events/startup/link_handlers.
- **Bootstrapping:** install the picker first — `herdr plugin install luisvinicius09/herdr-mihi --ref picker-latest` — then use it for everything else.

## Config (herdr-native, per 006) & state
- **Config:** `$HERDR_PLUGIN_CONFIG_DIR/.env` — `POPUP_WIDTH=90%`, `POPUP_HEIGHT=85%`, `CATALOG_REFRESH=on-open|manual`, `OWNER=luisvinicius09/herdr-mihi` (the pin).
- **State:** none required — installed state is queried live; catalog is cached.

## Language & build (per 007)
- **Rust** — interactive TUI (ratatui-class) + non-trivial logic → the default compiled language. Prebuilt binaries + `install.sh` (source fallback) per 007. Minimal, audited deps (TUI + JSON), each justified in the capability declaration.

## Keybinding
- Documented (no manifest keys surface): default `prefix+p` → `herdr-mihi.picker.open`. Note the **herdr-remote caveat** (`plugin_action` bindings omitted in default `--remote`).

## Role / references
No direct reference plugin; mines `herdr-plus` (install.sh, fuzzy list) and `herdr-navigator` (fuzzy navigation). This is the collection's front door and the clearest demonstration of the trust thesis.

## Deferred
Fuzzy search/filter (if the catalog grows) · a plugin detail view · dependency/compat checks · an AI-anything.

## Acceptance criteria
- Opens as a popup; lists the catalog with correct installed / available / update states (from `herdr plugin list` + catalog).
- Multi-select across install/update/uninstall; the confirm screen shows the **exact commands + resolved SHA** + the pinned-source / no-arbitrary-URL / network-only-to-pinned-repo guarantees **before** anything runs.
- Runs only the shown commands; refuses any non-pinned source.
- Bootstraps via `--ref picker-latest`; Rust; prebuilt + source fallback; capability declaration accurate.
