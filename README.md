# zmux-scroll

An opinionated Zellij plugin for tracking scroll mode per terminal pane and restoring it as you focus around. Mostly in line with my experience of how tmux scroll mode behaves.
Additionally:
- it automatically switches to scroll mode when using the mouse wheel
- Switches out of scroll mode when switching pane from a scrolled pane

## Build

```sh
cargo build --release --target wasm32-wasip1
```

Build with debug logging compiled in:

```sh
cargo build --release --target wasm32-wasip1 --features debug-logs
```

The plugin artifact will be written to:

```text
target/wasm32-wasip1/release/zmux-scroll.wasm
```

## Run in Zellij

From inside a Zellij session, load or reload the background plugin with:

```sh
zellij action start-or-reload-plugin \
  "file:$PWD/target/wasm32-wasip1/release/zmux-scroll.wasm"
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

The plugin requests the Zellij permissions it needs on startup.
