---
id: 001
title: "Research: how herdr fetches & stores a subdir plugin install"
labels: [wayfinder:research]
status: closed
assignee: Luis Vinicius <luisvinicius0906@gmail.com>
blocked_by: []
---

## Question

When a user runs `herdr plugin install owner/repo/subdir`, what actually lands on their
machine? Answer precisely, because it decides whether a monorepo can satisfy "don't bring
code the user didn't choose":

- Does herdr **clone the whole repo** or only the subdir (sparse/partial checkout)?
- **Where** is the checkout stored, and what is the **on-disk footprint** of installing one
  subdir out of a large monorepo?
- Does `herdr plugin install` from the **same repo, different subdir** re-clone, or reuse a
  shared checkout? What does **uninstall** of one plugin remove — does it leave siblings?
- Does `--ref` pinning and the build step operate per-subdir or per-repo?
- How does `herdr plugin link /path` differ (dev path)?

Sources: https://herdr.dev/docs/plugins/ and any herdr source/CLI help available. Produce a
markdown summary as a linked asset and state the bottom line: **can a single monorepo install
only-chosen-code, or do we need a different topology?** (Feeds ticket 005.)

## Resolution

**A single monorepo cannot honor "don't bring unchosen code."** `herdr plugin install owner/repo/subdir` shallow-clones
(`git init` + `git fetch --depth 1 origin HEAD` + `git checkout --detach FETCH_HEAD`) the **whole repo working tree** and
renames it into `<config>/plugins/github/<plugin_id>` — a **full copy per installed plugin-id**. The subdir is merely
located *inside* that checkout (`manifest_root = checkout.join(subdir)`). No sparse checkout, no blob filter; install
source is GitHub `owner/repo[/subdir]` shorthand only, with `--ref` and `--yes` the only flags. So installing N plugins
from one monorepo = **N full copies of the entire monorepo** on disk.

To satisfy the footprint preference, each installed plugin must map to a git ref whose tree is **only that plugin** —
separate repos, or per-plugin release branches/tags (subtree split) from a dev monorepo. A meta-installer over a plain
monorepo does **not** help footprint. `herdr plugin link <path>` (dev) sidesteps clone/build entirely.

Full write-up + source refs: [../assets/001-herdr-install-footprint.md](../assets/001-herdr-install-footprint.md).
Leads for the topology decision: herdr's `workers/plugin-marketplace/`; prior art `cloudmanic/herdr-plus`.
