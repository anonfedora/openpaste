# OpenPaste

A modern, cross-platform clipboard manager with advanced features like encryption, search, and sync.

## Current Status

**Version:** 0.1.0 (Development)

**Status:** Active Development

## Features Implemented

### Phase 1: Foundation ✅
- Project architecture with Rust workspace
- Tauri + React desktop application
- Core clipboard capture (macOS)
- SQLite database with migrations
- IPC communication (Unix domain sockets)
- Basic search functionality
- Metallic UI design

### Phase 2: Core Features (In Progress)
- Clipboard history management
- Pin and favorite items
- Delete items
- Copy to clipboard
- Auto-refresh (2-second polling)
- Deduplication (hash-based)
- Custom title bar with window controls
- FTS5 full-text search with triggers
- Zstd compression for storage
- AES-256-GCM encryption with proper nonce handling
- Argon2id key derivation with secure parameters

## Technology Stack

**Backend:**
- Rust
- SQLite (with sqlx)
- Tokio async runtime
- Unix domain sockets (IPC)

**Frontend:**
- React + TypeScript
- Tauri
- Tailwind CSS
- Lucide React icons

## Running the Application

### Prerequisites
- Rust (latest stable)
- Node.js (18+)
- pnpm

### Development

1. Start the clipboard daemon:
```bash
cargo run --bin openpaste-daemon
```

2. Start the desktop app:
```bash
cd apps/desktop
pnpm install
pnpm tauri dev
```

## Project Structure

```
openpaste/
├── apps/
│   └── desktop/          # Tauri desktop application
├── crates/
│   ├── clipboard-core/   # Core clipboard types
│   ├── clipboard-db/     # Database operations
│   ├── clipboard-ipc/    # IPC communication
│   └── clipboard-daemon/ # Background daemon
└── docs/                 # Documentation
```

## Roadmap

See [ROADMAP.md](docs/ROADMAP.md) for detailed planning.

## Contributing

See [CONTRIBUTING.md](docs/CONTRIBUTING.md) for contribution guidelines.

## License

TBD