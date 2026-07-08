# OpenPaste Technology Stack

## Overview

This document explains every technology choice in OpenPaste, alternatives considered, and the tradeoffs involved.

## Backend Technologies

### Rust

**Version:** 1.75+

**Why Rust?**
- **Memory Safety**: No null pointer dereferences, no data races
- **Performance**: Zero-cost abstractions, predictable performance
- **Cross-Platform**: First-class support for Windows, Linux, macOS
- **Concurrency**: Excellent async/await support with Tokio
- **Ecosystem**: Mature crates for all our needs (database, crypto, networking)
- **Package Management**: Cargo is excellent for dependency management
- **Tooling**: rustfmt, clippy, cargo test, cargo doc
- **WebAssembly**: Native WASM support for plugins

**Alternatives Considered:**
- **Go**: Good concurrency, but garbage collector adds latency
- **C++**: Performance is great, but memory safety concerns, build complexity
- **Python**: Too slow for clipboard operations, not ideal for daemon
- **Node.js**: Single-threaded, not suitable for system-level operations

**Tradeoffs:**
- **Learning Curve**: Rust has a steep learning curve
- **Compile Times**: Slower than Go/Python, but acceptable
- **Ecosystem Size**: Smaller than JavaScript/Python, but growing rapidly

**Usage:**
- Daemon process
- All backend crates
- CLI tool
- Plugin runtime host

### Tokio

**Version:** 1.35+

**Why Tokio?**
- **Mature**: Most widely used async runtime in Rust
- **Performance**: Highly optimized, zero-cost futures
- **Ecosystem**: Most libraries use Tokio
- **Features**: Timer, IO, net, sync utilities included
- **Compatibility**: Works well with Tauri

**Alternatives Considered:**
- **async-std**: Good alternative, but smaller ecosystem
- **smol**: Lightweight, but less feature-rich

**Tradeoffs:**
- **Complexity**: Async adds complexity to code
- **Debugging**: Async backtraces can be harder to debug

**Usage:**
- Async runtime for daemon
- HTTP server (Axum)
- WebSocket connections
- Network operations (sync)
- Timer-based tasks

### SQLite

**Version:** 3.40+

**Why SQLite?**
- **Embedded**: No separate database server to manage
- **Cross-Platform**: Works identically on all platforms
- **Mature**: Battle-tested, extremely reliable
- **Performance**: Excellent for our read-heavy workload
- **FTS5**: Built-in full-text search extension
- **Small**: Single binary, minimal dependencies
- **ACID**: Full transaction support
- **Backup**: Easy backup with single file copy

**Alternatives Considered:**
- **PostgreSQL**: Overkill for single-user local storage
- **MySQL**: Same as PostgreSQL, requires separate server
- **RocksDB**: Great performance, but no SQL, more complex
- **Sled**: Rust-native, but less mature than SQLite
- **Redis**: Requires separate server, overkill for local use

**Tradeoffs:**
- **Write Concurrency**: Limited write concurrency (not an issue for single daemon)
- **Distributed**: Not suitable for distributed systems (not needed)

**Usage:**
- Primary database for clipboard storage
- FTS5 full-text search index
- Metadata storage
- Configuration storage

### Rusqlite

**Version:** 0.30+

**Why Rusqlite?**
- **Mature**: Most widely used SQLite library for Rust
- **Feature-Rich**: Supports FTS5, blobs, custom functions
- **Performance**: Efficient bindings
- **Synchronous**: Good for our use case (single-threaded writes)

**Alternatives Considered:**
- **sqlx**: Async, compile-time checked queries, but more complex
- **sqlite**: Lower-level, less ergonomic

**Tradeoffs:**
- **Synchronous**: Blocking I/O (we handle with thread pool)
- **No Compile-Time Query Checking**: Runtime errors possible (mitigated with tests)

**Usage:**
- Database connection management
- Query execution
- Migration management

### RustCrypto

**Libraries:**
- **aes-gcm**: 0.10+ (AES-256-GCM encryption)
- **argon2**: 0.5+ (password hashing)
- **chacha20poly1305**: 0.10+ (alternative cipher)
- **rand**: 0.8+ (cryptographically secure random)
- **sha2**: 0.10+ (SHA-256 hashing)

