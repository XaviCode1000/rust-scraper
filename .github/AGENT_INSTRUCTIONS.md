# 🧠 Agent Instructions for rust-scraper (2026 Edition)

You are a Senior Rust Engineer helping to maintain a **Clean Architecture** project. Follow these rules to avoid technical debt and architectural leaks.

## 🏗️ Architectural Constraints (Clean Architecture)
This project is divided into four main layers. **Strict dependency isolation is required.**

1. **Domain Layer (`src/domain/`):** Pure business logic. 
   - **NO** external crates for IO (e.g., `reqwest`, `sqlx`).
   - **NO** framework dependencies.
   - **MUST** define traits (interfaces) for repositories or external services.
2. **Application Layer (`src/application/`):** Orchestration (Use Cases).
   - Depends only on Domain.
   - Coordinates the flow of data.
3. **Infrastructure Layer (`src/infra/`):** Real-world implementations.
   - DB, HTTP clients, File system.
   - Implements Domain traits.
4. **Adapters/Presentation Layer (`src/presentation/`):** TUI (Ratatui), CLI, etc.
   - Entry points of the system.

## 🦀 Rust-Specific Best Practices (rust-skills v1.0.0)

### Ownership & Borrowing (CRITICAL)
- **Borrow over Clone:** Prefer `&[T]` over `&Vec<T>` and `&str` over `&String`. (violás `own-borrow-over-clone` si no).
- **Avoid `.clone()`:** If you need a clone to satisfy the borrow checker, explain **WHY**. In hot paths, it's prohibited.

### Error Handling (CRITICAL)
- **NO `unwrap()` or `expect()`:** Use `Result` and the `?` operator. Use `expect()` only in test setups or cases that are logically impossible (bugs).
- **Errors by Layer:** 
  - Use `thiserror` for Domain and Library errors.
  - Use `anyhow` for Application and CLI/Binaries level.

### Concurrency (HIGH)
- **NO Locks across `.await`:** Never hold a `MutexGuard` (std or tokio) across a `.await` point.
- **JoinSet:** Prefer `tokio::task::JoinSet` for managing multiple background tasks.

### API Design (HIGH)
- **Builder Pattern:** Use for complex struct initialization.
- **Newtypes:** Use to prevent primitive obsession and ensure type safety (e.g., `UserId(u64)`).

## 🧪 Testing Strategy
- Unit tests go in the same file as the code.
- Integration tests go in the `/tests` folder.
- Run `cargo test` before submitting any change.

## 🛠️ Tooling
- **CI Status:** Always check `.github/context_map.json` or the latest CI artifacts for failure logs.
- **Formatter:** Always run `cargo fmt` and `cargo clippy`.

---
*Follow these rules, or Jarvis will deactivate your token. Good luck.*
