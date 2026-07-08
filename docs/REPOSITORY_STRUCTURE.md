# OpenPaste Repository Structure

## Directory Tree

```
openpaste/
├── .github/
│   ├── workflows/
│   │   ├── ci.yml
│   │   ├── release.yml
│   │   └── security.yml
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── plugin_proposal.md
│   └── PULL_REQUEST_TEMPLATE.md
│
├── apps/
│   ├── desktop/
│   │   ├── src-tauri/
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── src/
│   │   │   │   ├── main.rs
│   │   │   │   └── lib.rs
│   │   │   └── icons/
│   │   ├── src/
│   │   │   ├── components/
│   │   │   ├── pages/
│   │   │   ├── hooks/
│   │   │   ├── stores/
│   │   │   ├── utils/
│   │   │   ├── App.tsx
│   │   │   └── main.tsx
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── tailwind.config.js
│   │   └── vite.config.ts
│   │
│   └── cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│
├── crates/
│   ├── clipboard-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs
│   │       ├── content.rs
│   │       └── metadata.rs
│   │
│   ├── clipboard-db/
│   │   ├── Cargo.toml
│   │   ├── migrations/
│   │   │   ├── 001_initial.sql
│   │   │   ├── 002_fts.sql
│   │   │   └── 003_encryption.sql
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs
│   │       ├── models.rs
│   │       └── schema.rs
│   │
│   ├── clipboard-index/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fts.rs
│   │       └── tokenizer.rs
│   │
│   ├── clipboard-search/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── query.rs
│   │       ├── ranking.rs
│   │       └── filters.rs
│   │
│   ├── clipboard-storage/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── store.rs
│   │       ├── compression.rs
│   │       └── images.rs
│   │
│   ├── clipboard-sync/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── protocol.rs
│   │       ├── conflict.rs
│   │       └── backends/
│   │           ├── mod.rs
│   │           ├── webdav.rs
│   │           ├── s3.rs
│   │           └── git.rs
│   │
│   ├── clipboard-api/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── server.rs
│   │       ├── handlers/
│   │       │   ├── mod.rs
│   │       │   ├── clipboard.rs
│   │       │   ├── search.rs
│   │       │   └── sync.rs
│   │       ├── websocket.rs
│   │       └── auth.rs
│   │
│   ├── clipboard-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ipc.rs
│   │       └── output.rs
│   │
│   ├── clipboard-plugin/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── runtime.rs
│   │       ├── host_api.rs
│   │       ├── permissions.rs
│   │       └── sandbox.rs
│   │
│   ├── clipboard-encryption/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── key_derivation.rs
│   │       ├── cipher.rs
│   │       └── memory.rs
│   │
│   ├── clipboard-platform/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── mod.rs
│   │       ├── windows.rs
│   │       ├── linux.rs
│   │       ├── macos.rs
│   │       └── common.rs
│   │
│   ├── clipboard-events/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bus.rs
│   │       ├── types.rs
│   │       └── subscription.rs
│   │
│   ├── clipboard-ai/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── categorize.rs
│   │       ├── summarize.rs
│   │       └── embeddings.rs
│   │
│   └── clipboard-utils/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs
│           ├── logging.rs
│           └── time.rs
│
├── plugins/
│   ├── examples/
│   │   ├── hello-world/
│   │   │   ├── Cargo.toml
│   │   │   ├── manifest.json
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   ├── url-detector/
│   │   │   ├── Cargo.toml
│   │   │   ├── manifest.json
│   │   │   └── src/
│   │   │       └── lib.rs
│   │   └── markdown-formatter/
│   │       ├── Cargo.toml
│   │       ├── manifest.json
│   │       └── src/
│   │           └── lib.rs
│   │
│   └── registry/
│       └── plugins.json
│
├── docs/
│   ├── PROJECT.md
│   ├── ARCHITECTURE.md
│   ├── REPOSITORY_STRUCTURE.md
│   ├── STACK.md
│   ├── DATABASE.md
│   ├── SEARCH.md
│   ├── STORAGE.md
│   ├── ENCRYPTION.md
│   ├── PLATFORM.md
│   ├── EVENTS.md
│   ├── IPC.md
│   ├── UI.md
│   ├── DESIGN_SYSTEM.md
│   ├── PLUGIN_SDK.md
│   ├── AI.md
│   ├── REST_API.md
│   ├── CLI.md
│   ├── CONTRIBUTING.md
│   ├── TESTING.md
│   ├── RELEASE.md
│   ├── SECURITY.md
│   ├── ROADMAP.md
│   └── TASKS.md
│
├── scripts/
│   ├── build.sh
│   ├── test.sh
│   ├── release.sh
│   ├── install.sh
│   ├── dev.sh
│   └── format.sh
│
├── tests/
│   ├── integration/
│   │   ├── clipboard_test.rs
│   │   ├── search_test.rs
│   │   └── sync_test.rs
│   ├── e2e/
│   │   ├── desktop_test.rs
│   │   └── cli_test.rs
│   └── fixtures/
│       ├── sample_text.txt
│       ├── sample_image.png
│       └── sample_html.html
│
├── assets/
│   ├── icons/
│   │   ├── icon-16x16.png
│   │   ├── icon-32x32.png
│   │   ├── icon-64x64.png
│   │   ├── icon-128x128.png
│   │   ├── icon-256x256.png
│   │   └── icon-512x512.png
│   ├── logos/
│   │   ├── logo-horizontal.svg
│   │   └── logo-vertical.svg
│   └── screenshots/
│       ├── search.png
│       ├── settings.png
│       └── collections.png
│
├── Cargo.toml (workspace root)
├── Cargo.lock
├── README.md
├── LICENSE-APACHE
├── LICENSE-MIT
├── CONTRIBUTING.md
├── CHANGELOG.md
├── .gitignore
├── .editorconfig
├── rustfmt.toml
└── clippy.toml
```