**Why RustCrypto?**
- **Pure Rust**: No C dependencies, easier cross-compilation
- **Audited**: Well-audited cryptographic implementations
- **Modern**: Supports modern algorithms (Argon2, ChaCha20)
- **Performance**: Optimized implementations
- **Maintained**: Active maintenance and security updates

**Alternatives Considered:**
- **OpenSSL**: Mature, but C dependencies, licensing concerns
- **libsodium**: Excellent, but C dependencies

**Tradeoffs:**
- **API Surface**: Can be complex, but we abstract in clipboard-encryption
- **Documentation**: Some lack detailed examples

**Usage:**
- Master password derivation (Argon2id)
- Database encryption (AES-256-GCM)
- Secure random number generation
- Hashing operations

### Zeroize

**Version:** 1.7+

**Why Zeroize?**
- **Secure Memory**: Zeroes memory when values are dropped
- **No-Op in Release**: Zero-cost in release builds (compiler optimizes)
- **Well-Tested**: Extensive test coverage
- **Simple API**: Easy to use

**Alternatives Considered:**
- **Manual zeroing**: Error-prone, compiler may optimize away
- **secrecy**: Similar, but zeroize is more focused on secure memory

**Tradeoffs:**
- **Debug Builds**: Zeroing happens in debug (slower, but acceptable)
- **Not Foolproof**: Can't guarantee memory is never swapped to disk

**Usage:**
- Zeroing encryption keys after use
- Zeroing master password from memory
- Zeroing sensitive clipboard content

### Zstd

**Version:** 0.13+

**Why Zstd?**
- **Performance**: Fast compression and decompression
- **Ratio**: Good compression ratio
- **Streaming**: Supports streaming compression
- **Rust Implementation**: Pure Rust binding available

**Alternatives Considered:**
- **gzip**: Slower, worse ratio
- **lz4**: Faster, but worse ratio
- **brotli**: Better ratio, but slower

**Tradeoffs:**
- **CPU Usage**: Compression uses CPU (acceptable for background daemon)
- **Complexity**: Adds complexity to storage layer

**Usage:**
- Compressing large text clipboard items
- Compressing HTML content
- Reducing database size

### Axum

**Version:** 0.7+

**Why Axum?**
- **Tokio-Native**: Built on Tokio, perfect fit
- **Ergonomic**: Extractor-based API, easy to use
- **Performance**: Highly optimized
- **Type-Safe**: Compile-time route checking
- **WebSocket Support**: Built-in WebSocket support
- **Middleware**: Excellent middleware system

**Alternatives Considered:**
- **Actix Web**: More mature, but more complex
- **Rocket**: Excellent, but requires nightly Rust
- **Warp**: Functional style, but steeper learning curve

**Tradeoffs:**
- **Maturity**: Newer than Actix, but rapidly maturing
- **Ecosystem**: Smaller than Actix, but growing

**Usage:**
- REST API server (localhost:7890)
- WebSocket server for real-time updates
- Request validation
- Response serialization

### Serde

**Version:** 1.0+

**Why Serde?**
- **De Facto Standard**: Most widely used serialization library
- **Performance**: Zero-cost deserialization
- **Ergonomic**: Derive macros make it easy
- **Format Support**: JSON, bincode, YAML, etc.
- **Type-Safe**: Compile-time type checking

**Alternatives Considered:**
- **Manual serialization**: Error-prone, verbose
- **rkyv**: Zero-copy, but more complex

**Tradeoffs:**
- **Runtime Reflection**: Some overhead (acceptable)
- **Binary Size**: Adds to binary size (acceptable)

**Usage:**
- JSON serialization for REST API
- Bincode serialization for IPC
- Configuration file parsing
- Plugin manifest parsing

### Bincode

**Version:** 2.0+

**Why Bincode?**
- **Binary Format**: Compact, efficient binary serialization
- **Performance**: Fast serialization/deserialization
- **No Schema**: Schemaless, easy to use
- **Rust-Native**: Designed for Rust

**Alternatives Considered:**
- **MessagePack**: More language support, but slower
- **CBOR**: Similar, but less Rust-optimized
- **JSON**: Too verbose, slower

**Tradeoffs:**
- **Language Support**: Rust-only (acceptable for IPC)
- **Versioning**: No built-in versioning (we handle manually)

