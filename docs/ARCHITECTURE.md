# OpenPaste Architecture

## System Overview

OpenPaste follows a **daemon-client architecture** where a background Rust daemon handles all clipboard operations, storage, and business logic. Multiple clients (desktop app, CLI, future mobile apps) communicate with the daemon via IPC and REST API.

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Environment                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Desktop UI  │  │     CLI      │  │  REST API    │          │
│  │   (Tauri)    │  │   (Rust)     │  │   (Axum)     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         └─────────────────┼─────────────────┘                   │
│                           │                                     │
│                    IPC / HTTP                                   │
│                           │                                     │
│  ┌────────────────────────▼─────────────────────────────────┐   │
│  │                    OpenPaste Daemon                       │   │
│  │                        (Rust)                             │   │
│  ├───────────────────────────────────────────────────────────┤   │
│  │                                                           │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐       │   │
│  │  │   Platform  │  │   Events    │  │   Storage   │       │   │
│  │  │   Layer     │  │    Bus      │  │   Engine    │       │   │
│  │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘       │   │
│  │         │                 │                 │             │   │
│  │  ┌──────▼──────┐  ┌──────▼──────┐  ┌──────▼──────┐       │   │
│  │  │   Search    │  │ Encryption  │  │   Plugin    │       │   │
│  │  │   Engine    │  │   Module    │  │  Runtime    │       │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘       │   │
│  │                                                           │   │
│  └───────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Process Architecture

### Daemon Process (`openpaste-daemon`)

The daemon is a long-running background process that:

- Monitors system clipboard for changes
- Stores clipboard items in SQLite database
- Provides IPC endpoints for client communication
- Serves REST API on localhost:7890
- Loads and manages WebAssembly plugins
- Handles encryption/decryption operations
- Maintains full-text search index
- Manages retention and cleanup policies

**Lifecycle:**
1. Started by OS (systemd/launchd/Windows Service) or user login
2. Initializes database, loads plugins, starts clipboard watcher
3. Listens for IPC and HTTP requests
4. Runs until system shutdown or explicit stop command

### Desktop Client (`openpaste-desktop`)

Tauri-based desktop application that:

- Provides graphical user interface
- Communicates with daemon via IPC
- Shows search results and clipboard history
- Manages settings and preferences
- Displays notifications
- Handles keyboard shortcuts

**Lifecycle:**
1. Launched by user (or auto-start on login)
2. Connects to running daemon (starts daemon if not running)
3. Opens main window or tray icon
4. Runs until user closes

### CLI Client (`openpaste`)

Command-line tool that:

- Provides terminal-based clipboard access
- Communicates with daemon via IPC
- Supports scripting and automation
- Can run commands without UI

**Lifecycle:**
1. Invoked by user or script
2. Connects to daemon (starts if needed)
3. Executes command
4. Exits with result

## Module Decomposition

### Core Crates

#### `clipboard-core`

**Purpose:** Core clipboard operations and data structures

**Responsibilities:**
- Clipboard item data models
- Content type detection and normalization
- Duplicate detection
- Content validation
- Metadata extraction

**Dependencies:**
- `clipboard-utils` (shared types)

**Used by:**
- `clipboard-platform`
- `clipboard-db`
- All other crates

#### `clipboard-platform`

**Purpose:** Platform-specific clipboard access

**Responsibilities:**
- Windows clipboard API integration
- Linux Wayland clipboard access
- Linux X11 clipboard access
- macOS clipboard API
- Clipboard change detection
- Format conversion

**Dependencies:**
- `clipboard-core`
- Platform-specific crates (`clipboard-win`, `x11-clipboard`, etc.)

**Used by:**
- Daemon

#### `clipboard-db`

**Purpose:** Database abstraction layer

**Responsibilities:**
- SQLite connection management
- Schema migrations
- CRUD operations for clipboard items
- Transaction management
- Connection pooling

**Dependencies:**
- `clipboard-core`
- `rusqlite`
- `clipboard-encryption` (for encrypted databases)

**Used by:**
- `clipboard-storage`
- `clipboard-search`

#### `clipboard-storage`

**Purpose:** High-level storage operations

**Responsibilities:**
- Storing clipboard items
- Retrieving clipboard items
- Compression/decompression
- Image handling
- Binary data storage
- Retention policy enforcement

**Dependencies:**
- `clipboard-core`
- `clipboard-db`
- `clipboard-encryption`
- `zstd` (compression)

