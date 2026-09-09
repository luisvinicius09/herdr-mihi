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
      n=$((n+1))
    done
    echo "synced shared/ into $n plugin(s)"

# CI gate: fail if any plugin's synced shims drifted from shared/.
check-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    fail=0
    for p in plugins/*/; do
      [ -f "${p}herdr/run.sh" ] || continue
      diff -q shared/run.sh "${p}herdr/run.sh" >/dev/null || { echo "DRIFT: ${p}herdr/run.sh"; fail=1; }
    done
    [ "$fail" = 0 ] && echo "shared shims in sync" || exit 1

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

# Everything CI runs (no herdr needed): sync check + per-plugin lint/test + bash tests.
ci: check-sync
    #!/usr/bin/env bash
    set -euo pipefail
    for p in plugins/*/; do
      name=$(basename "$p")
      echo "== $name =="
      if [ -f "${p}Cargo.toml" ]; then ( cd "$p" && cargo fmt --check && cargo clippy -- -D warnings && cargo test ); fi
      if [ -f "${p}go.mod" ];     then ( cd "$p" && test -z "$(gofmt -l .)" && go test ./... ); fi
      for t in "${p}"tests/*-test.sh; do [ -f "$t" ] && bash "$t"; done
    done
    echo "ci ok"