**Usage:**
- IPC message serialization
- Efficient data transfer between daemon and clients

### Clap

**Version:** 4.4+

**Why Clap?**
- **Ergonomic**: Derive macros make CLI definition easy
- **Feature-Rich**: Subcommands, arguments, flags, validation
- **Help Generation**: Automatic help and man page generation
- **Shell Completion**: Automatic shell completion
- **Type-Safe**: Compile-time argument checking

**Alternatives Considered:**
- **StructOpt**: Deprecated in favor of Clap 4
- **pico-args**: Minimalist, but less features

**Tradeoffs:**
- **Compile Time**: Derive macros increase compile time (acceptable)
- **Binary Size**: Adds to binary size (acceptable)

**Usage:**
- CLI argument parsing
- Command definition
- Help text generation
- Shell completion

### Anyhow

**Version:** 1.0+

**Why Anyhow?**
- **Ergonomic**: Easy error handling with context
- **No Boilerplate**: Less verbose than thiserror
- **Compatible**: Works with any error type
- **Context**: Easy to add error context

**Alternatives Considered:**
- **thiserror**: More structured, but more boilerplate
- **eyre**: Similar, but anyhow is more widely used

**Tradeoffs:**
- **Type Safety**: Less type-safe than thiserror (acceptable for application code)
- **Performance**: Slight overhead (acceptable)

**Usage:**
- Error handling in application code
- Adding context to errors
- Error propagation

### Thiserror

**Version:** 1.0+

**Why Thiserror?**
- **Structured**: Define error types with fields
- **Type-Safe**: Compile-time error type checking
- **Library-Friendly**: Perfect for library crates
- **Display**: Easy to implement Display

**Alternatives Considered:**
- **anyhow**: Less structured, more for application code
- **failure**: Deprecated

**Tradeoffs:**
- **Boilerplate**: More verbose than anyhow (acceptable for libraries)

**Usage:**
- Error types in library crates
- Public API error definitions
- Structured error information

### Tracing

**Version:** 0.1+

**Why Tracing?**
- **Structured**: Structured logging with spans
- **Performance**: Zero-cost when disabled
- **Ecosystem**: Wide ecosystem support
- **Async-Native**: Designed for async/await
- **Filters**: Fine-grained log filtering

**Alternatives Considered:**
- **log**: Traditional logging, less structured
- **slog**: Similar, but tracing is more modern

**Tradeoffs:**
- **Complexity**: More complex than log (acceptable for benefits)
- **Learning Curve**: New concepts (spans, subscribers)

**Usage:**
- Structured logging
- Performance tracing
- Debugging async operations
- Request tracing in API

### Wasmtime

**Version:** 15.0+

**Why Wasmtime?**
- **Performance**: Fast WASM runtime
- **Security**: Sandboxed execution
- **WASI Support**: WebAssembly System Interface
- **Embedding**: Easy to embed in Rust applications
- **Active Development**: Rapidly improving
- **Component Model**: Support for WASM components

**Alternatives Considered:**
- **wasmer**: Similar, but Wasmtime has better WASI support
- **wasm-bindgen**: For browser, not for server-side

**Tradeoffs:**
- **Complexity**: WASM sandboxing adds complexity
- **Performance**: Slight overhead vs native (acceptable for plugins)

**Usage:**
- WebAssembly plugin runtime
- Sandboxed plugin execution
- Host API implementation

## Desktop Technologies

### Tauri

**Version:** 1.5+

**Why Tauri?**
- **Rust Backend**: Shared backend with daemon
- **Small Bundle**: Much smaller than Electron
- **Performance**: Better performance than Electron
- **Security**: More secure by default
- **Native Integration**: Better OS integration
- **Web Technologies**: Familiar web tech for UI

**Alternatives Considered:**
- **Electron**: Larger bundle, more resource usage
- **Flutter**: Good, but Dart learning curve
- **Qt Native**: Excellent, but C++ complexity
- **GTK/Rust Native**: Good, but more complex UI development

**Tradeoffs:**
- **Ecosystem**: Smaller than Electron (but growing)
- **Maturity**: Newer than Electron (but stable)
- **Learning Curve**: Requires Rust + web skills

**Usage:**
- Desktop application framework
- Window management
- System tray integration
- Native OS APIs
- IPC between frontend and backend

### React

**Version:** 18.2+

