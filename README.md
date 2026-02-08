# live_server_rs

A minimal Rust-based static file server with live-reload support over WebSockets. It serves a local directory over HTTP and injects a tiny client script into HTML pages to trigger reloads when files change.

## Table of Contents

- Overview
- Features
- Requirements
- Install
- Quick Start
- Usage
- Live Reload Behavior
- File Serving Behavior
- Security Considerations
- Limitations
- Development
- Continuous Integration
- Release Process
- Troubleshooting
- License

## Overview

`live_server_rs` is a lightweight development server meant for local use. It watches a directory recursively and, when any file changes, it notifies connected browsers via a WebSocket endpoint. HTML files are modified on-the-fly to include the reload script; non-HTML files are served as-is with a best-effort MIME type.

The server currently binds to `127.0.0.1:3030` and expects browsers to connect to `ws://localhost:3030/livereload` for live reload notifications.

## Features

- Serves files from a local directory (default: current working directory).
- Injects a reload script into HTML responses.
- Watches for file changes (create/modify/remove) recursively.
- Notifies connected browsers over WebSocket to reload.
- Basic path traversal protection using canonical paths.
- Cross-platform builds via GitHub Actions and optional `cross` for Linux targets.

## Requirements

- Rust toolchain (stable).
- Cargo (comes with Rust toolchain).

Optional for cross-compilation:
- `cross` (for Linux ARM and i686 targets).

## Install

### From source (local build)

```bash
cargo build --release
```

The binary will be available at:

```
target/release/live_server_rs
```

## Quick Start

Serve the current directory:

```bash
cargo run --release
```

Serve a specific directory:

```bash
cargo run --release -- /path/to/site
```

Then open:

```
http://localhost:3030
```

If `index.html` exists in the chosen directory, it will be served for the root path.

## Usage

### Command line

```
live_server_rs [path]
```

- `path` (optional): Directory to serve. If omitted, the current working directory is used.

### Exit behavior

Stop the server with `Ctrl+C` in the terminal.

## Live Reload Behavior

When an HTML file is served, the server injects a small script into the response. The script:

- Opens a WebSocket connection to `ws://localhost:3030/livereload`.
- Listens for the text message `reload`.
- On receiving `reload`, triggers `location.reload()`.
- On WebSocket close, waits 1 second and then reloads the page to recover.

The server sends `reload` to all connected clients when the filesystem watcher receives create, modify, or remove events.

## File Serving Behavior

- The server responds to `GET` requests.
- Root path (`/`) is mapped to `index.html`.
- Non-root paths are served from the requested file path relative to the base directory.
- If a requested path is a directory and `index.html` exists within it, that file is served.
- MIME type is determined using `mime_guess` for non-HTML files.
- HTML files (`.html` or `.htm`) are modified to include the live reload script.

## Security Considerations

This server is intended for local development and is not hardened for production use.

- Path traversal is mitigated by canonicalizing the base directory and requested file path.
- Requests that resolve outside the base directory are rejected with a 404.
- The server binds to `127.0.0.1` to avoid exposing it to the network by default.

If you need to expose the server over a network, add your own network controls and consider TLS termination and authentication.

## Limitations

- The host and port are fixed to `127.0.0.1:3030`.
- The WebSocket URL is hardcoded to `ws://localhost:3030/livereload`.
- No HTTPS support.
- No directory listing.
- No caching controls or compression configuration.
- The reload script is injected using a simple `</body>` string replacement.
- Large HTML files are fully loaded into memory for injection.

## Development

### Format and lint

```bash
cargo fmt
```

### Run tests

```bash
cargo test
```

### Run in debug mode

```bash
cargo run
```

### Cross compilation

For Linux cross-compilation targets, install and use `cross`:

```bash
cargo install cross
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target armv7-unknown-linux-gnueabihf
cross build --release --target i686-unknown-linux-gnu
```

## Continuous Integration

GitHub Actions builds and tests multiple targets. The workflow:

- Uses `cargo` for native builds and `cross` for Linux ARM and i686 targets.
- Caches Cargo registry, git index, and build output per target.
- Runs tests only on native x86_64 targets for Linux, Windows, and macOS.

Build matrix (current targets):

- Linux: `x86_64-unknown-linux-gnu` (native), `aarch64-unknown-linux-gnu` (cross), `armv7-unknown-linux-gnueabihf` (cross), `i686-unknown-linux-gnu` (cross)
- Windows: `x86_64-pc-windows-msvc`, `i686-pc-windows-msvc`, `aarch64-pc-windows-msvc` (all native)
- macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin` (native)

Artifacts are uploaded per target with unique names, for example `live_server_rs-linux-x86_64`.

## Release Process

A GitHub Release is created automatically when a tag is pushed.

Example:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow will:

1. Build all configured targets.
2. Create a release on GitHub.
3. Attach all binaries as release assets.

## Troubleshooting

### Browser does not reload

- Ensure you are accessing the site through `http://localhost:3030`.
- Check the browser console for WebSocket errors.
- Make sure the server is running and not blocked by a local firewall.

### Changes are not detected

- Confirm the file is inside the served directory.
- Some editors may write files atomically; the watcher still emits create/modify/remove events.

### 404 for a directory

- Ensure `index.html` exists inside the requested directory.
- If the directory does not contain `index.html`, the server returns 404.

## License

TBD. Replace this section with your chosen license.

