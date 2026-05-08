# AGENTS.md — rustfy

## What this is
A single-binary Rust tray app using GTK3 and `libappindicator` that bridges **ntfy** topics to **notify-send** (libnotify). It runs as a resident GUI; do not run it in headless environments.

## How it works
- Reads config from `$HOME/.config/rustfy/config.toml` (`base_url`, `topics`, `reconnect_delay`)
- Spawns one thread per topic to listen via SSE (Server-Sent Events) from the ntfy server
- When a message arrives, calls `notify-send` to display a native desktop notification
- Auto-reconnects with configurable delay if connection drops

## Build prerequisites
- **Rust 1.85+** (required for `edition = "2024"`)
- **System GTK3 / AppIndicator libraries**
  - Debian/Ubuntu: `libgtk-3-dev`, `libappindicator3-dev`
  - Fedora: `gtk3-devel`, `libappindicator-gtk3-devel`

## Runtime requirements
- A running **X11/Wayland display** (it calls `gtk::init()` and enters `gtk::main()`).
- Assets (`icon-enabled.png`, `icon-disabled.png`) are **embedded at build time** via `include_bytes!` and extracted to `/tmp/rustfy-assets-<pid>/` on startup. The binary does **not** need an external `assets/` folder at runtime.
- Config (`~/.config/rustfy/config.toml`) is **self-healing**: missing fields get populated from defaults, malformed files are recreated with defaults, and the file is rewritten on every startup to ensure it stays complete and up-to-date.

## Key dependencies
- `gtk` 0.18 with feature `v3_24`
- `gdk-pixbuf` 0.18 (for logo in About dialog)
- `libappindicator` 0.9
- `rich_rust` 0.2 (used for styled console output)
- `curl` 0.4 (libcurl bindings for HTTP/2 streaming)
- `notify-rust` 4.17 (native Rust notifications via D-Bus, replaces `notify-send` binary)
- Supports full ntfy message format: title, message, priority, tags (emojis), and icon
- Icon URLs are downloaded to `~/.cache/rustfy/icons/` and reused; stale icons (>2 days) are auto-cleaned
- Ignores cached/historical messages on startup — only processes new messages received after launch
- `chrono` 0.4 (ISO 8601 timestamps in console output)

## Common commands
```bash
# Build
cargo build --release

# Run (requires a display)
cargo run
```

## Version bump workflow
- The app prints `RustFy <version>` on startup, reading from `Cargo.toml` via `env!("CARGO_PKG_VERSION")`.
- When the user says **"atualize a versão"**, ask which semver component to bump (major / minor / patch), then run:
  ```bash
  ./scripts/bump-version.sh <major|minor|patch>
  cargo build --release
  ```
- The script updates `Cargo.toml` in-place. Rebuild to embed the new version in the binary.

## Tray menu
- Right-click menu shows **Sobre** and **Sair** only.
- **Sobre** opens a centered modal `gtk::AboutDialog` with the RustFy logo (`icon-enabled.png` scaled to 40%), description, author, GitHub link, and MIT license.

## Notes
- No tests, no CI, no workspace — this is a plain single-package crate.
- UI strings are in Portuguese; keep new menu items / labels consistent with the existing locale.