**Used by:**
- Daemon
- `clipboard-sync`

#### `clipboard-index`

**Purpose:** Full-text search indexing

**Responsibilities:**
- FTS5 index management
- Index updates on clipboard changes
- Index optimization
- Tokenization
- Language detection

**Dependencies:**
- `clipboard-db`
- SQLite FTS5

**Used by:**
- `clipboard-search`

#### `clipboard-search`

**Purpose:** Query processing and ranking

**Responsibilities:**
- Query parsing
- Search execution
- Result ranking (BM25, recency, frequency)
- Regex support
- Highlighting
- Filtering

**Dependencies:**
- `clipboard-core`
- `clipboard-index`
- `clipboard-db`

**Used by:**
- Daemon
- REST API
- CLI

#### `clipboard-encryption`

**Purpose:** Cryptographic operations

**Responsibilities:**
- Master password derivation (Argon2id)
- Key management
- AES-256-GCM encryption/decryption
- Secure memory handling
- Key rotation

**Dependencies:**
- RustCrypto libraries
- `zeroize`

**Used by:**
- `clipboard-db`
- `clipboard-storage`
- `clipboard-sync`

#### `clipboard-events`

**Purpose:** Event bus and pub/sub

**Responsibilities:**
- Event type definitions
- Channel management
- Publish/subscribe
- Async event handling
- Backpressure management

**Dependencies:**
- `tokio`
- `clipboard-core`

**Used by:**
- All crates that need to react to changes

#### `clipboard-sync`

**Purpose:** Synchronization protocols

**Responsibilities:**
- Sync protocol implementation
- Conflict resolution
- Change tracking
- Network operations
- WebDAV/S3/Git backends

**Dependencies:**
- `clipboard-core`
- `clipboard-storage`
- `clipboard-encryption`
- HTTP client libraries

**Used by:**
- Daemon

#### `clipboard-api`

**Purpose:** REST API server

**Responsibilities:**
- HTTP server (Axum)
- Endpoint handlers
- Request validation
- Response serialization
- Authentication
- Rate limiting
- WebSocket support

**Dependencies:**
- `clipboard-core`
- `clipboard-search`
- `clipboard-storage`
- `clipboard-events`
- `axum`
- `tokio`

**Used by:**
- Daemon

#### `clipboard-cli`

**Purpose:** Command-line interface

**Responsibilities:**
- Command parsing
- Argument validation
- IPC communication
- Output formatting
- Shell completion

**Dependencies:**
- `clipboard-core`
- IPC client library
- `clap`

**Used by:**
- CLI binary

#### `clipboard-plugin`

**Purpose:** WebAssembly plugin runtime

**Responsibilities:**
- WASM runtime (Wasmtime)
- Plugin lifecycle management
- Permission enforcement
- Host API implementation
- Sandboxing
- Plugin discovery

**Dependencies:**
- `wasmtime`
- `clipboard-core`
- `clipboard-events`
- `clipboard-storage`

**Used by:**
- Daemon

#### `clipboard-ai`

**Purpose:** AI-powered features

**Responsibilities:**
- Content categorization
- Automatic summarization
- Embedding generation
- Offline model integration
- API integration (optional)

**Dependencies:**
- `clipboard-core`
- ML libraries (candle, ort, etc.)
- HTTP client (for cloud APIs)

**Used by:**
- Daemon
- Plugins

#### `clipboard-utils`

**Purpose:** Shared utilities

**Responsibilities:**
- Common data structures
- Error types
- Logging configuration
- Testing utilities
- Benchmarking helpers

**Dependencies:**
- Standard library only

**Used by:**
- All other crates

## Layer Architecture

### Presentation Layer

- **Desktop UI:** Tauri + React + TypeScript
- **CLI:** Rust command-line interface
- **REST API:** HTTP endpoints for external integration

### Application Layer

- **IPC Handler:** Inter-process communication
- **REST Handler:** HTTP request processing
- **Plugin Manager:** WASM plugin lifecycle
- **Event Bus:** Internal event distribution

### Domain Layer

- **Clipboard Operations:** Core clipboard logic
- **Search Engine:** Query processing and ranking
- **Storage Engine:** Data persistence
- **Encryption:** Cryptographic operations
- **Sync:** Synchronization logic

### Infrastructure Layer

