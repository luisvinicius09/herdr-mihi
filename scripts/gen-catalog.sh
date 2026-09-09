#!/usr/bin/env bash
# Generate catalog.json from every plugin manifest (single source of truth = the manifests).
# Run via `just catalog`. The release CI enriches each entry with the resolved commit SHA.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

out="catalog.json"; tmp="$(mktemp)"
{
  echo '{'
  echo '  "owner": "luisvinicius09/herdr-mihi",'
  echo '  "plugins": ['
  first=1
  for m in plugins/*/herdr-plugin.toml; do
    [ -f "$m" ] || continue
    id=$(sed -n 's/^id *= *"\(.*\)"/\1/p' "$m" | head -1)
    name=${id##*.}
    ver=$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$m" | head -1)
    desc=$(sed -n 's/^description *= *"\(.*\)"/\1/p' "$m" | head -1)
    [ $first -eq 1 ] || echo '    ,'
    first=0
    printf '    { "id": "%s", "name": "%s", "version": "%s", "latest_ref": "%s-latest", "latest_commit": null, "description": "%s" }\n' \
      "$id" "$name" "$ver" "$name" "$desc"
  done
  echo '  ]'
  echo '}'
} > "$tmp"
mv "$tmp" "$out"
echo "wrote $out"
