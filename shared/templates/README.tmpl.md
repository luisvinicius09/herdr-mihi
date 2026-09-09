# __NAME__

**Platforms: macOS · Linux**

__ONE-LINE PURPOSE__

## Install
```sh
herdr plugin install luisvinicius09/herdr-mihi --ref __NAME__-latest
```

## Setup
```sh
# if the plugin is configurable:
cp .env.example "$(herdr plugin config-dir herdr-mihi.__NAME__)/.env"
```

## Keybinding
herdr binds no keys by default. Add to your herdr `config.toml`, then `herdr server reload-config`:
```toml
[[keys.command]]
key = "prefix+g"
command = "herdr plugin action invoke herdr-mihi.__NAME__.open"
```
Over `--remote`, bindings need `--remote-keybindings server` or they silently no-op.

## Capability declaration (trust)
- **Spawns:** __none | list exactly__
- **Network:** __none | only luisvinicius09/herdr-mihi__
- **Files:** __reads/writes exactly__
- **No telemetry.**

## Behavior
__…__

## Requirements
__runtime versions + external tools__
