# OpenPaste Testing Strategy

## Overview

OpenPaste uses a comprehensive testing strategy covering unit tests, integration tests, end-to-end tests, and performance tests. This document outlines the testing approach, tools, and guidelines for ensuring code quality.

## Testing Philosophy

**Principles:**
- Test early and often
- Write tests alongside code
- Aim for high coverage on critical paths
- Test behavior, not implementation
- Keep tests fast and reliable
- Use tests as documentation

## Test Structure

### Directory Structure

```
crates/
  clipboard-core/
    src/
    tests/           # Integration tests
  clipboard-db/
    src/
    tests/
  clipboard-search/
    src/
    tests/
apps/
  desktop/
    src/
    __tests__/       # E2E tests (Playwright)
    src/__tests__/   # Unit tests (Jest)
```

## Unit Tests

### Rust Unit Tests

**Location:** In each module's `src/` directory

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        let data = b"Hello, World!";
        let content_type = detect_content_type(data);
        assert_eq!(content_type, ContentType::Text);
    }

    #[test]
    fn test_compression() {
        let text = "Hello, World!";
        let compressed = compress_text(text).unwrap();
        let decompressed = decompress_text(&compressed).unwrap();
        assert_eq!(decompressed, text);
    }

    #[test]
    #[should_panic(expected = "empty input")]
    fn test_empty_input_panics() {
        compress_text("").unwrap();
    }
}
```

**Running Unit Tests:**
```bash
# Run all unit tests
cargo test

# Run specific crate tests
cargo test -p clipboard-core

# Run specific test
cargo test test_content_type_detection

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### TypeScript Unit Tests

**Location:** `apps/desktop/src/__tests__/`

**Example:**
```typescript
import { describe, it, expect } from 'vitest';
import { detectContentType } from '../utils/content';

describe('detectContentType', () => {
  it('should detect text content', () => {
    const result = detectContentType('Hello, World!');
    expect(result).toBe('text');
  });

  it('should detect image content', () => {
    const result = detectContentType(new Uint8Array([0x89, 0x50, 0x4E, 0x47]));
    expect(result).toBe('image');
  });

  it('should handle empty input', () => {
    expect(() => detectContentType('')).toThrow('empty input');
  });
});
```

**Running Unit Tests:**
```bash
cd apps/desktop
npm test

# Run specific test
npm test -- detectContentType

# Run with coverage
npm run test:coverage
```

## Integration Tests

### Rust Integration Tests

**Location:** `crates/*/tests/` directory

**Example:**
```rust
use clipboard_core::ClipboardManager;
use clipboard_db::Database;

#[tokio::test]
async fn test_clipboard_capture_and_store() {
    // Setup test database
    let db = Database::in_memory().await.unwrap();
    
    // Setup clipboard manager
    let manager = ClipboardManager::new(db.clone());
    
    // Simulate clipboard capture
    let item = manager.capture_clipboard().await.unwrap();
    
    // Verify item was stored
    let retrieved = db.get_item(item.id).await.unwrap();
    assert_eq!(retrieved.id, item.id);
    assert_eq!(retrieved.content, item.content);
}
```

**Running Integration Tests:**
```bash
# Run all integration tests
cargo test --test '*'

# Run specific integration test
cargo test --test clipboard_integration
```

### API Integration Tests

**Example:**
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::test]
async fn test_search_endpoint() {
    let client = Client::new();
    let response = client
        .post("http://localhost:7890/api/v1/search")
        .json(json!({"query": "hello"}))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
}
```

## End-to-End Tests

### Playwright E2E Tests

**Location:** `apps/desktop/e2e/`

**Example:**
```typescript
import { test, expect } from '@playwright/test';

test.describe('Clipboard Search', () => {
  test('search and copy item', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Type search query
    await page.fill('[data-testid="search-input"]', 'hello');
    
    // Wait for results
    await page.waitForSelector('[data-testid="result-0"]');
    
    // Click first result
    await page.click('[data-testid="result-0"]');
    
    // Click copy button
    await page.click('[data-testid="copy-button"]');
    
    // Verify success notification
    await expect(page.locator('[data-testid="notification"]')).toBeVisible();
  });

  test('keyboard navigation', async ({ page }) => {
    await page.goto('http://localhost:3000');
    
    // Focus search
    await page.keyboard.press('Meta+K');
    
    // Type query
    await page.keyboard.type('hello');
    
    // Navigate down
    await page.keyboard.press('ArrowDown');
    
    // Copy item
    await page.keyboard.press('Enter');
    
    // Verify copied
    await expect(page.locator('[data-testid="notification"]')).toBeVisible();
  });
});
```

**Running E2E Tests:**
```bash
cd apps/desktop
npm run test:e2e

# Run specific test
npx playwright test search.spec.ts

# Run with UI
npx playwright test --ui

