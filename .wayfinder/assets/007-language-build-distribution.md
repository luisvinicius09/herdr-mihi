# Spec: language & build/distribution conventions (ticket 007)

**Status:** Decided (2026-09-08). Grounded in herdr's own docs + a stars-ranked survey of the ecosystem.

## Decisions

**Language heuristic**
- **bash** — thin glue: launching a TUI, opening panes, simple event reactions. No build step; cheapest to ship.
- **Rust — the DEFAULT compiled language** — for anything stateful, long-running, or parse-heavy (daemons/pollers,
  the picker, non-trivial logic). Matches herdr's own core (Rust) so herdr-adjacent crates are reusable, and it
  dominates the top of the plugin ecosystem (reviewr, file-viewer, nvim).
- **Go** — allowed **per-plugin when specifically justified** (a Go-only library, faster iteration on a simple tool).
  Not the house default.

**Build/distribution → prebuilt per-OS binaries** (no build-on-install as the primary path).

## Why (evidence)
- **herdr docs:** build commands run at install, **a build failure aborts the install, and herdr will not install
  missing toolchains** → build-on-install is fragile.
- **Ecosystem survey:** every popular compiled plugin ships prebuilt binaries. The house pattern is
  `[[build]] = ["bash","herdr/install.sh"]` → download the matching binary → **source fallback** (herdr-reviewr 629★,
  herdr-nvim); herdr-plus uses goreleaser. None compile on the user's machine as the primary path.

## The convention

### Manifest
- Compiled plugins: `[[build]] command = ["bash","herdr/install.sh"]`.
- bash plugins: **no** `[[build]]` — the split ref is the scripts themselves.

### `/shared/install.sh` (canonical, synced into each compiled plugin's `herdr/`)
1. Read plugin id + version from the manifest.
2. Map `uname -s`/`-m` → `<os>` ∈ {darwin, linux}, `<arch>` ∈ {arm64, amd64}.
3. Download `…/releases/download/<plugin>-v<ver>/<plugin>-<os>-<arch>` (+ `.sha256`), verify checksum, place in
   `bin/`, `chmod +x`.
4. **Fallback:** if download fails and a toolchain is present, build from source (`cargo build --release`, or
   `go build` for Go plugins); else exit non-zero with a clear message naming the missing tool.
- **Uniform `<plugin>-<os>-<arch>` asset naming** so one downloader serves both Rust and Go (we normalize cargo target
  triples / goreleaser names to this in CI).

### `release.yml` (per-plugin, on version tag)
- Trigger: a plugin's conventional-commit version bump (release-please, multi-package) → tag.
- **Matrix (macOS + Linux only; Windows out of scope):** darwin arm64, darwin amd64, linux amd64, linux arm64.
  - Rust: `cargo build --release --target <triple>` (prefer `*-unknown-linux-musl` for portable static linux binaries).
  - Go: `GOOS`/`GOARCH` loop from a single runner.
- Rename outputs to uniform `<plugin>-<os>-<arch>`, emit `.sha256`.
- `git subtree split --prefix=plugins/<plugin>` → tag `<plugin>-v<ver>` (root = plugin) → create the GH release on
  that tag, attach binaries + checksums → move `<plugin>-latest`.
- Rust: `[profile.release] lto = true, strip = true`. `bin/` and `target/` gitignored (never committed).

## Discoverability note (from herdr docs)
- herdr's marketplace auto-indexes public repos **tagged `herdr-plugin`**. Our monorepo is **one** repo → one
  marketplace entry; per-plugin discovery is handled by our **catalog + picker** (tickets 006/010). Tag the repo
  `herdr-plugin` for baseline discoverability.

## Handoffs
- Tickets 008/009/010 each name their language per this heuristic and inherit this build/dist convention.
  Projected (the briefs decide): **auto-title = Rust** (poller/daemon), **lazygit = bash** (thin wrapper),
  **picker = Rust** (TUI pane + shelling out to `herdr`).
