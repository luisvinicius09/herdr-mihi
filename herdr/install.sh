#!/usr/bin/env bash
# ── herdr-mihi shared build step for COMPILED plugins — CANONICAL COPY (`just sync-shared`). ──
# Runs during `herdr plugin install` (after confirm, before registration). Downloads the prebuilt
# binary for this OS/arch from the pinned repo's release; falls back to building from source only
# if the download fails and a toolchain is present. It talks ONLY to the pinned repo.
set -euo pipefail

OWNER="luisvinicius09/herdr-mihi"
root="${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
manifest="$root/herdr-plugin.toml"

id="$(sed -n 's/^id *= *"\(.*\)"/\1/p' "$manifest" | head -1)"
name="${id##*.}"
ver="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$manifest" | head -1)"

case "$(uname -s)" in Darwin) os=darwin;; Linux) os=linux;; *) echo "herdr-mihi.$name: unsupported OS" >&2; exit 1;; esac
case "$(uname -m)" in arm64|aarch64) arch=arm64;; x86_64|amd64) arch=amd64;; *) echo "herdr-mihi.$name: unsupported arch" >&2; exit 1;; esac

asset="${name}-${os}-${arch}"
base="https://github.com/${OWNER}/releases/download/${name}-v${ver}"
mkdir -p "$root/bin"

echo "herdr-mihi.$name: fetching prebuilt $asset (v$ver)…" >&2
if curl -fsSL "$base/$asset" -o "$root/bin/$name" && curl -fsSL "$base/$asset.sha256" -o "$root/bin/$name.sha256"; then
  ( cd "$root/bin" && awk -v f="$name" '{print $1"  "f}' "$name.sha256" | shasum -a 256 -c - ) \
    || { echo "herdr-mihi.$name: checksum mismatch — refusing binary" >&2; exit 1; }
  chmod +x "$root/bin/$name"
  echo "herdr-mihi.$name: installed verified prebuilt binary." >&2
  exit 0
fi

echo "herdr-mihi.$name: no prebuilt binary; attempting source build…" >&2
if [[ -f "$root/Cargo.toml" ]] && command -v cargo >/dev/null 2>&1; then
  ( cd "$root" && cargo build --release ) && cp "$root/target/release/$name" "$root/bin/$name" && exit 0
fi
if [[ -f "$root/go.mod" ]] && command -v go >/dev/null 2>&1; then
  ( cd "$root" && go build -o "bin/$name" . ) && exit 0
fi

echo "herdr-mihi.$name: no prebuilt binary and no toolchain (need cargo or go). Try a newer release." >&2
exit 1
