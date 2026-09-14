#!/usr/bin/env bash
# herdr-mihi.lazygit — thin, pure-bash herdr <-> lazygit glue.
#   open : open a lazygit popup rooted at the current worktree
#   pane : the popup body — runs YOUR lazygit, readable error on failure
# Capability: spawns lazygit (yours) / git / herdr CLI. Network: none. No state, no bundled binaries.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib.sh
source "$here/lib.sh"

HERDR="${HERDR_BIN_PATH:-herdr}"

envf="${HERDR_PLUGIN_CONFIG_DIR:-}/.env"
[ -n "${HERDR_PLUGIN_CONFIG_DIR:-}" ] && [ -f "$envf" ] && { set -a; . "$envf"; set +a; }
LAZYGIT_BIN="${LAZYGIT_BIN:-lazygit}"
# How lazygit opens (the setting `open` honors):
#   popup = floating overlay over the workspace (doesn't rearrange your split; session-modal)
#   side  = a split pane beside your work (workspace-tied, persistent)
PLACEMENT="${PLACEMENT:-popup}"
POPUP_WIDTH="${POPUP_WIDTH:-80%}"
POPUP_HEIGHT="${POPUP_HEIGHT:-80%}"

# _open <popup|side>: open lazygit via the matching declared pane, rooted at the current worktree.
_open() {
  local mode="$1" cwd root entry placement
  cwd="$(resolve_cwd "${HERDR_PLUGIN_CONTEXT_JSON:-}")"
  root="$(worktree_root "$cwd")"
  case "$mode" in
    side|split) entry="lazygit-side"; placement="split" ;;
    *)          entry="lazygit";      placement="popup" ;;
  esac
  local args=(plugin pane open --plugin herdr-mihi.lazygit --entrypoint "$entry"
    --placement "$placement" --cwd "$root" --focus)
  [ "$placement" = "popup" ] && args+=(--width "$POPUP_WIDTH" --height "$POPUP_HEIGHT")
  "$HERDR" "${args[@]}" >/dev/null 2>&1 \
    || { echo "herdr-mihi.lazygit: failed to open lazygit ($placement)" >&2; exit 1; }
}

cmd_open() { _open "$PLACEMENT"; }   # honors the PLACEMENT setting (default popup)
cmd_open_side() { _open side; }      # always the side pane

cmd_pane() {
  # runs inside the popup; herdr set cwd to the target worktree.
  if ! command -v "$LAZYGIT_BIN" >/dev/null 2>&1 && [ ! -x "$LAZYGIT_BIN" ]; then
    echo "herdr-mihi.lazygit: '$LAZYGIT_BIN' not found on PATH." >&2
    echo "  install it (e.g. 'brew install lazygit') or set LAZYGIT_BIN in your .env." >&2
    read -rp "press enter to close… " _ || true
    exit 0
  fi
  "$LAZYGIT_BIN" || {
    rc=$?
    echo "herdr-mihi.lazygit: lazygit exited with code $rc" >&2
    read -rp "press enter to close… " _ || true
    exit "$rc"
  }
}

# Opt-in: add the open keybind to the user's herdr config.toml (idempotent; prints what it adds).
cmd_setup() {
  local action="herdr-mihi.lazygit.open" key="prefix+g"
  local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/herdr/config.toml"
  [ -f "$cfg" ] || { echo "herdr-mihi.lazygit: herdr config not found at $cfg" >&2; exit 1; }
  if grep -q "invoke $action" "$cfg"; then
    echo "herdr-mihi.lazygit: keybind already configured in $cfg"; exit 0
  fi
  if grep -q "\"$key\"" "$cfg"; then
    echo "herdr-mihi.lazygit: key '$key' is already bound — add this yourself with a free key:" >&2
    printf '\n[[keys.command]]\nkey = "%s"\ntype = "shell"\ncommand = "%s plugin action invoke %s"\n' "$key" "$HERDR" "$action" >&2
    exit 0
  fi
  {
    printf '\n# --- herdr-mihi.lazygit (added by: herdr plugin action invoke herdr-mihi.lazygit.setup) ---\n'
    printf '[[keys.command]]\nkey = "%s"\ntype = "shell"\ncommand = "%s plugin action invoke %s"\n' "$key" "$HERDR" "$action"
  } >> "$cfg"
  echo "herdr-mihi.lazygit: added keybind '$key' -> $action in $cfg"
  "$HERDR" server reload-config >/dev/null 2>&1 \
    && echo "reloaded — press your prefix then 'g' in a repo pane to open lazygit" \
    || echo "run 'herdr server reload-config' to apply"
}

case "${1:-}" in
  open) cmd_open ;;
  open-side) cmd_open_side ;;
  pane) cmd_pane ;;
  setup) cmd_setup ;;
  *) echo "usage: run.sh {open|open-side|pane|setup}" >&2; exit 2 ;;
esac