## Directory Explanations

### Root Files

- **Cargo.toml**: Workspace configuration defining all member crates
- **Cargo.lock**: Locked dependency versions (committed for reproducibility)
- **README.md**: Project overview, quick start, and links to documentation
- **LICENSE-APACHE / LICENSE-MIT**: Dual license files
- **CONTRIBUTING.md**: Contribution guidelines (links to docs/CONTRIBUTING.md)
- **CHANGELOG.md**: Version history and changes
- **.gitignore**: Git ignore patterns (build artifacts, IDE files, OS files)
- **.editorconfig**: Editor configuration for consistent formatting
- **rustfmt.toml**: Rust code formatter configuration
- **clippy.toml**: Rust linter configuration

### .github/

GitHub-specific configuration:
- **workflows/**: CI/CD pipeline definitions
- **ISSURE_TEMPLATE/**: Templates for creating issues
- **PULL_REQUEST_TEMPLATE.md**: Template for PR descriptions

### apps/

Application binaries and frontends:
- **desktop/**: Tauri desktop application
- **cli/**: Command-line interface

### crates/

Rust library crates (workspace members):
- **clipboard-core**: Core data structures and types
- **clipboard-db**: Database abstraction
- **clipboard-index**: Full-text search indexing
- **clipboard-search**: Query processing and ranking
- **clipboard-storage**: Data storage operations
- **clipboard-sync**: Synchronization logic
- **clipboard-api**: REST API server
- **clipboard-cli**: CLI library (shared by CLI binary)
- **clipboard-plugin**: WebAssembly plugin runtime
- **clipboard-encryption**: Cryptographic operations
- **clipboard-platform**: Platform-specific clipboard access
- **clipboard-events**: Event bus
- **clipboard-ai**: AI-powered features
- **clipboard-utils**: Shared utilities

### plugins/

WebAssembly plugins:
- **examples/**: Example plugins for developers
- **registry/**: Plugin registry metadata

### docs/

Project documentation (all markdown files):
- Phase 1: Foundation documents
- Phase 2: Backend documents
- Phase 3: Platform documents
- Phase 4: Desktop documents
- Phase 5: Extensions documents
- Phase 6: Infrastructure documents
- Phase 7: Planning documents

### scripts/

Build and development scripts:
- **build.sh**: Build all components
- **test.sh**: Run all tests
- **release.sh**: Prepare release artifacts
- **install.sh**: Install OpenPaste locally
- **dev.sh**: Start development environment
- **format.sh**: Format code (Rust and TypeScript)

### tests/

Test suites:
- **integration/**: Integration tests (Rust)
- **e2e/**: End-to-end tests (Playwright for UI)
- **fixtures/**: Test data and assets

### assets/

Binary assets:
- **icons/**: Application icons in various sizes
- **logos/**: Project logos
- **screenshots/**: Screenshots for documentation

## Naming Conventions

### Files

- **Rust**: `snake_case.rs` (e.g., `key_derivation.rs`)
- **TypeScript**: `PascalCase.tsx` for components, `camelCase.ts` for utilities
- **Markdown**: `UPPER_SNAKE_CASE.md` for documentation
- **SQL**: `###_description.sql` for migrations (zero-padded numbers)
- **JSON**: `kebab-case.json` for configuration
- **Shell scripts**: `kebab-case.sh`

### Directories

- **Rust crates**: `kebab-case` (e.g., `clipboard-core`)
- **TypeScript directories**: `kebab-case` (e.g., `components`, `hooks`)
- **Documentation**: `UPPER_SNAKE_CASE.md` (in docs/)

### Code

- **Rust types**: `PascalCase` (e.g., `ClipboardItem`)
- **Rust functions**: `snake_case` (e.g., `get_clipboard_item`)
- **Rust constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_ITEMS`)
- **TypeScript types/interfaces**: `PascalCase`
- **TypeScript functions/variables**: `camelCase`
- **TypeScript constants**: `SCREAMING_SNAKE_CASE`

### Database

- **Tables**: `snake_case` (e.g., `clipboard_items`)
- **Columns**: `snake_case` (e.g., `created_at`)
- **Indexes**: `idx_table_columns` (e.g., `idx_clipboard_items_created_at`)

## Workspace Configuration

### Cargo.toml (Root)

```toml
[workspace]
members = [
    "crates/clipboard-core",
    "crates/clipboard-db",
    "crates/clipboard-index",
    "crates/clipboard-search",
    "crates/clipboard-storage",
    "crates/clipboard-sync",
    "crates/clipboard-api",
    "crates/clipboard-cli",
    "crates/clipboard-plugin",
    "crates/clipboard-encryption",
    "crates/clipboard-platform",
    "crates/clipboard-events",
    "crates/clipboard-ai",
    "crates/clipboard-utils",
    "apps/desktop/src-tauri",
    "apps/cli",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["OpenPaste Contributors"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/openpaste/openpaste"

[workspace.dependencies]
# Shared dependencies defined here
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"
```

### Package.json (Desktop)

```json
{
  "name": "openpaste-desktop",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tanstack/react-query": "^5.0",
    "@tanstack/react-router": "^1.0",
    "react": "^18.2",
    "react-dom": "^18.2"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^1.5",
    "typescript": "^5.3",
    "vite": "^5.0",
    "tailwindcss": "^3.4"
  }
}
```

## Build System

### Development

```bash
# Build all Rust components
cargo build

# Run tests
cargo test

# Format code
cargo fmt
./scripts/format.sh

# Lint code
cargo clippy
```

### Desktop App Development

```bash
cd apps/desktop
npm install
npm run tauri dev
```

### CLI Development

```bash
cd apps/cli
cargo run -- search "query"
```

### Production Build

```bash
./scripts/build.sh
```

This builds:
- All Rust crates (release mode)
- Desktop app (optimized bundle)
- CLI binary
- Plugins

## Monorepo Organization

### Why Monorepo?

- **Shared Code**: Rust crates shared between daemon, CLI, and desktop
- **Synchronized Releases**: All components versioned together
- **Simplified CI**: Single pipeline for all components
- **Easier Development**: Work on backend and frontend together
- **Consistent Tooling**: Shared formatting, linting, testing

### Workspace Benefits

- **Dependency Deduplication**: Shared dependencies compiled once
- **Atomic Changes**: Update multiple crates in one commit
- **Cross-Crate Testing**: Easy integration tests
- **Unified Documentation**: All docs in one place

### Versioning

- **Workspace Version**: All crates share version from workspace
- **Release Versioning**: Semantic versioning for entire project
- **Component Versioning**: Individual crates can have independent versions if needed

## Platform-Specific Code

### Directory Structure

Platform-specific code isolated in `clipboard-platform/src/`:
- `windows.rs`: Windows-specific implementation
- `linux.rs`: Linux-specific implementation (Wayland + X11)
- `macos.rs`: macOS-specific implementation
- `common.rs`: Cross-platform utilities
- `mod.rs`: Platform abstraction and conditional compilation

### Conditional Compilation

```rust
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;
```

### Testing Platform-Specific Code

Each platform tested on CI:
- Windows runners for Windows code
- Linux runners for Linux code
- macOS runners for macOS code

## Configuration Files

### Rust Configuration

- **rustfmt.toml**: Code formatting rules
- **clippy.toml**: Lint configuration
- **config.toml.example**: Example user configuration

### TypeScript Configuration

- **tsconfig.json**: TypeScript compiler options
- **vite.config.ts**: Vite bundler configuration
- **tailwind.config.js**: Tailwind CSS configuration

### Tauri Configuration

- **tauri.conf.json**: Tauri app configuration
- **icons/**: App icons for various platforms

### Plugin Configuration

- **manifest.json**: Plugin metadata (name, version, permissions)
- **Cargo.toml**: Plugin build configuration

## Documentation Structure

### docs/ Directory

All documentation in `docs/` with consistent naming:
- `UPPER_SNAKE_CASE.md` for major documents
- Linked from README.md
- Cross-referenced with relative links

### Inline Documentation

- **Rust**: `///` for public APIs, `//!` for module docs
- **TypeScript**: JSDoc comments for components and functions
- **SQL**: Comments in migration files

### Examples

- **plugins/examples/**: Example plugins
- **tests/fixtures/**: Test data examples
- **docs/**: Usage examples in documentation

## Testing Structure

### Unit Tests

Located in each crate's `src/` directory:
- `tests/` module in each file
- Integration with `cargo test`

### Integration Tests

Located in `tests/integration/`:
- Test interactions between crates
- Test database operations
- Test IPC communication

### E2E Tests

Located in `tests/e2e/`:
- Playwright tests for desktop UI
- CLI integration tests
- Full workflow tests

### Test Fixtures

Located in `tests/fixtures/`:
- Sample data for tests
- Mock configurations
- Test assets

## Asset Management

### Icons

Multiple sizes for different platforms:
- 16x16: Taskbar/dock
- 32x32: Standard icon
- 64x64: Large icon
- 128x128: macOS bundle
- 256x256: Windows installer
- 512x512: High-DPI displays

### Screenshots

For documentation and marketing:
- `search.png`: Search interface
- `settings.png`: Settings page
- `collections.png`: Collections view

### Logos

For website and documentation:
- `logo-horizontal.svg`: Horizontal layout
- `logo-vertical.svg`: Vertical layout

## CI/CD Structure

### GitHub Workflows

- **ci.yml**: Continuous integration (test on all platforms)
- **release.yml**: Release automation (build, sign, publish)
- **security.yml**: Security scanning (dependabot, codeql)

### Issue Templates

- **bug_report.md**: Bug report template
- **feature_request.md**: Feature request template
- **plugin_proposal.md**: Plugin proposal template

### PR Template

Standard PR description template for consistency.

## Contributing to Structure

When adding new components:

1. **New Crate**: Add to `Cargo.toml` workspace members
2. **New App**: Add to `apps/` directory
3. **New Plugin**: Add to `plugins/examples/` or `plugins/registry/`
4. **New Documentation**: Add to `docs/` with appropriate phase
5. **New Script**: Add to `scripts/` with executable permission
6. **New Test**: Add to appropriate `tests/` subdirectory

Follow existing naming conventions and structure patterns.
