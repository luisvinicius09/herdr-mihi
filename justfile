# herdr-mihi — collection task runner.  `just` lists recipes.
set shell := ["bash", "-uc"]

default:
    @just --list

# Copy the canonical shared shims into every plugin. Run after editing shared/.
sync-shared:
    #!/usr/bin/env bash
    set -euo pipefail
    n=0
    for p in plugins/*/; do
      [ -d "$p" ] || continue
      mkdir -p "${p}herdr"
      cp shared/run.sh "${p}herdr/run.sh"
      # install.sh only for compiled plugins (those that reference it in their manifest)
      if grep -q 'herdr/install.sh' "${p}herdr-plugin.toml" 2>/dev/null; then
        cp shared/install.sh "${p}herdr/install.sh"
      fi
      # canonical Rust modules -> src/shared/ for any plugin whose src declares `mod shared`
      if [ -d shared/rust ] && grep -rlq 'mod shared' "${p}src" 2>/dev/null; then
        mkdir -p "${p}src/shared"
        cp shared/rust/*.rs "${p}src/shared/"
      fi
      n=$((n+1))
    done
    echo "synced shared/ into $n plugin(s)"

# CI gate: fail if any plugin's synced shims drifted from shared/.
check-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    fail=0
    for p in plugins/*/; do
      [ -f "${p}herdr/run.sh" ] && { diff -q shared/run.sh "${p}herdr/run.sh" >/dev/null || { echo "DRIFT: ${p}herdr/run.sh"; fail=1; }; }
      [ -f "${p}herdr/install.sh" ] && { diff -q shared/install.sh "${p}herdr/install.sh" >/dev/null || { echo "DRIFT: ${p}herdr/install.sh"; fail=1; }; }
      if [ -d "${p}src/shared" ]; then
        for f in shared/rust/*.rs; do
          diff -q "$f" "${p}src/shared/$(basename "$f")" >/dev/null 2>&1 || { echo "DRIFT: ${p}src/shared/$(basename "$f")"; fail=1; }
        done
      fi
    done
    [ "$fail" = 0 ] && echo "shared shims + modules in sync" || exit 1

# Scaffold a new plugin:  just new-plugin <name>
new-plugin name:
    #!/usr/bin/env bash
    set -euo pipefail
    d="plugins/{{name}}"
    [ -e "$d" ] && { echo "$d already exists"; exit 1; }
    mkdir -p "$d/herdr" "$d/src" "$d/tests"
    sed "s/__NAME__/{{name}}/g" shared/templates/herdr-plugin.toml.tmpl > "$d/herdr-plugin.toml"
    sed "s/__NAME__/{{name}}/g" shared/templates/README.tmpl.md > "$d/README.md"
    printf 'bin/\ntarget/\n.env\n' > "$d/.gitignore"
    just sync-shared
    echo "created $d — fill in the manifest, README, and src/"

# Regenerate catalog.json from every plugin manifest.
catalog:
    bash scripts/gen-catalog.sh

# Publish a plugin as a release ref (dry run; add --push to publish): just release lazygit
release name *flags:
    bash scripts/release.sh {{name}} {{flags}}

# Everything CI runs (no herdr needed): sync check + shell syntax + per-plugin lint/test.
# Same script the GitHub workflow runs, so `just ci` == CI.
ci:
    bash scripts/ci.sh

# Dependency trust gate: licenses + advisories + sources (needs cargo-deny + cargo-audit).
audit:
    #!/usr/bin/env bash
    set -euo pipefail
    for p in plugins/*/; do
      [ -f "${p}Cargo.toml" ] || continue
      echo "== $(basename "$p") =="
      ( cd "$p" && cargo deny check --config "$PWD/../../deny.toml" )
      ( cd "$p" && cargo audit --deny warnings )
    done
