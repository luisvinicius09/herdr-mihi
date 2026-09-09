#!/usr/bin/env bash
# ── herdr-mihi shared wrapper — CANONICAL COPY. Edit here, then `just sync-shared`. ──
# herdr runs plugin commands shell-less with a minimal PATH. This normalizes PATH and
# execs the plugin's entrypoint: a compiled bin/<name>, else src/main.sh (bash plugins).
# The manifest only ever names this wrapper.
set -euo pipefail

export PATH="/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:${PATH:-}"

name="${HERDR_PLUGIN_ID##*.}"   # herdr-mihi.lazygit -> lazygit
root="${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

if [[ -x "$root/bin/$name" ]]; then
  exec "$root/bin/$name" "$@"
fi
if [[ -f "$root/src/main.sh" ]]; then
  exec bash "$root/src/main.sh" "$@"
fi

printf 'herdr-mihi.%s: no entrypoint (looked for bin/%s and src/main.sh)\n' "$name" "$name" >&2
exit 1
