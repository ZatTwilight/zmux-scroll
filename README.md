# zmux-scroll

An opinionated Zellij plugin for tracking scroll mode per terminal pane and restoring it as you focus around. Mostly in line with my experience of how tmux scroll mode behaves.
Additionally:
- it automatically switches to scroll mode when using the mouse wheel
- Switches out of scroll mode when switching pane from a scrolled pane

## Build

```sh
cargo build --release --target wasm32-wasip1
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

The plugin requests the Zellij permissions it needs on startup.