**Why React?**
- **Ecosystem**: Largest ecosystem of components and libraries
- **Performance**: Virtual DOM, efficient updates
- **Developer Experience**: Excellent DX with hooks
- **TypeScript Support**: First-class TypeScript support
- **Community**: Large community, lots of resources

**Alternatives Considered:**
- **Vue**: Simpler, but smaller ecosystem
- **Svelte**: Smaller bundle, but smaller ecosystem
- **Solid**: Excellent performance, but smaller ecosystem

**Tradeoffs:**
- **Bundle Size**: Larger than Svelte/Solid (acceptable with Tauri)
- **Complexity**: Hooks can be complex (but well-understood)

**Usage:**
- UI component library
- State management
- User interface
- React hooks for logic

### TypeScript

**Version:** 5.3+

**Why TypeScript?**
- **Type Safety**: Catch errors at compile time
- **IDE Support**: Excellent autocomplete and refactoring
- **Modern**: Modern JavaScript features
- **Ecosystem**: Widely adopted in React ecosystem
- **Refactoring**: Safe refactoring with types

**Alternatives Considered:**
- **JavaScript**: No type safety, more runtime errors
- **Flow**: Similar, but less popular

**Tradeoffs:**
- **Build Step**: Requires compilation (acceptable)
- **Learning Curve**: Types add complexity (but worth it)

**Usage:**
- Type-safe UI development
- Component props
- API client types
- Configuration types

### TanStack Query

**Version:** 5.0+

**Why TanStack Query?**
- **Data Fetching**: Excellent data fetching and caching
- **React Integration**: Perfect React integration
- **Type Safety**: Full TypeScript support
- **Background Updates**: Automatic background refetching
- **Optimistic Updates**: Easy optimistic updates

**Alternatives Considered:**
- **SWR**: Similar, but TanStack Query has more features
- **RTK Query**: Good, but tied to Redux
- **Apollo**: GraphQL-focused, overkill for REST

**Tradeoffs:**
- **Learning Curve**: New concepts (queries, mutations, cache)
- **Bundle Size**: Adds to bundle (acceptable)

**Usage:**
- Data fetching from REST API
- Caching clipboard items
- Optimistic updates
- Background synchronization

### TanStack Router

**Version:** 1.0+

**Why TanStack Router?**
- **Type Safety**: Fully typed routes
- **Code Splitting**: Automatic code splitting
- **Search Params**: Type-safe search params
- **Data Loading**: Integrated data loading
- **Modern**: Modern router design

**Alternatives Considered:**
- **React Router**: Less type-safe
- **Next.js**: Overkill for desktop app
- **Remix**: Full framework, overkill

**Tradeoffs:**
- **Newer**: Newer than React Router (but stable)
- **Learning Curve**: New concepts (file-based routing)

**Usage:**
- Client-side routing
- Page navigation
- Search params
- Code splitting

### Tailwind CSS

**Version:** 3.4+

**Why Tailwind CSS?**
- **Utility-First**: Fast UI development
- **Small Bundle**: Purges unused styles
- **Customizable**: Highly customizable
- **Responsive**: Easy responsive design
- **Dark Mode**: Built-in dark mode support

**Alternatives Considered:**
- **CSS Modules**: More verbose, harder to maintain
- **Styled Components**: Runtime overhead, larger bundle
- **Emotion**: Similar to Styled Components

**Tradeoffs:**
- **HTML Verbosity**: More verbose HTML (acceptable)
- **Learning Curve**: New utility class names

**Usage:**
- UI styling
- Responsive design
- Dark mode
- Custom design system

### Vite

**Version:** 5.0+

**Why Vite?**
- **Fast Development**: Instant HMR
- **Fast Builds**: Optimized builds
- **ESM Native**: Native ES modules
- **Plugin Ecosystem**: Rich plugin ecosystem
- **TypeScript Support**: Built-in TypeScript support

**Alternatives Considered:**
- **Webpack**: Slower, more complex
- **esbuild**: Faster, but less feature-rich
- **Rollup**: Good, but Vite is simpler

**Tradeoffs:**
- **Newer**: Newer than Webpack (but mature enough)
- **Plugin Ecosystem**: Smaller than Webpack (but sufficient)

**Usage:**
- Development server
- Production builds
- HMR
- TypeScript compilation

