# OpenPaste Project

## Vision

OpenPaste is a fast, secure, cross-platform, extensible clipboard manager written in Rust. It aims to become the "VS Code of clipboard managers"—not just another utility, but a professional-grade platform for clipboard management that respects user privacy and enables powerful workflows.

## Motto

**Own your clipboard.**

## Long-Term Goals

### Core Principles

- **Completely Open Source**: MIT or Apache-2.0 licensed, with no proprietary components
- **Cross Platform**: First-class support for Windows, Linux (Wayland + X11), and macOS
- **Performance**: Sub-20ms search across thousands of clipboard items
- **Efficiency**: Low memory footprint suitable for always-running background service
- **Extensibility**: Rich plugin ecosystem via WebAssembly
- **Privacy First**: Local-first by default with optional encrypted sync
- **Developer Friendly**: Well-documented APIs, CLI, and plugin SDK

### User Experience Goals

- **Keyboard-First UX**: Full keyboard navigation and shortcuts
- **Instant Search**: Results appear as you type with intelligent ranking
- **Seamless Integration**: Works invisibly in the background
- **Rich Content Support**: Text, images, files, HTML, and custom formats
- **Organization**: Collections, tags, favorites, and smart filters
- **Automation**: CLI and REST API for scripting and integration

## Non-Goals

### What We Won't Build

- **Cloud-Only Service**: OpenPaste will always work offline; cloud sync is optional
- **Social Features**: No sharing, collaboration, or social networking
- **Mobile App (v1.0)**: Mobile support planned for v2.0
- **Browser Extension (v1.0)**: Planned for later versions
- **AI Processing (v1.0)**: Basic AI features planned for v1.0, advanced for v2.0
- **Enterprise Features**: No SSO, audit logs, or corporate compliance initially
- **Proprietary Formats**: All data stored in open, documented formats
- **Subscription Model**: One-time purchase or free, no recurring fees
- **Data Mining**: No analytics, telemetry, or user tracking

### Out of Scope (Forever)

- **Clipboard Monitoring of Other Apps**: We only monitor the system clipboard, not individual applications
- **Keylogging**: No keyboard logging beyond clipboard content
- **Screen Capture**: No screenshot or screen recording capabilities
- **Remote Access**: No remote control or administration features

## Design Philosophy

### Local-First

Your clipboard data stays on your machine by default. The daemon runs locally, stores data locally, and processes requests locally. Cloud sync is an opt-in feature that uses end-to-end encryption so even we can't read your data.

### Modular Architecture

Every major feature lives in its own Rust crate with clear interfaces:
- `clipboard-core`: Core clipboard operations
- `clipboard-db`: Database abstraction layer
- `clipboard-index`: Full-text search indexing
- `clipboard-search`: Query processing and ranking
- `clipboard-sync`: Synchronization protocols
- `clipboard-api`: REST API server
- `clipboard-cli`: Command-line interface
- `clipboard-plugin`: WebAssembly plugin runtime
- `clipboard-encryption`: Cryptographic operations
- `clipboard-platform`: Platform-specific clipboard access
- `clipboard-events`: Event bus and pub/sub
- `clipboard-ai`: AI-powered features
- `clipboard-utils`: Shared utilities

This modularity enables:
- Independent testing of components
- Clear dependency boundaries
- Easy addition of new features
- Reuse across different clients (desktop, CLI, future mobile)

### Secure by Default

- **Encryption**: All sensitive data encrypted at rest with AES-256-GCM
- **Key Derivation**: Master password derived using Argon2id
- **Least Privilege**: Plugins run in WASM sandbox with explicit permissions
- **Memory Protection**: Sensitive data zeroed from memory when no longer needed
- **No Telemetry**: No analytics or data collection by default
- **Audit Trail**: Optional logging of clipboard access events

### Cross-Platform First

No platform is an afterthought:
- Windows, Linux, and macOS tested equally
- Platform-specific code isolated in `clipboard-platform`
- Consistent behavior across platforms where possible
- Platform-specific features documented and tested

### Extensibility

Most functionality accessible through:
- **Plugin SDK**: WebAssembly plugins for custom logic
- **REST API**: HTTP endpoints for integration
- **CLI**: Command-line tool for scripting
- **IPC**: Inter-process communication for local clients

### Production Quality

- **Documentation**: Comprehensive docs for users and contributors
- **Testing**: Unit, integration, and UI tests with high coverage
- **CI/CD**: Automated testing on all platforms
- **Release Engineering**: Signed binaries, auto-updates, installers
- **Contributor Experience**: Clear contribution guidelines, responsive reviews

## Success Metrics

### Technical Metrics

- **Performance**: Search latency < 20ms for 10,000 items
- **Memory**: Daemon memory usage < 100MB with 1,000 items
- **Startup**: Daemon startup time < 500ms
- **Storage**: Database size < 50MB for 10,000 text items
- **Reliability**: 99.9% uptime for background daemon

### User Metrics

- **Adoption**: 10,000+ active users by v1.0
- **Retention**: 70% of users still active after 30 days
- **Satisfaction**: 4.5+ star rating on package managers
- **Plugin Ecosystem**: 50+ community plugins by v1.0

### Community Metrics

- **Contributors**: 50+ unique contributors by v1.0
- **Issues**: 90% of issues resolved within 7 days
- **PRs**: 80% of PRs reviewed within 3 days
- **Documentation**: All public APIs documented with examples

## Contribution Model

### Governance

OpenPaste is a community-driven project:
- **Benevolent Dictator**: Initial maintainer has final say
- **Core Team**: Trusted contributors with merge access
- **Contributors**: Community members submitting PRs
- **Users**: People using and providing feedback

### Decision Making

- **Design Decisions**: Discussed in issues, decided by maintainer
- **API Changes**: Require proposal and review period
- **Breaking Changes**: Only in major versions, with migration guide
- **Feature Requests**: Evaluated against project goals and resources

### Contribution Areas

We welcome contributions in:
- Core functionality (Rust)
- Desktop UI (React/TypeScript)
- Plugins (Rust/WASM, Go, AssemblyScript)
- Documentation
- Testing
- Bug fixes
- Performance improvements
- Platform-specific support

### Recognition

- Contributors listed in README
- Plugin authors featured in plugin registry
- Significant contributions acknowledged in release notes
- Core team status for sustained, high-quality contributions

## License

**License**: Apache-2.0 OR MIT

Dual-licensed to maximize compatibility:
- Apache-2.0: Patent protections, corporate-friendly
- MIT: Simple, permissive, widely used

Contributors retain copyright to their contributions, licensed under the project's license.

## Project Status

**Current Phase**: Design and Planning

**Next Milestone**: v0.1 MVP
- Core clipboard capture
- Basic desktop UI
- Local storage
- Simple search
- No encryption, no plugins, no sync

**Target Timeline**: 6 months to v0.1, 18 months to v1.0

## Related Projects

OpenPaste stands on the shoulders of giants:
- **Tauri**: Desktop application framework
- **SQLite**: Embedded database
- **Wasmtime**: WebAssembly runtime
- **Axum**: Web framework for REST API
- **RustCrypto**: Cryptographic primitives
- **Clipboard.rs**: Platform clipboard access inspiration

## Contact

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Discord**: (to be created)
- **Email**: (to be created)

## Acknowledgments

Thank you to:
- The Rust community for excellent tooling
- Tauri team for the desktop framework
- All clipboard manager projects that came before
- Future contributors and users