# Run headed
npx playwright test --headed
```

### E2E Test Scenarios

**Core Scenarios:**
- Clipboard capture and storage
- Search and retrieve items
- Pin/favorite items
- Delete items
- Create and manage collections
- Tag management
- Encryption lock/unlock
- Settings changes

**Edge Cases:**
- Large clipboard items
- Special characters
- Unicode content
- Empty clipboard
- Database errors
- Network errors

## Performance Tests

### Rust Performance Tests

**Criterion Benchmarks:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_search(c: &mut Criterion) {
    let db = setup_test_db();
    
    c.bench_function("search_1000_items", |b| {
        b.iter(|| {
            search_items(black_box(&db), black_box("hello"))
        });
    });
}

criterion_group!(benches, bench_search);
criterion_main!(benches);
```

**Running Benchmarks:**
```bash
cargo bench

# Run specific benchmark
cargo bench search
```

### Load Tests

**k6 Example:**
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '1m', target: 100 },
    { duration: '30s', target: 0 },
  ],
};

export default function () {
  let response = http.post('http://localhost:7890/api/v1/search', {
    query: 'hello',
  });
  
  check(response, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });
  
  sleep(1);
}
```

**Running Load Tests:**
```bash
k6 run load-test.js
```

## Test Coverage

### Coverage Goals

**Rust:**
- Core logic: 90%+
- Platform code: 70%+
- Overall: 80%+

**TypeScript:**
- Core logic: 85%+
- UI components: 70%+
- Overall: 75%+

### Measuring Coverage

**Rust (tarpaulin):**
```bash
cargo install tarpaulin
cargo tarpaulin --out Html
```

**TypeScript (vitest):**
```bash
npm run test:coverage
```

### Coverage Reports

**HTML Report:** Generated in `target/tarpaulin/` or `coverage/`

**CI Integration:** Upload coverage to Codecov or Coveralls

## Test Data

### Fixtures

**Rust Fixtures:**
```rust
#[cfg(test)]
mod fixtures {
    use super::*;

    pub fn test_clipboard_item() -> ClipboardItem {
        ClipboardItem {
            id: 1,
            content_type: ContentType::Text,
            content: "Hello, World!".to_string(),
            hash: "abc123".to_string(),
            created_at: 1704067200000,
            ..Default::default()
        }
    }
}
```

**TypeScript Fixtures:**
```typescript
export const testClipboardItem: ClipboardItem = {
  id: 1,
  contentType: 'text',
  content: 'Hello, World!',
  hash: 'abc123',
  createdAt: 1704067200000,
};
```

### Test Database

**In-Memory Database:**
```rust
async fn setup_test_db() -> Database {
    Database::in_memory().await.unwrap()
}
```

**Seeded Database:**
```rust
async fn setup_seeded_db() -> Database {
    let db = Database::in_memory().await.unwrap();
    
    // Add test data
    for i in 0..100 {
        db.insert_item(test_item(i)).await.unwrap();
    }
    
    db
}
```

## Mocking

### Rust Mocking

**Mock Traits:**
```rust
#[cfg(test)]
mock! {
    pub ClipboardProvider {
        fn get_content(&self) -> Result<ClipboardContent, PlatformError>;
        fn set_content(&self, content: &ClipboardContent) -> Result<(), PlatformError>;
    }
}

#[test]
fn test_with_mock() {
    let mut mock = MockClipboardProvider::new();
    mock.expect_get_content()
        .returning(|| Ok(ClipboardContent::Text("test".to_string())));
    
    // Use mock in test
}
```

### TypeScript Mocking

**Vitest Mocks:**
```typescript
import { vi } from 'vitest';
import { getClipboardContent } from '../clipboard';

vi.mock('../clipboard', () => ({
  getClipboardContent: vi.fn(() => Promise.resolve('test content')),
}));

test('with mock', async () => {
  const content = await getClipboardContent();
  expect(content).toBe('test content');
});
```

## Test Utilities

### Rust Test Utilities

**Test Helpers:**
```rust
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub async fn wait_for_condition<F>(mut condition: F, timeout_ms: u64) -> bool
    where
        F: FnMut() -> bool,
    {
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(timeout_ms) {
            if condition() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        false
    }
}
```

### TypeScript Test Utilities

**Test Helpers:**
```typescript
export async function waitForCondition(
  condition: () => boolean,
  timeoutMs: number = 5000
): Promise<boolean> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (condition()) return true;
    await new Promise(resolve => setTimeout(resolve, 10));
  }
  return false;
}
```

## CI/CD Testing

### GitHub Actions

**Test Workflow:**
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
      
      - name: Install Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Run Rust tests
        run: cargo test --all
      
      - name: Run TypeScript tests
        run: |
          cd apps/desktop
          npm install
          npm test
      
      - name: Run E2E tests
        run: |
          cd apps/desktop
          npm run test:e2e
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### Pre-Commit Hooks

**Husky Configuration:**
```bash
npm install husky lint-staged
npx husky install
```

**lint-staged Configuration:**
```json
{
  "*.{rs,toml}": ["cargo fmt -- --check", "cargo clippy -- -D warnings"],
  "*.{ts,tsx}": ["eslint --fix", "prettier --write"],
  "*.{json,md}": ["prettier --write"]
}
```

## Test Guidelines

### Writing Good Tests

**DO:**
- Test one thing per test
- Use descriptive test names
- Arrange-Act-Assert pattern
- Test edge cases
- Keep tests independent
- Use fixtures for common data

**DON'T:**
- Test implementation details
- Write fragile tests
- Skip tests without reason
- Use sleeps for synchronization
- Test multiple things in one test

### Test Naming

**Rust:**
```rust
#[test]
fn test_get_item_returns_item_when_exists() {
    // ...
}

