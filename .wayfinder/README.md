# Wayfinder tracker (local-markdown)

This directory is a [wayfinder](https://github.com/) map for the herdr plugin collection.
It's the **local-markdown tracker** (no GitHub remote yet). It has no UI, so blocking and
claiming are expressed in ticket frontmatter instead of native tracker relationships.

- **`map.md`** — the map (`wayfinder:map`). The low-resolution index: Destination, Notes,
  Decisions-so-far (closed tickets), Not-yet-specified (fog), Out-of-scope.
- **`tickets/NNN-*.md`** — the child tickets. Each has frontmatter + a `## Question`.

## Ticket frontmatter

```yaml
id: 001
title: "…"
labels: [wayfinder:research]   # research | prototype | grilling | task
status: open                   # open | closed
assignee: null                 # who's driving it — set this to CLAIM before any work
blocked_by: []                 # ticket ids that must be closed first
```

## Operations

- **Frontier** (what's takeable now) = tickets that are `status: open`, `assignee: null`,
  and whose every `blocked_by` id is `status: closed`.
  List candidates: `grep -rl "status: open" tickets/` then check `blocked_by`.
- **Claim** — set `assignee:` to yourself *before* doing any work, so parallel sessions skip it.
- **Resolve** — append a `## Resolution` section (the answer + links to any assets),
  set `status: closed`, then add a one-line gist to `map.md` → "Decisions so far".
- **Out of scope** — close the ticket and leave one line in `map.md` → "Out of scope".
- **Never resolve more than one ticket per session.**
