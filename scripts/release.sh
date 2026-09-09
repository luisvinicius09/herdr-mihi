#!/usr/bin/env bash
# Publish one plugin as an installable release ref, WITHOUT git-subtree.
#
# herdr installs the manifest at a ref's ROOT, so we build a commit whose root tree IS the
# plugin's directory (core git plumbing: `git rev-parse HEAD:plugins/<name>` -> `git commit-tree`),
# tag it `<name>-v<version>` (immutable) and move `<name>-latest` to it. Works on macOS + Linux.
#
#   scripts/release.sh <plugin>            # dry run: create local tags only, print the push command
#   scripts/release.sh <plugin> --push     # also push the tags to origin
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

name="${1:?usage: release.sh <plugin> [--push]}"
push=false
[ "${2:-}" = "--push" ] && push=true

dir="plugins/$name"
[ -d "$dir" ] || { echo "no such plugin: $dir" >&2; exit 1; }
ver="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$dir/herdr-plugin.toml" | head -1)"
[ -n "$ver" ] || { echo "no version in $dir/herdr-plugin.toml" >&2; exit 1; }

# The ref must reflect committed files only.
[ -z "$(git status --porcelain -- "$dir" scripts catalog.json)" ] \
  || { echo "uncommitted changes affect the release — commit first" >&2; exit 1; }

tree="$(git rev-parse "HEAD:$dir")"
commit="$(git commit-tree "$tree" -m "release: $name v$ver")"
vtag="$name-v$ver"
latest="$name-latest"

if git rev-parse -q --verify "refs/tags/$vtag" >/dev/null; then
  echo "tag $vtag already exists (immutable). Bump the version in $dir/herdr-plugin.toml to re-release." >&2
  exit 1
fi
git tag "$vtag" "$commit"
git tag -f "$latest" "$commit"
echo "created $vtag and moved $latest -> ${commit:0:12}  (root tree = $dir)"

if $push; then
  git push origin "$vtag"
  git push -f origin "$latest"
  echo "pushed $vtag and $latest to origin"
  echo "install:  herdr plugin install $(git config --get remote.origin.url | sed -E 's#.*github.com[:/]##; s#\.git$##') --ref $latest"
else
  echo "dry run — local tags only. Publish with:  scripts/release.sh $name --push"
fi
