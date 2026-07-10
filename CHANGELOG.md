# Changelog

All notable changes to OpenPaste are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
OpenPaste uses [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

## [0.1.0] — 2026-07-10

### Added
- Clipboard history capture with 500 ms polling (text + images)
- SQLite storage with FTS5 full-text search
- AES-256-GCM encryption with Argon2id key derivation
- Auto-lock vault after configurable inactivity timeout
- Pin and favorite items
- Color-coded tags with sidebar filter
- WASM plugin system (wasmtime sandbox) — auto-loads `.wasm` files from the plugins directory
- Cross-device sync via HTTP push/pull relay server (`clipboard-api`)
- Global shortcuts: `⌘⇧V` show/hide, `⌘⇧C` quick-paste
- System tray with Show, Quick Paste, and Quit menu items
- Hide-to-tray on window close
- Launch at login (macOS LaunchAgent)
- `openpaste` CLI: list, search, copy, get, pin, favorite, delete, status
- Daemon auto-spawned by the desktop app; probe-before-spawn avoids double-start
- Settings panel: max items, retention days, refresh interval, notifications, encryption toggle
- Sync configuration UI (server URL, API token, enable toggle, Sync Now)
- Plugin Manager UI (load from file picker, list loaded plugins, unload)
- 26 automated tests (clipboard-db + clipboard-encryption)
- GitHub Actions CI (test + build-desktop, all platforms)
- GitHub Actions release workflow (macOS arm64/x64, Windows x64, Linux x64)