#[test]
fn test_get_item_returns_error_when_not_found() {
    // ...
}
```

**TypeScript:**
```typescript
it('returns item when exists', () => {
  // ...
});

it('returns error when not found', () => {
  // ...
});
```

### Test Organization

**Group Related Tests:**
```rust
#[cfg(test)]
mod clipboard_tests {
    mod capture_tests { /* ... */ }
    mod storage_tests { /* ... */ }
    mod search_tests { /* ... */ }
}
```

## Debugging Tests

### Rust Test Debugging

**Print Debug Output:**
```bash
cargo test -- --nocapture
```

**Debug Specific Test:**
```bash
cargo test test_name -- --nocapture
```

**Use Debugger:**
```bash
cargo test --no-run
rust-lldb target/debug/deps/test_name
```

### TypeScript Test Debugging

**Debug with VS Code:**
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "node",
      "request": "launch",
      "name": "Debug Vitest",
      "program": "${workspaceFolder}/node_modules/.bin/vitest",
      "args": ["run", "--inspect-brk", "--no-coverage"]
    }
  ]
}
```

## Flaky Tests

### Identifying Flaky Tests

**Symptoms:**
- Test passes locally, fails in CI
- Test passes sometimes, fails sometimes
- Test fails with timeout

### Fixing Flaky Tests

**Common Causes:**
- Race conditions
- Timing dependencies
- External dependencies
- Shared state

**Solutions:**
- Use proper synchronization
- Avoid sleeps, use waits
- Mock external dependencies
- Isolate test state

### Retrying Flaky Tests

**Vitest Retry:**
```typescript
test('flaky test', { retry: 3 }, () => {
  // ...
});
```

## Security Testing

### Dependency Scanning

**cargo-audit:**
```bash
cargo install cargo-audit
cargo audit
```

**npm audit:**
```bash
npm audit
```

### Static Analysis

**Rust:**
```bash
cargo clippy -- -D warnings
```

**TypeScript:**
```bash
npm run lint
```

## Accessibility Testing

### Axe DevTools

**Automated Testing:**
```typescript
import { test, expect } from '@playwright/test';

test('accessibility', async ({ page }) => {
  await page.goto('http://localhost:3000');
  
  const violations = await page axe();
  expect(violations).toHaveLength(0);
});
```

### Manual Testing

**Checklist:**
- Keyboard navigation
- Screen reader compatibility
- Color contrast
- Focus indicators
- ARIA labels

## Test Documentation

### Documenting Tests

**Test Comments:**
```rust
/// Tests that clipboard capture stores item in database
/// 
/// # Scenario
/// 1. Setup test database
/// 2. Capture clipboard content
/// 3. Verify item stored in database
#[test]
fn test_clipboard_capture_and_store() {
    // ...
}
```

### Test Documentation Files

**README in tests directory:**
```markdown
# Clipboard Core Tests

## Running Tests
```bash
cargo test -p clipboard-core
```

## Test Coverage
- Content type detection: 100%
- Compression: 95%
- Storage: 90%
```

## Continuous Improvement

### Reviewing Test Coverage

**Weekly:**
- Review coverage reports
- Identify untested critical paths
- Add tests for gaps

### Refactoring Tests

**When:**
- Tests become slow
- Tests are hard to understand
- Tests are fragile

**How:**
- Extract common patterns
- Improve test helpers
- Simplify fixtures

### Test Metrics

**Track:**
- Test execution time
- Test pass rate
- Coverage trends
- Flaky test rate

## Resources

### Rust Testing

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-nextest](https://nexte.st/)
- [mockall](https://docs.rs/mockall/)

### TypeScript Testing

- [Vitest](https://vitest.dev/)
- [Playwright](https://playwright.dev/)
- [Testing Library](https://testing-library.com/)

### Performance Testing

- [Criterion](https://bheisler.github.io/criterion.rs/book/)
- [k6](https://k6.io/)
