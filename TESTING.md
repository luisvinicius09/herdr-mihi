# Testing & developing herdr-mihi plugins

Plugins run *inside* herdr (socket, panes, events), so "does it work" needs more than unit tests.
Four tiers, cheapest first — most bugs die in tiers 1–3 without a live herdr. (Design rationale and
source citations live in `.wayfinder/assets/011-testing-and-dev-loop.md`.)

## Tier 0 — the dev loop (manual, fast)

`herdr plugin link` is offline and runs **no build**, so build first:

```sh
just sync-shared
# compiled plugins: build the binary the wrapper will exec
( cd plugins/<name> && cargo build --release && mkdir -p bin && cp target/release/<name> bin/ )
herdr plugin link plugins/<name>

# exercise it
herdr plugin action invoke herdr-mihi.<name>.open
herdr plugin pane open --plugin herdr-mihi.<name> --entrypoint <name> --placement popup --focus
herdr plugin log list --plugin herdr-mihi.<name>     # stdout / stderr / exit of every run
herdr plugin config-dir herdr-mihi.<name>            # where to drop .env
```

- **Startup plugins** (auto-title): `[[startup]]` runs when the server starts the plugin — trigger with
  `herdr server stop` + restart. Config is read once at startup.
- **Keybindings:** add `[[keys.command]]` to your herdr `config.toml`, then `herdr server reload-config`.
  Over `--remote`, use `--remote-keybindings server` or bindings silently no-op.

## Tier 1 — unit tests (no herdr)

Pure logic, hermetic. Rust: `cargo test`, redirecting `$HERDR_PLUGIN_CONFIG_DIR`/`$HERDR_PLUGIN_STATE_DIR`
in-test. Feed **golden herdr-JSON fixtures** (canned `session.snapshot` / pane / context JSON) under
`plugins/<name>/tests/fixtures/`. bash: `tests/<area>-test.sh` scripts asserting decision logic against
faked inputs.

## Tier 2 — mock-herdr tests (no live herdr)

Assert what the plugin *sends*. Socket plugins: a stub socket server (newline-delimited JSON, one request
per connection) that returns canned snapshots and records `tab.rename`. CLI plugins: a fake `herdr` on
`$PATH` that logs its argv and returns canned JSON — assert the picker builds the exact
`plugin install … --ref <plugin>-latest` commands, only names the pinned owner, and runs nothing before confirm.

## Tier 3 — integration tests (real headless herdr)

herdr has a headless mode and is installable in CI. In a temp `HOME`/runtime dir: spawn a headless server,
`herdr plugin link` the plugin, create a workspace, invoke actions/panes, assert on `herdr … --json` output
and `herdr plugin log list`. Runs as a separate, slower CI job.

## Tier 4 — trust assertions ARE tests (CI-enforced)

The capability declarations become gates: `cargo-deny` bans networking crates, `cargo-audit` for advisories,
a grep asserts no `std::process::Command` / no `curl`/`wget` / no non-pinned owner where a plugin promises so,
and `just check-sync` fails on drifted shared shims.

## CI shape

- `ci.yml` (push/PR): `just ci` — tiers 1, 2, 4 (fast, no herdr).
- `integration.yml`: install herdr → tier 3.
- Each plugin README carries a short **manual checklist** (tier 0) for eyeballing it live.
