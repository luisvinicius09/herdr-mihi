# Brief: auto-title plugin (`herdr-mihi.auto-title`)

**Status:** Ready to build (2026-09-08). Grounds: ticket 002 (reference research), 006 (conventions), 007 (language/build).
**Driver:** *trust* — a tab-title plugin you can fully audit and provably trust, not a feature race. Also the **pilot for the phase-2 dashboard's engine** (poll + socket client + sanitization + resolver).

## Why this exists
auto-title is the most crowded category in the ecosystem and the reference (`kryptamine/herdr-auto-title`) is genuinely
excellent. We are **not** building ours to out-feature it — we're building one that is **self-authored, minimal, and
provably safe**, so it can live in a collection you trust end-to-end. Fresh implementation: mine the architecture,
write our own code (the reference is MIT/Go; ours is Rust under the collection's license).

## Security spine — the point of the plugin (every item is an acceptance criterion)
1. **No network, ever.** No networking crate in the tree; `cargo-deny` bans them; no outbound syscalls. Documented `network: none`. No telemetry.
2. **No subprocess / no shell.** The plugin spawns nothing. All terminal/agent-derived text is treated as hostile and never reaches a shell. Git is read from `.git` files, never by running `git`.
3. **Minimal, pinned, audited deps.** As few crates as possible, vendored, pinned; `cargo-audit` + `cargo-deny` clean in CI; reproducible-build target. Every dependency justified in the capability declaration.
4. **Trustable binary.** *Source-first for you* — `herdr plugin link` (dev) / `cargo build` (you trust your toolchain). Prebuilt binaries are the convenience path *for others*, built in CI from tagged source with `SHA256SUMS` + build provenance/attestation, aiming reproducible.
5. **Reliable.** Deterministic (no LLM, no network), self-supervising `[[startup]]` daemon (survives herdr outages, exits cleanly on supersession, no poll failure fatal), atomic state writes, manual-rename protection.

**Capability declaration** (in README + a manifest comment) — the concrete trust artifact, verifiable in 30 seconds:
> Socket methods: `session.snapshot`, `pane.process_info`, `tab.rename`. Files: `$HERDR_PLUGIN_CONFIG_DIR/.env` (read), `$HERDR_PLUGIN_STATE_DIR/manual-names.json` (read/write). `network: none` · `subprocess: none` · `CLI: unused`.

## Behavior (v1 = the reference's core, minimal)
- **Keeps each tab's title in sync with the work happening in it.**
- **Architecture:** one long-lived Rust binary launched via `[[startup]]`; polls `session.snapshot` over `HERDR_SOCKET_PATH` every `POLL_MS` (default 500 ms); renames only tabs whose computed label drifted, via `tab.rename`. **No `[[events]]`** — the reference measured herdr's event stream lagging 10–13 s; polling "describes the present" in one request. Reads a single pane's `pane.process_info` on demand (cached, revision-gated).
- **Title = deterministic confidence ladder** (reference-style, simplified): agent title → terminal title → Claude Code transcript → foreground process → ssh host → git branch → cwd → `Shell`. Fields `{context, branch, activity}`; a higher source wins a field, a lower one may fill an empty field.
- **Grammar:** reference-style but simplified — default `<context> › <branch> › <activity>`, with position prefix and agent-name toggnable. **Fully configurable.**
- **Multi-pane tab → context pane:** focused → a `working`/`blocked` agent pane → most-recently-changed (deterministic tiebreak).
- **Manual-rename protection:** if the user renames a tab to something the resolver wouldn't produce, lock it; persist locks atomically (temp + `rename`), label-guarded; release when the tab no longer carries that label. Never fight the user.
- **Sanitization:** strip ANSI/control/bidi/zero-width chars, collapse whitespace, truncate by terminal **column width** via grapheme clusters; reject "meaningless" values (bare paths, program/shell names, prompts).
- **Git branch** read from `.git` (`HEAD`/refs), following worktrees/submodules via `gitdir:`; long branches → issue key; detached HEAD → short hash.

## Herdr surfaces
- `[[startup]]` → `herdr/run.sh` → the binary (one-shot launch; self-supervising).
- `[[build]]` → `["bash","herdr/install.sh"]` (prebuilt download + source fallback, per 007).
- **Entire herdr surface touched:** three socket methods (above). No CLI, no events, no panes, no actions, no link handlers.

## Config (herdr-native, per 006) & state
- **Config:** `$HERDR_PLUGIN_CONFIG_DIR/.env` (+ committed `.env.example`), plugin-root `./.env` dev fallback. Read once at startup (no hot-reload in v1). Keys: `POLL_MS=500`, `MAX_LENGTH=50` (columns), `BRANCH_MAX=12` (0=off), `POSITION=true`, `TRANSCRIPT=true`, `AGENT_NAME=true`, `DEBUG=false`.
- **State:** manual-rename locks in `$HERDR_PLUGIN_STATE_DIR` (atomic, label-guarded). All else rebuilt from each snapshot.

## Language & build (per 007)
- **Rust.** Memory safety + first-class supply-chain tooling (`cargo-audit`/`deny`/`vet`) directly serve guarantees 2–3; matches herdr's core.
- Deps minimal: grapheme-width (column truncation), a minimal/audited or hand-rolled dotenv + newline-JSON codec. Justify each in the capability declaration.

## Deferred to later versions (kept out of v1 to preserve auditability)
Multi-agent transcripts (beyond Claude Code) · config hot-reload / live-reset channel · friendly `command → name` map · bespoke title grammar.

## Acceptance criteria
- Titles track work at least as well as the reference on a Claude Code + shell + nvim + ssh session.
- CI **proves the spine**: no networking crate/syscall; no subprocess; `cargo-audit`/`deny` clean; deterministic unit tests for the ladder, sanitization, and manual-rename logic.
- Capability declaration present and accurate.
- Survives `herdr server stop`/restart cleanly; never overwrites a manual rename.
- Installs via `herdr plugin link` and `cargo build`; release produces checksummed binaries from tagged source.
