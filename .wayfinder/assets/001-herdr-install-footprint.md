# Research: how herdr fetches & stores a subdir plugin install

**Ticket:** 001 · **Type:** research · **Source:** `herdrdev/herdr` @ `master` (Rust, Apache-2.0, v0.9.0)
Primary files: `src/cli/plugin.rs`, `src/plugin_paths.rs`.

## Bottom line

**A single monorepo cannot honor "don't bring unchosen code."** `herdr plugin install owner/repo/subdir`
shallow-clones the **entire repo working tree** and stores a **full copy per installed plugin-id**. There is
**no sparse checkout and no blob filter**. Installing one plugin from a monorepo brings *every* plugin's source
onto disk; installing N plugins from the same monorepo makes **N full copies** of the whole monorepo.

To honor the footprint preference, plugins the user installs must resolve to a **git ref whose tree contains only
that plugin** — i.e. separate repos, or per-plugin release branches/tags in a monorepo (see Options for ticket 005).

## Exactly how it fetches (`git_checkout`, `src/cli/plugin.rs`)

```rust
std::fs::create_dir_all(checkout)?;
run_git(checkout, ["init"]);
run_git(checkout, ["remote", "add", "origin", "https://github.com/OWNER/REPO.git"]);
// with --ref REF:   git fetch --depth 1 origin REF
// without:          git fetch --depth 1 origin HEAD
run_git(checkout, ["checkout", "--detach", "FETCH_HEAD"]);
```

- `--depth 1` → shallow: only **1 commit of history**, but the **complete working tree at that commit** (all
  subdirs). Shallow limits *history* size, not *tree* size.
- The subdir is **not** fetched selectively. `manifest_root(checkout) = checkout.join(subdir)` — herdr just looks
  *inside* the full checkout for the manifest.
- Source parsing is **GitHub-only shorthand**: `owner/repo[/subdir...]`. Full URLs / `git@` / other hosts are
  rejected ("plugin install v1 accepts only owner/repo[/subdir] shorthand"). `--ref REF` and `--yes` are the only flags.

## Install flow (`plugin_install`)

1. Parse `owner/repo[/subdir]` → `GithubPluginSource`.
2. `git_checkout` into a **temp** dir; `git rev-parse HEAD` records the resolved commit.
3. Load manifest from `manifest_root`, show a **preview**, prompt to confirm (`--yes`/`-y` to skip; **required** when
   stdin isn't a TTY).
4. Run the manifest's **build commands** (`run_plugin_build_commands`), then re-load the manifest and assert it's
   unchanged after build.
5. **Rename the whole temp checkout** → `managed_checkout_path(plugin_id)`. Prior install of the same id is moved
   aside and restored on failure (atomic-ish with rollback).
6. Register in the plugin registry; print config dir.

## On-disk layout (`src/plugin_paths.rs`)

| What | Path |
|---|---|
| Managed checkout (the cloned repo tree) | `<config_dir>/plugins/github/<sanitized-plugin_id>` |
| User config (editable, e.g. `.env`) | `<config_dir>/plugins/config/<sanitized-plugin_id>` |
| Durable runtime state | `<state_dir>/plugins/<sanitized-plugin_id>` |

- Keyed by **plugin_id**, not by repo → two plugins from one monorepo = two independent full-tree copies.
- `herdr plugin uninstall` removes only that plugin's managed checkout/config → copies are independent; removing one
  leaves siblings intact.
- `herdr plugin link /path` (dev) skips clone/build entirely and points at your working tree — good for local dev of
  the collection regardless of published topology.

## What this means for the footprint preference

- **Monorepo, plain subdir install** → worst case: brings all unchosen code, times N copies. Fails the preference.
- **Separate repo per plugin** → each install brings only that plugin. Satisfies the preference; fragments maintenance.
- **Monorepo + per-plugin release branch/tag (subtree split)** → maintain in one repo; publish each plugin to a ref
  whose *root tree is just that plugin*; users `herdr plugin install owner/monorepo --ref release/<plugin>` (or a thin
  per-plugin repo). Reconciles single-repo maintenance with minimal footprint, at the cost of a split/release step.
- **Meta-installer over a plain monorepo does NOT fix footprint** — it would just call `herdr plugin install` per
  pick, each still cloning the whole monorepo. A meta-installer only helps footprint if it installs from per-plugin
  refs/repos.

## Leads for ticket 005 (topology) — not investigated here

- **`workers/plugin-marketplace/` exists** in herdr (a Cloudflare-worker marketplace/index). v1 CLI install is
  GitHub-git only, so the marketplace is likely discovery, not an alternate install channel — but worth a quick look
  when deciding selection UX.
- **Prior art:** `cloudmanic/herdr-plus` ships "a collection of tools as a first-class herdr plugin" (a plugin-of-
  plugins); `yigitkonur/awesome-herdr` curates the ecosystem. Study how herdr-plus packages multiple tools.
