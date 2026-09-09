# Spec: testing & dev-loop against herdr (ticket 011)

**Status:** Decided (2026-09-09). The collection's `TESTING.md` convention. Grounds: herdr source
(`tests/cli/*`, `src/server/headless.rs`, `src/api/schema/*`, `tests/fixtures/*`), the reference
plugins' own test suites, and the trust-first principle.

## Bottom line (two findings that shape everything)

1. **herdr's docs say "no test tooling," but the source says otherwise.** herdr ships a **headless server
   mode** (`src/server/headless.rs`, `tests/server_headless.rs`) and its own tests **spawn a real server
   and drive it over the socket via the CLI** (`tests/cli/plugins.rs`: `spawn_herdr` / `spawn_named_server`
   / `wait_for_socket` / `run_cli_json_in_dir`). herdr is also installable in CI (`curl … install.sh`). So
   **integration tests can run against a real headless herdr — in CI, not just locally.**
2. **The mock-socket approach is proven.** `herdr-auto-title` ships **`internal/herdr/herdrtest/stub.go`** —
   a fake herdr socket server — to test its client + poll loop with no live herdr. And herdr ships **JSON
   fixtures** for the protocol shapes (`tests/fixtures/endpoint-snapshot-v1.json`,
   `session/current-herdr-session.json`, `endpoint-method-shapes-v1.json`) we can reuse as golden inputs.

So we get four tiers, cheapest-first; most bugs die in tiers 1–3 without a live herdr.

## Tier 0 — the dev loop (manual, fast)

`herdr plugin link` is **offline and runs no build** (verified: `plugin_link_works_offline_and_is_global`),
so build/sync first:

```
just sync-shared && just build <name>      # link runs no [[build]] — do it yourself
herdr plugin link plugins/<name>           # register the working tree (global, disabled or enabled)
# exercise it:
herdr plugin action invoke herdr-mihi.<name>.<action>
herdr plugin pane open --plugin herdr-mihi.<name> --entrypoint <pane-id> --placement popup --focus
# inspect stdout/stderr/exit code of every plugin process:
herdr plugin log list --plugin herdr-mihi.<name>
herdr plugin config-dir herdr-mihi.<name>  # where to drop .env
```
- **Startup plugins (auto-title):** `[[startup]]` runs when the server starts the plugin — trigger with a
  `herdr server stop` + restart (or re-link). Config is read once at startup, so a settings change also
  needs a restart.
- **Keybindings:** add `[[keys.command]]` to your herdr `config.toml`, then `herdr server reload-config`.
- **herdr-remote caveat:** in default `--remote` mode herdr omits `plugin_action` bindings — keys silently
  no-op; use `--remote-keybindings server`.

## Tier 1 — unit tests (no herdr)

Pure logic, hermetic, fast. This is where most correctness lives.
- **Rust (auto-title, picker):** `cargo test`. Redirect config/state by setting `$HERDR_PLUGIN_CONFIG_DIR`
  / `$HERDR_PLUGIN_STATE_DIR` in-test (herdr-nvim's `ConfigEnvGuard`/`StateDirGuard` mutex pattern).
  **Golden fixtures** = canned herdr JSON → assert output: reuse herdr's `endpoint-snapshot-v1.json` /
  `session/current-herdr-session.json` and herdr-nvim's `layout_3pane.json` / `pane_get_with_session.json`.
  (auto-title unit-tests every resolver source + sanitize this way: `resolver/*_test`, `sanitize_test`.)
- **bash (lazygit):** `tests/<area>-test.sh` scripts (herdr-lazygit's model — plain scripts, no framework):
  assert the launcher decision logic (worktree-identity match, open/focus/close toggle) against faked cwds
  and pane lists.

## Tier 2 — mock-herdr tests (no live herdr, exercises the protocol)

Stand up a fake herdr and assert what the plugin *sends*.
- **Socket plugins (auto-title):** a **stub socket server** (port of `herdrtest/stub.go` to Rust) that
  speaks herdr's newline-delimited JSON (one request per connection), returns canned `session.snapshot` /
  `pane.process_info`, and **records `tab.rename` calls** to assert against. Protocol shapes are defined in
  `src/api/schema/*.rs`.
- **CLI plugins (picker, lazygit):** a **fake `herdr` on `$PATH`** (a script that logs its argv and returns
  canned JSON). Assert the picker builds the **exact** `plugin install … --ref <plugin>-latest` commands,
  that it **only ever names the pinned owner**, and that **nothing runs before confirm**. For lazygit, a
  fake `lazygit`/`git` on `$PATH` lets you assert the pane command + worktree scoping.

## Tier 3 — integration tests (real headless herdr)

End-to-end, mirroring herdr's own `tests/cli/plugins.rs`:
```
# in a temp HOME/runtime dir, spawn a real headless server, then drive via CLI --json
herdr server (headless) &  ; wait_for_socket
herdr plugin link plugins/<name>
herdr workspace create ; herdr plugin action invoke … ; herdr plugin pane open …
# assert on `herdr … --json` output + `herdr plugin log list`
```
- Runnable **in CI** (install herdr via its script) as a separate, slower job; keep it isolated
  (temp config/runtime dirs) so it can't touch a real session.
- Per shape: **auto-title** — link, open panes, assert titles track a scripted snapshot; **lazygit** —
  invoke `open`, assert a lazygit popup appears and toggles, worktree-scoped (fake `lazygit`);
  **picker** — run an install of a throwaway test plugin, assert it registered via `herdr plugin list`.

## Tier 4 — trust assertions ARE tests (the security spine, CI-enforced)

The capability declarations become CI gates:
- `cargo-deny` **bans networking crates**; `cargo-audit` for advisories (auto-title, picker).
- A test/grep asserts **no `std::process::Command`** in auto-title (its "no subprocess" promise); for lazygit
  assert **no `curl`/`wget`/network + no `python`**; for the picker assert **no non-pinned owner/URL** is ever
  constructed.
- `just check-sync` fails if a synced `/shared` file drifted (ties to ticket 006).

## CI shape (per plugin, via the `justfile`)
- **`ci.yml`** (push/PR): `just ci` = fmt/lint + Tier 1 + Tier 2 + Tier 4 (fast, no herdr).
- **`integration.yml`** (separate job): install herdr → Tier 3 headless integration.
- **Manual checklist** in each plugin README: the Tier 0 dev-loop steps a human/agent runs to eyeball it
  live (incl. the herdr-remote caveat).

## Deliverable
A root **`TESTING.md`** carrying this convention + the three per-shape recipes (daemon/poller, action+popup,
TUI+installer), and a `tests/` layout per plugin. Fixtures may be vendored from herdr's `tests/fixtures/`.
