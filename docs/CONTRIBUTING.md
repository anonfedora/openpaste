# Contributing to OpenPaste

Thank you for your interest in contributing to OpenPaste! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Issue Reporting](#issue-reporting)
- [Feature Requests](#feature-requests)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors. We value respect, kindness, and collaboration.

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or exclusion
- Personal attacks or insults
- Public or private harassment
- Publishing others' private information
- Other unethical or unprofessional conduct

### Reporting Issues

Report conduct issues to the project maintainers via email or private message.

## Getting Started

### Prerequisites

**Required:**
- Rust 1.75 or later
- Node.js 18 or later
- Git

**Platform-Specific:**
- Windows: Visual Studio Build Tools
- Linux: build-essential, libssl-dev
- macOS: Xcode Command Line Tools

### Setting Up Development Environment

**1. Fork and Clone:**
```bash
git clone https://github.com/your-username/openpaste.git
cd openpaste
```

**2. Install Rust Dependencies:**
```bash
cargo install cargo-watch
cargo install cargo-nextest
cargo install cargo-edit
```

**3. Install Node Dependencies:**
```bash
cd apps/desktop
npm install
```

**4. Build Project:**
```bash
# Build all Rust components
cargo build

# Build desktop app
cd apps/desktop
npm run tauri build
```

**5. Run Tests:**
```bash
cargo test
```

### Development Workflow

**1. Create Branch:**
```bash
git checkout -b feature/your-feature-name
```

**2. Make Changes:**
- Write code
- Add tests
- Update documentation

**3. Commit Changes:**
```bash
git add .
git commit -m "feat: add your feature"
```

**4. Push and Create PR:**
```bash
git push origin feature/your-feature-name
```

## Pull Request Process

### PR Guidelines

**Before Submitting:**
- [ ] Code follows project style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Commit messages follow conventions
- [ ] PR description clearly describes changes
- [ ] No merge conflicts

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How did you test this change?

## Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. **Automated Checks:** CI runs tests and linting
2. **Code Review:** Maintainers review code
3. **Feedback:** Address review comments
4. **Approval:** At least one maintainer approval
5. **Merge:** Maintainer merges PR

### Commit Message Conventions

**Format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Test changes
- `chore`: Build process or tooling changes
- `ci`: CI configuration changes

**Examples:**
```
feat(search): add fuzzy search support

Implement fuzzy search using the fuzzy-matcher crate.
Users can now enable fuzzy search in settings.

Closes #123
```

```
fix(encryption): handle vault lock timeout

Fixed issue where vault would not lock after timeout.
Added proper error handling and retry logic.

Fixes #456
```

## Coding Standards

### Rust Code Style

**Formatting:**
```bash
cargo fmt
```

**Linting:**
```bash
cargo clippy -- -D warnings
```

**Guidelines:**
- Use `cargo fmt` for formatting
- Fix all `clippy` warnings
- Follow Rust naming conventions
- Add doc comments for public APIs
- Keep functions focused and small

### TypeScript Code Style

**Formatting:**
```bash
cd apps/desktop
npm run format
```

**Linting:**
```bash
cd apps/desktop
npm run lint
```

**Guidelines:**
- Use Prettier for formatting
- Follow ESLint rules
- Use TypeScript strict mode
- Add JSDoc comments for functions
- Follow React best practices

### Documentation

**Code Comments:**
```rust
/// Gets clipboard item by ID
///
/// # Arguments
///
/// * `id` - The item ID
///
/// # Returns
///
/// * `Ok(ClipboardItem)` - The clipboard item
/// * `Err(StorageError)` - If item not found
pub fn get_item(id: i64) -> Result<ClipboardItem, StorageError> {
    // ...
}
```

**README Updates:**
- Update README for user-facing changes
- Update CHANGELOG for version changes
- Update relevant documentation files

## Testing

### Unit Tests

**Rust:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_item() {
        let item = get_item(123).unwrap();
        assert_eq!(item.id, 123);
    }
}
```

**TypeScript:**
```typescript
describe('getClipboardItem', () => {
  it('should return item by ID', () => {
    const item = getClipboardItem(123);
    expect(item.id).toBe(123);
  });
});
```

### Integration Tests

**Rust:**
```rust
#[tokio::test]
async fn test_clipboard_flow() {
    let db = setup_test_db().await;
    let item = create_test_item(&db).await;
    let retrieved = get_item(&db, item.id).await;
    assert_eq!(retrieved.id, item.id);
}
```

### E2E Tests

**Playwright:**
```typescript
test('search and copy item', async ({ page }) => {
  await page.goto('http://localhost:3000');
  await page.fill('[data-testid="search-input"]', 'hello');
  await page.click('[data-testid="result-0"]');
  await page.click('[data-testid="copy-button"]');
  // Verify clipboard content
});
```

### Test Coverage

**Target:** 80% coverage for core code

**Check Coverage:**
```bash
cargo tarpaulin --out Html
```

## Documentation

### API Documentation

**Rust:**
```bash
cargo doc --no-deps --open
```

**TypeScript:**
```typescript
/**
 * Gets clipboard item by ID
 * @param id - The item ID
 * @returns The clipboard item
 */
export function getClipboardItem(id: number): ClipboardItem {
  // ...
}
```

### User Documentation

**Update When:**
- Adding new features
- Changing behavior
- Fixing bugs
- Deprecating features

**Location:**
- README.md
- docs/ directory
- Inline code comments

## Issue Reporting

### Bug Reports

**Template:**
```markdown
## Description
Clear description of the bug

## Steps to Reproduce
1. Go to...
2. Click on...
3. Scroll down to...
4. See error

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- OS: [e.g. Windows 11]
- OpenPaste Version: [e.g. 0.1.0]
- Rust Version: [e.g. 1.75]

## Additional Context
Screenshots, logs, etc.
```

### Feature Requests

**Template:**
```markdown
## Problem Description
What problem would this feature solve?

## Proposed Solution
How should this feature work?

## Alternatives
What alternatives have you considered?

## Additional Context
Examples, mockups, etc.
```

## Development Tools

### Recommended Tools

**Rust:**
- VS Code with rust-analyzer
- IntelliJ IDEA with Rust plugin
- CLion

**TypeScript:**
- VS Code with ESLint and Prettier
- WebStorm

**Git:**
- GitHub CLI
- GitKraken
- SourceTree

### VS Code Extensions

**Rust:**
- rust-analyzer
- CodeLLDB
- Even Better TOML

**TypeScript:**
- ESLint
- Prettier
- TypeScript Vue Plugin (Volar)

**General:**
- GitLens
- GitHub Copilot (optional)
- Error Lens

## Performance Guidelines

### Performance Targets

- **Search:** < 20ms for 10,000 items
- **Clipboard Capture:** < 50ms
- **Startup:** < 500ms
- **Memory:** < 100MB with 1,000 items

### Profiling

**Rust:**
```bash
cargo install flamegraph
cargo flamegraph --bin openpaste-daemon
```

**TypeScript:**
- Chrome DevTools Profiler
- React DevTools Profiler

### Optimization

**Before Optimizing:**
1. Measure current performance
2. Identify bottleneck
3. Implement optimization
4. Measure improvement

## Security Guidelines

### Security Best Practices

- Never commit secrets or API keys
- Use environment variables for sensitive data
- Validate all user input
- Use prepared statements for database queries
- Keep dependencies updated

### Reporting Security Issues

**Do NOT:**
- Open public issue
- Discuss in public channels

**DO:**
- Email maintainers privately
- Use security@openpaste.org (when available)
- Provide detailed description and reproduction steps

## Release Process

### Version Bumping

**Semantic Versioning:**
- MAJOR: Breaking changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes (backward compatible)

### Release Checklist

- [ ] Update version in Cargo.toml
- [ ] Update version in package.json
- [ ] Update CHANGELOG.md
- [ ] Tag release
- [ ] Create GitHub release
- [ ] Build and publish packages

## Community

### Communication Channels

- **GitHub Issues:** Bug reports, feature requests
- **GitHub Discussions:** Questions, ideas
- **Discord:** Real-time chat (when available)
- **Email:** Private matters

### Getting Help

**Before Asking:**
- Search existing issues and discussions
- Read documentation
- Try to solve the problem yourself

**When Asking:**
- Provide context and details
- Share what you've tried
- Include error messages and logs

## Recognition

### Contributor Recognition

- Contributors listed in README
- Significant contributions acknowledged in release notes
- Core team status for sustained contributions

### Becoming a Maintainer

**Criteria:**
- Sustained, high-quality contributions
- Understanding of codebase
- Active participation in reviews
- Alignment with project goals

**Process:**
- Invitation from existing maintainers
- Consensus among maintainers
- Onboarding period

## License

By contributing, you agree that your contributions will be licensed under the project's license (MIT or Apache-2.0).

## Questions?

If you have questions about contributing, feel free to:
- Open a GitHub Discussion
- Ask in Discord (when available)
- Email maintainers

Thank you for contributing to OpenPaste!
