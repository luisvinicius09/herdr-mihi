#!/usr/bin/env bash
# Tier-1 unit tests (no herdr): the pure helpers in src/lib.sh.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=../src/lib.sh
source "$here/../src/lib.sh"

fail=0
ok()  { printf '  ok   %s\n' "$1"; }
bad() { printf '  FAIL %s\n' "$1"; fail=1; }

# --- json_str ---
[ "$(printf '%s' '{"focused_pane_cwd":"/a/b","workspace_cwd":"/a"}' | json_str focused_pane_cwd)" = "/a/b" ] \
  && ok "json_str extracts a field" || bad "json_str extracts a field"
[ -z "$(printf '%s' '{"workspace_cwd":"/a"}' | json_str focused_pane_cwd)" ] \
  && ok "json_str empty on missing field" || bad "json_str empty on missing field"

# --- resolve_cwd ---
[ "$(resolve_cwd '{"focused_pane_cwd":"/x","workspace_cwd":"/y"}')" = "/x" ] \
  && ok "resolve_cwd prefers focused pane" || bad "resolve_cwd prefers focused pane"
[ "$(resolve_cwd '{"workspace_cwd":"/y"}')" = "/y" ] \
  && ok "resolve_cwd falls back to workspace" || bad "resolve_cwd falls back to workspace"
[ "$(HOME=/home/tester resolve_cwd '{}')" = "/home/tester" ] \
  && ok "resolve_cwd falls back to HOME" || bad "resolve_cwd falls back to HOME"

# --- worktree_root (non-git fallback) ---
[ "$(worktree_root /no/such/dir/xyz)" = "/no/such/dir/xyz" ] \
  && ok "worktree_root falls back to the dir for non-git paths" || bad "worktree_root falls back to the dir for non-git paths"

[ "$fail" -eq 0 ] && echo "lazygit: all unit tests passed" || echo "lazygit: FAILURES"
exit "$fail"
