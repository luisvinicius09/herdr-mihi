# herdr-mihi

**Platforms: macOS · Linux**

A collection of [herdr](https://herdr.dev) plugins I build and trust — self-authored, minimal,
auditable, and provably safe. Pick what you use.

## Why this exists — trust-first

I don't run terminal plugins I can't verify. Every plugin here is built to a hard security spine:

- **No hidden network** — no telemetry; networking is called out explicitly where it exists.
- **No unexpected subprocess / shell** — external input is treated as hostile.
- **Minimal, pinned, audited dependencies** — enforced in CI (`cargo-deny`, `cargo-audit`).
- **A capability declaration** in every plugin's README — exactly what it spawns, touches, and
  reaches — verifiable by reading it.

## Install

Each plugin is published to its own release ref, so **only the plugin you pick lands on your machine**:

```sh
herdr plugin install luisvinicius09/herdr-mihi --ref <plugin>-latest
# e.g.
herdr plugin install luisvinicius09/herdr-mihi --ref picker-latest
```

Or install the **picker** and browse/install the rest — it shows you the exact install commands before
running anything.

## Plugins (phase 1)

| Plugin | Lang | What |
|---|---|---|
| `auto-title` | Rust | tab titles that follow the work in each tab |
| `lazygit`    | bash | a git popup running **your own** lazygit |
| `picker`     | Rust | browse & install the collection (shows exact commands first) |

## Layout

```
plugins/<name>/   one self-contained plugin each (subtree-split-friendly)
shared/           canonical shims (run.sh, install.sh) + templates, synced into each plugin
scripts/          catalog generator, etc.
justfile          task runner:  just sync-shared | check-sync | new-plugin | ci | catalog
TESTING.md        dev loop + the four test tiers
```

## Develop

```sh
just new-plugin <name>              # scaffold from templates
herdr plugin link plugins/<name>    # register your working tree (runs no build)
```

See [TESTING.md](TESTING.md) for the full dev loop and testing tiers.

## License

TBD (the herdr ecosystem trends Apache-2.0).
