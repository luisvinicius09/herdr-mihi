#!/usr/bin/env bash
# Pure helpers for herdr-mihi.lazygit — NO herdr calls, NO network, NO side effects.
# Unit-tested in tests/launcher-decision-test.sh. Defensive: never assume a JSON field exists.

# json_str <key> : read JSON on stdin, print the first "<key>":"<string>" value (or empty).
json_str() {
  local key="$1"
  sed -n "s/.*\"$key\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" | head -1
}

# resolve_cwd <context-json> : focused pane cwd -> workspace cwd -> $HOME.
resolve_cwd() {
  local j="$1" c
  c=$(printf '%s' "$j" | json_str focused_pane_cwd); [ -n "$c" ] && { printf '%s' "$c"; return; }
  c=$(printf '%s' "$j" | json_str workspace_cwd);     [ -n "$c" ] && { printf '%s' "$c"; return; }
  printf '%s' "${HOME:-/}"
}

# worktree_root <dir> : the git worktree top-level, else the dir itself (bare/non-git safe).
worktree_root() {
  local d="$1" top
  top=$(git -C "$d" rev-parse --show-toplevel 2>/dev/null) && { printf '%s' "$top"; return; }
  printf '%s' "$d"
}