## Platform-Specific Libraries

### Windows

**clipboard-win**
- **Purpose**: Windows clipboard API
- **Why**: Mature, well-maintained, Rust-native

**windows-rs**
- **Purpose**: Windows API bindings
- **Why**: Official Microsoft bindings, comprehensive

### Linux

**x11-clipboard**
- **Purpose**: X11 clipboard access
- **Why**: Mature, supports X11 clipboard

**wayland-clipboard**
- **Purpose**: Wayland clipboard access
- **Why**: Supports Wayland clipboard protocol

**smithay-clipboard**
- **Purpose**: Unified Wayland/X11 clipboard
- **Why**: Abstraction over both protocols

### macOS

**cocoa**
- **Purpose**: macOS Cocoa framework
- **Why**: Official bindings, comprehensive

**objc**
- **Purpose**: Objective-C runtime
- **Why**: Needed for Cocoa interop

## Testing Technologies

### Rust Testing

**cargo test**
- Built-in test runner
- Unit tests
- Integration tests
- Doc tests

**criterion**
- Benchmarking library
- Statistical benchmarking
- Performance regression detection

**proptest**
- Property-based testing
- Fuzzing
- Edge case discovery

### JavaScript Testing

**Vitest**
- Fast unit test runner
- Vite-native
- TypeScript support

**Playwright**
- E2E testing
- Cross-browser testing
- Desktop app testing

**Testing Library**
- Component testing
- User-centric testing
- React integration

## Development Tools

### Rust

**rustfmt**
- Code formatting
- Consistent style

**clippy**
- Linting
- Catch common mistakes

**cargo-watch**
- Watch mode for development
- Auto-run tests on changes

**cargo-edit**
- Easy dependency management
- cargo add, cargo upgrade

### JavaScript/TypeScript

**ESLint**
- Linting
- Code quality

**Prettier**
- Code formatting
- Consistent style

**TypeScript**
- Type checking
- Compile-time error detection

## CI/CD

### GitHub Actions

**Why GitHub Actions?**
- Integrated with GitHub
- Free for open source
- Good runner support (Windows, Linux, macOS)
- Large action ecosystem

**Usage:**
- CI testing on all platforms
- Release automation
- Security scanning

### cargo-nextest**

**Why cargo-nextest?**
- Faster test execution
- Better test output
- Parallel test execution

**Usage:**
- Faster CI runs
- Better test feedback

## Documentation

### mdBook

**Why mdBook?**
- Rust-native
- Easy to use
- Good for technical docs
- Searchable

### cargo-doc

**Why cargo-doc?**
- Auto-generate API docs
- Integrated with Rust
- Hostable on docs.rs

## Version Requirements

### Minimum Rust Version

**1.75+**

Required for:
- Async trait improvements
- Const generics stability
- Standard library improvements

### Node.js Version

**18+** (for development)

Required for:
- Vite
- TypeScript compiler
- npm packages

### Platform Support

**Windows**: 10+
**Linux**: Kernel 5.4+ (for Wayland support)
**macOS**: 11+ (Big Sur)

## Future Migration Paths

### Database

**SQLite → Tantivy** (for search)
- If SQLite FTS5 performance is insufficient
- Tantivy offers more advanced search features
- Migration path: hybrid SQLite + Tantivy

### Encryption

**AES-256-GCM → ChaCha20Poly1305**
- If AES hardware acceleration unavailable
- ChaCha20 is faster without hardware acceleration
- Migration path: support both, choose based on hardware

### Search

**FTS5 → Hybrid FTS5 + Tantivy**
- Keep SQLite for storage
- Use Tantivy for advanced search
- Migration path: gradual, feature flag

### Plugin Runtime

**Wasmtime → Wasmtime + Wasmi**
- Wasmi for smaller footprint
- Wasmtime for performance
- Migration path: choose based on plugin needs

## Summary

OpenPaste's technology stack prioritizes:
- **Performance**: Rust, Tokio, SQLite, Axum
- **Security**: RustCrypto, zeroize, sandboxing
- **Cross-Platform**: Rust, Tauri, SQLite
- **Developer Experience**: Cargo, Vite, TypeScript
- **Extensibility**: WASM, plugin system, REST API

All choices are mature, well-maintained, and have strong community support.
