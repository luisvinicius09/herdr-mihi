#!/usr/bin/env bash
# Everything CI runs — no herdr needed. Single source of truth for `just ci` AND
# .github/workflows/ci.yml, so the two never drift.
#   1) shared shims match the canonical copies in shared/
#   2) every shell script parses (bash -n)
#   3) per-plugin: cargo fmt/clippy/test (--locked) or gofmt/go test, plus bash unit tests
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

# 1) Shared shims must match shared/ (run.sh everywhere; install.sh for compiled plugins).
fail=0
for p in plugins/*/; do
  [ -f "${p}herdr/run.sh" ] && { diff -q shared/run.sh "${p}herdr/run.sh" >/dev/null || { echo "DRIFT: ${p}herdr/run.sh"; fail=1; }; }
  [ -f "${p}herdr/install.sh" ] && { diff -q shared/install.sh "${p}herdr/install.sh" >/dev/null || { echo "DRIFT: ${p}herdr/install.sh"; fail=1; }; }
done
[ "$fail" = 0 ] || { echo "shared shims drifted — run 'just sync-shared'"; exit 1; }
echo "shared shims in sync"

# 2) Syntax-check every shell script (cheap, no external tools).
while IFS= read -r -d '' f; do bash -n "$f"; done < <(find shared scripts plugins -name '*.sh' -print0)
echo "shell syntax ok"

# 3) Per-plugin lint + test.
for p in plugins/*/; do
  name=$(basename "$p")
  echo "== $name =="
  if [ -f "${p}Cargo.toml" ]; then ( cd "$p" && cargo fmt --check && cargo clippy --locked -- -D warnings && cargo test --locked ); fi
  if [ -f "${p}go.mod" ];     then ( cd "$p" && test -z "$(gofmt -l .)" && go test ./... ); fi
  for t in "${p}"tests/*-test.sh; do [ -f "$t" ] && bash "$t"; done
done
echo "ci ok"