- **Database:** SQLite with FTS5
- **Platform APIs:** OS-specific clipboard access
- **File System:** Configuration and data storage
- **Network:** Sync and API communication

## Data Flow

### Clipboard Capture Flow

```
System Clipboard
       │
       ▼
clipboard-platform (detect change)
       │
       ▼
clipboard-core (normalize content)
       │
       ▼
clipboard-encryption (encrypt if enabled)
       │
       ▼
clipboard-storage (compress and store)
       │
       ▼
clipboard-db (persist to SQLite)
       │
       ▼
clipboard-index (update FTS index)
       │
       ▼
clipboard-events (publish ClipboardAdded event)
       │
       ▼
Clients (desktop, CLI, plugins) receive update
```

### Search Flow

```
User Query
       │
       ▼
clipboard-search (parse query)
       │
       ▼
clipboard-index (execute FTS search)
       │
       ▼
clipboard-search (rank results)
       │
       ▼
clipboard-storage (retrieve full items)
       │
       ▼
clipboard-encryption (decrypt if needed)
       │
       ▼
Return results to client
```

### Plugin Execution Flow

```
Plugin Request
       │
       ▼
clipboard-plugin (load WASM module)
       │
       ▼
clipboard-plugin (check permissions)
       │
       ▼
clipboard-plugin (execute in sandbox)
       │
       ▼
Host API (access clipboard, storage, etc.)
       │
       ▼
Return result to plugin
       │
       ▼
Plugin returns result
       │
       ▼
Return to caller
```

## Threading Model

### Main Thread

- Clipboard monitoring (platform-specific)
- IPC request handling
- Event loop

### Worker Threads

- Database operations (blocking I/O)
- Encryption/decryption (CPU-intensive)
- Compression/decompression
- Search index updates
- Plugin execution (WASM runtime)

### Async Runtime

- Tokio async runtime for:
  - HTTP server
  - WebSocket connections
  - Network operations (sync)
  - Timer-based tasks (cleanup, retention)

## Design Decisions

### Why Daemon-Client Architecture?

**Pros:**
- Single source of truth for clipboard data
- Multiple clients can share state
- CLI works without UI
- Easier to add new clients (mobile, browser extension)
- Better resource utilization (one daemon, many clients)
- Cleaner separation of concerns

**Cons:**
- More complex deployment
- Need IPC mechanism
- Daemon lifecycle management

**Decision:** Benefits outweigh complexity. Enables extensibility and multiple client types.

### Why SQLite?

**Pros:**
- Cross-platform
- Embedded (no separate server)
- Mature and reliable
- Excellent performance for our use case
- FTS5 for full-text search
- Small footprint
- Easy backup

**Cons:**
- Limited write concurrency
- Not suitable for distributed systems

**Decision:** Perfect fit for local-first clipboard manager. Write concurrency not an issue (single daemon).

### Why WebAssembly for Plugins?

**Pros:**
- Language-agnostic (Rust, Go, AssemblyScript)
- Sandboxed execution
- Near-native performance
- Portable across platforms
- Easy to distribute (single .wasm file)

**Cons:**
- Limited host API access
- WASM file size overhead
- Tooling complexity

**Decision:** Provides best balance of safety, performance, and extensibility.

### Why Tauri for Desktop?

**Pros:**
- Rust backend (shared with daemon)
- Smaller bundle size than Electron
- Better performance
- Native OS integration
- Web technologies for UI (familiar to many)

**Cons:**
- Smaller ecosystem than Electron
- Less mature (though rapidly improving)

**Decision:** Aligns with Rust-first philosophy, better resource usage.

### Why REST API + IPC?

**IPC (for local clients):**
- Lower latency
- No HTTP overhead
- Can use binary serialization
- Better for frequent updates

**REST API (for external integration):**
- Language-agnostic
- Easy to use from any tool
- Standard authentication
- Firewall-friendly

**Decision:** Use both - IPC for desktop/CLI, REST for external tools and future clients.

## Dependency Graph

