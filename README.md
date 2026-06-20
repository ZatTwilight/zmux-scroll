# zmux-scroll

An opinionated Zellij plugin for tracking scroll mode per terminal pane and restoring it as you focus around. Mostly in line with my experience of how tmux scroll mode behaves.
Additionally:
- it automatically switches to scroll mode when using the mouse wheel
- Switches out of scroll mode when switching pane from a scrolled pane

## Install / run

### Option 1: GitHub release URL

From inside a Zellij session, load or reload the latest release directly from GitHub:

```sh
zellij action start-or-reload-plugin \
  "https://github.com/ZatTwilight/zmux-scroll/releases/latest/download/zmux-scroll.wasm"
```

To pin a specific version, use that release tag instead of `latest`:

```sh
zellij action start-or-reload-plugin \
  "https://github.com/ZatTwilight/zmux-scroll/releases/download/v0.1.0/zmux-scroll.wasm"
```

### Option 2: Local build

```sh
cargo build --release --target wasm32-wasip1
```

The plugin artifact will be written to:

```text
target/wasm32-wasip1/release/zmux-scroll.wasm
```

Then load it from inside a Zellij session:

```sh
zellij action start-or-reload-plugin \
  "file:$PWD/target/wasm32-wasip1/release/zmux-scroll.wasm"
```

Build with debug logging compiled in:

```sh
cargo build --release --target wasm32-wasip1 --features debug-logs
```

## Dev reload

From inside a Zellij session, rebuild, copy to your local Zellij plugin directory, and reload:

```sh
./scripts/reload
```

By default this writes to:

```text
~/.config/zellij/plugins/zmux-scroll.wasm
```

Override the destination directory if needed:

```sh
ZELLIJ_PLUGIN_DIR=/path/to/plugins ./scripts/reload
```

Build and reload with debug logging compiled in:

```sh
ZMUX_SCROLL_DEBUG=1 ./scripts/reload
```

Debug logging is a compile-time feature (`debug-logs`), not a runtime plugin configuration option.

## Releases

GitHub Actions builds the WASM artifact on pushes and pull requests. Pushing a version tag like `v0.1.0` creates or updates a GitHub release with:

- `zmux-scroll.wasm`
- `zmux-scroll.wasm.sha256`

The plugin requests the Zellij permissions it needs on startup.
