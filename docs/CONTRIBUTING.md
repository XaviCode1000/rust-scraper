# Contributing to Rust Scraper

Thank you for considering contributing to rust-scraper! This document provides guidelines and instructions for contributing.

## 🏗️ Architecture Overview

This project uses **Clean Architecture** with four layers:

```
Domain (pure business logic)
    ↓
Application (use cases, orchestration)
    ↓
Infrastructure (technical implementations)
    ↓
Adapters (external integrations)
```

**Key Principle:** Dependencies point inward. Domain knows nothing about infrastructure or adapters.

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed architecture documentation.

## 🚀 Development Setup

```bash
# Clone repository
git clone https://github.com/XaviCode1000/rust-scraper.git
cd rust-scraper

# Install dependencies
cargo fetch

# Build in debug mode
cargo build

# Run tests
cargo test

# Run clippy (linting)
cargo clippy -- -D clippy::correctness

# Format code
cargo fmt
```

## 📁 Project Structure

```
src/
├── lib.rs                  # Library root, public API re-exports
├── main.rs                 # CLI entry point
├── config.rs               # Logging configuration
├── error.rs                # ScraperError enum (thiserror)
├── url_path.rs             # URL path handling
├── user_agent.rs           # User-Agent rotation
│
├── domain/                 # Domain layer (pure)
│   ├── mod.rs
│   ├── entities.rs         # ScrapedContent, DownloadedAsset
│   └── value_objects.rs    # ValidUrl
│
├── application/            # Application layer (use cases)
│   ├── mod.rs
│   ├── http_client.rs      # HTTP client with retry
│   └── scraper_service.rs  # Scraping orchestration
│
├── infrastructure/         # Infrastructure layer
│   ├── mod.rs
│   ├── http/
│   ├── scraper/
│   │   ├── readability.rs
│   │   ├── fallback.rs
│   │   └── asset_download.rs
│   ├── converter/
│   │   ├── html_to_markdown.rs
│   │   └── syntax_highlight.rs
│   └── output/
│       ├── file_saver.rs
│       └── frontmatter.rs
│
└── adapters/               # Adapters layer
    ├── mod.rs
    ├── detector/           # MIME type detection
    ├── extractor/          # URL extraction
    └── downloader/         # Asset downloading
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_validate_url

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration

# Run doctests
cargo test --doc

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Writing Tests

**Unit tests** go in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Arrange
        // Act
        // Assert
    }
}
```

**Integration tests** go in `tests/` directory.

**Doctests** go in documentation comments:

```rust
/// # Examples
///
/// ```
/// use rust_scraper::example;
///
/// let result = example();
/// assert!(result.is_ok());
/// ```
```

## 📝 Code Style

This project uses standard Rust formatting:

```bash
# Format code (required before commit)
cargo fmt

# Check for clippy warnings
cargo clippy -- -D clippy::correctness
```

### Rust Skills Rules

This project follows the [rust-skills](rust-skills/INDEX.md) guidelines (179 rules). Key rules:

| Category | Rules |
|----------|-------|
| **Error Handling** | `err-thiserror-lib`, `err-no-unwrap-prod`, `err-retry-strategy` |
| **Ownership** | `own-borrow-over-clone`, `own-arc-for-shared-state` |
| **Async** | `async-concurrency-limit`, `async-no-lock-across-await` |
| **API Design** | `api-builder-pattern`, `api-fallible-operations` |
| **Testing** | `test-unit-isolation`, `test-async-with-tokio` |

### Code Review Checklist

Before submitting a PR, ensure:

- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy -- -D clippy::correctness`)
- [ ] All tests pass (`cargo test --all`)
- [ ] New code has tests
- [ ] No `unwrap()` in production code (use `?` or `expect()` with message)
- [ ] Error types are well-defined
- [ ] Documentation is updated

## 🎯 Adding Features

### New Output Format

1. Add variant to `OutputFormat` enum in `lib.rs`
2. Implement saving logic in `infrastructure/output/file_saver.rs`
3. Add tests

### New URL Handling

1. Add methods to `domain/value_objects.rs` (`ValidUrl`)
2. Ensure type safety is maintained
3. Add tests

### New Dependencies

1. Update `Cargo.toml`
2. Document why the dependency is needed
3. Update architecture documentation if layer changes

### New Error Type

1. Add variant to `ScraperError` enum in `error.rs`
2. Add `From` trait if automatic conversion is needed
3. Update documentation

### Asset Download Enhancement

1. Modify `adapters/detector/mime.rs` for new MIME types
2. Update `infrastructure/scraper/asset_download.rs` for download logic
3. Add tests for new asset types

## 🔄 Pull Request Process

1. **Fork** the repository
2. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/amazing-feature
   ```
3. **Make changes** with tests
4. **Ensure tests pass**:
   ```bash
   cargo test --all
   cargo clippy -- -D clippy::correctness
   cargo fmt
   ```
5. **Commit** with conventional commits (see below)
6. **Push** to your fork
7. **Create PR** with description of changes

## 📝 Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/):

| Type | Description |
|------|-------------|
| `feat:` | New feature |
| `fix:` | Bug fix |
| `refactor:` | Code refactoring (no behavior change) |
| `docs:` | Documentation only |
| `test:` | Adding or updating tests |
| `chore:` | Maintenance tasks |
| `perf:` | Performance improvement |
| `style:` | Code style changes (formatting, etc.) |

### Examples

```bash
# New feature
git commit -m "feat: add syntax highlighting for code blocks"

# Bug fix
git commit -m "fix: handle invalid URLs gracefully"

# Refactoring
git commit -m "refactor: extract HTML conversion to separate module"

# Documentation
git commit -m "docs: update ARCHITECTURE.md with layer diagrams"

# Tests
git commit -m "test: add integration tests for asset download"
```

### Breaking Changes

For breaking changes, add `BREAKING CHANGE:` in commit body:

```bash
git commit -m "refactor: migrate to Clean Architecture

Major refactoring from monolithic structure to layered architecture.

BREAKING CHANGE: Migrated from anyhow::Result to ScraperError::Result
```

## 🐛 Reporting Bugs

Open an issue with:

- **Description:** Clear description of the bug
- **Steps to Reproduce:** How to trigger the bug
- **Expected Behavior:** What should happen
- **Actual Behavior:** What actually happens
- **Environment:** Rust version, OS, etc.
- **Logs:** Relevant error messages

### Bug Report Template

```markdown
## Description
[Clear description]

## Steps to Reproduce
1. [First step]
2. [Second step]
3. [and so on...]

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happens]

## Environment
- Rust version: [e.g., 1.75.0]
- OS: [e.g., Ubuntu 22.04]
- Version: [e.g., 0.3.0]

## Logs
```
[Error messages or stack traces]
```
```

## 💡 Feature Requests

Open an issue with:

- **Problem:** What problem does this solve?
- **Proposal:** How should it work?
- **Alternatives:** What other solutions exist?
- **Use Cases:** Who will use this and how?

## ❓ Questions

For general questions:

1. Check existing [documentation](../docs/)
2. Search existing issues
3. Open a new issue with the "question" label

## 🎯 Areas Needing Contribution

- [ ] More unit tests for infrastructure layer
- [ ] Benchmark tests for performance-critical paths
- [ ] Documentation examples
- [ ] CI/CD improvements
- [ ] Performance optimizations

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [rust-skills](rust-skills/INDEX.md) - 179 Rust best practices

## 🙏 Thank You

Every contribution, no matter how small, is appreciated!

---

**Questions?** Open an issue or reach out to the maintainers.