```
clipboard-utils (no dependencies)
    │
    ├── clipboard-core
    │       │
    │       ├── clipboard-platform
    │       ├── clipboard-db
    │       ├── clipboard-events
    │       └── clipboard-encryption
    │
    ├── clipboard-storage
    │       ├── clipboard-core
    │       ├── clipboard-db
    │       └── clipboard-encryption
    │
    ├── clipboard-index
    │       └── clipboard-db
    │
    ├── clipboard-search
    │       ├── clipboard-core
    │       ├── clipboard-index
    │       └── clipboard-db
    │
    ├── clipboard-sync
    │       ├── clipboard-core
    │       ├── clipboard-storage
    │       └── clipboard-encryption
    │
    ├── clipboard-api
    │       ├── clipboard-core
    │       ├── clipboard-search
    │       ├── clipboard-storage
    │       └── clipboard-events
    │
    ├── clipboard-cli
    │       └── clipboard-core
    │
    ├── clipboard-plugin
    │       ├── clipboard-core
    │       ├── clipboard-events
    │       └── clipboard-storage
    │
    └── clipboard-ai
            └── clipboard-core
```

## Security Architecture

### Defense in Depth

1. **Process Isolation:** Daemon runs as separate process
2. **Encryption at Rest:** All sensitive data encrypted
3. **Sandboxing:** Plugins run in WASM sandbox
4. **Permission System:** Plugins require explicit permissions
5. **Memory Protection:** Sensitive data zeroed after use
6. **Authentication:** IPC and API require authentication
7. **Audit Logging:** Optional logging of clipboard access

### Threat Model

See [SECURITY.md](SECURITY.md) for detailed threat model.

## Performance Architecture

### Performance Targets

- **Search Latency:** < 20ms for 10,000 items
- **Clipboard Capture:** < 50ms from clipboard change to storage
- **Startup Time:** < 500ms for daemon
- **Memory Usage:** < 100MB with 1,000 items
- **Database Size:** < 50MB for 10,000 text items

### Optimization Strategies

- **FTS5 Indexing:** Fast full-text search
- **Connection Pooling:** Reuse database connections
- **Async I/O:** Non-blocking operations
- **Compression:** zstd for large text
- **Lazy Loading:** Load clipboard content on demand
- **Caching:** Cache frequent searches
- **Batch Operations:** Batch database writes

## Scalability Considerations

### Single-User Design

OpenPaste is designed for single-user, single-machine use:
- No multi-tenancy
- No distributed architecture
- No horizontal scaling

### Future Scaling

If needed, can scale by:
- Multiple daemon instances (one per user)
- Distributed sync (for multi-device)
- Sharded databases (for very large histories)

## Extensibility Points

### Plugin System

Plugins can extend:
- Clipboard processing (transform content)
- Search behavior (custom ranking)
- Storage backends (cloud storage)
- Sync protocols (custom sync)
- UI components (desktop app)
- AI features (custom models)

### Client API

External tools can integrate via:
- REST API
- WebSocket events
- CLI commands

### Configuration

Extensible configuration system for:
- User preferences
- Plugin settings
- Sync providers
- Custom keybindings

## Error Handling Strategy

### Error Types

- **Recoverable Errors:** Retry with backoff
- **Transient Errors:** Log and continue
- **Permanent Errors:** Log and notify user
- **Critical Errors:** Shutdown daemon

### Error Propagation

- Use Rust's `Result<T, E>` throughout
- Custom error types per crate
- Error context with `anyhow` or `eyre`
- Structured error logging

## Logging Strategy

### Log Levels

- **ERROR:** Critical errors requiring attention
- **WARN:** Warning conditions
- **INFO:** Normal operational messages
- **DEBUG:** Detailed debugging information
- **TRACE:** Very detailed tracing

### Log Destinations

- **Development:** stdout/stderr
- **Production:** File with rotation
- **Optional:** Syslog/journald integration

### Sensitive Data

- Never log clipboard content
- Never log encryption keys
- Sanitize error messages

## Testing Strategy

See [TESTING.md](TESTING.md) for comprehensive testing strategy.

## Deployment Architecture

### Development

- Run daemon directly from cargo
- Desktop app in development mode
- Local database in project directory

### Production

- Daemon installed as system service
- Desktop app installed to Applications/Program Files
- Database in user data directory
- Configuration in user config directory

### Package Distribution

- **Windows:** MSI installer
- **macOS:** DMG with app bundle
- **Linux:** AppImage, deb, rpm

## Monitoring and Observability

### Metrics

- Clipboard capture rate
- Search latency
- Database size
- Memory usage
- Plugin execution time
- Error rates

### Health Checks

- Daemon liveness (IPC ping)
- Database integrity
- Clipboard watcher status
- Plugin health

### Debugging

- Structured logging
- Optional debug mode
- Performance profiling
- Memory profiling
