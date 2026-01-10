---
trigger: model_decision
description: When working with rust
---

# Rust Best Practices & Guidelines (2025/2026)

This document outlines the standards for writing clean, safe, and performant Rust code. These guidelines align with the **Rust 2024 Edition** and modern community standards.

## 1. Project Setup & Tooling

### 1.1. The Toolchain
* **Use the Latest Stable:** Always pin your project to the latest stable version unless you have a specific reason to use Nightly.
* **Edition 2024:** Enable `edition = "2024"` in your `Cargo.toml` to use the latest features (like improved async closures and smarter return types).
* **Lockfiles:** Always commit `Cargo.lock` for applications to ensure consistent builds. (Libraries usually ignore it).

### 1.2. Linting & Formatting
* **Format on Save:** Enforce `cargo fmt` in your CI/CD pipeline and editor.
* **Clippy is Law:** Run `cargo clippy` regularly. Treat warnings as errors in CI.
    ```bash
    cargo clippy -- -D warnings
    ```
* **Strict Dependency Management:** avoid unused dependencies to keep build times fast.

---

## 2. Code Style & Idioms

### 2.1. Clarity Over Cleverness
* **Naming:**
    * Variables/Functions: `snake_case` (e.g., `user_login_count`)
    * Types/Traits: `PascalCase` (e.g., `UserRequest`)
    * Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_RETRIES`)
* **Variable Shadowing:** It is idiomatic to shadow variables when transforming data to avoid cluttering names.
    ```rust
    // Good
    let input = " 5 ";
    let input = input.trim();
    let input: i32 = input.parse().unwrap();
    ```

### 2.2. Type System Design
* **NewType Pattern:** Don't use primitives (like `i32` or `String`) for sensitive data. Wrap them to prevent logic errors.
    ```rust
    // Bad
    fn process_payment(amount: f64, user_id: i32) { ... }

    // Good
    struct UserId(i32);
    struct Amount(f64);
    fn process_payment(amount: Amount, user: UserId) { ... }
    ```
* **Make Illegal States Unrepresentable:** Use `enum` to represent distinct states rather than using boolean flags.

### 2.3. Error Handling
* **Avoid `.unwrap()`:** Never use `.unwrap()` or `.expect()` in production code unless you are writing a quick prototype or test.
* **Propagate Errors:** Use the `?` operator to pass errors up the chain.
* **Meaningful Errors:** Use crates like `thiserror` for libraries (to define custom errors) and `anyhow` for applications (to handle flexible error reporting).

---

## 3. Performance & Memory

### 3.1. Ownership & Borrowing
* **Prefer References:** Pass data by reference (`&str`, `&[T]`) instead of cloning (`String`, `Vec<T>`) whenever possible.
* **Watch Your `.clone()`:** Frequent cloning is a "code smell." If you are fighting the borrow checker, step back and rethink the architecture rather than just cloning to make it compile.

### 3.2. Async Rust (Tokio/Async-std)
* **Don't Block the Runtime:** Never run heavy computations or blocking I/O (like reading a huge file synchronously) inside an `async` function. This freezes the entire program.
    * *Solution:* Use `tokio::task::spawn_blocking` for heavy tasks.
* **Select Carefully:** When using `tokio::select!`, remember that if one branch wins, the other creates are "dropped" (cancelled). Ensure your operations are safe to cancel.

---

## 4. Safety & Security

### 4.1. Unsafe Code
* **Avoid `unsafe`:** 99% of Rust projects do not need `unsafe`. If you think you need it for performance, verify with benchmarks first.
* **Comment Safety:** If you MUST use `unsafe`, you must write a comment `// SAFETY: ...` explaining exactly why the operation is safe.

### 4.2. Input Validation
* **Parse, Don't Validate:** Validate incoming data immediately and convert it into a "smart type" (see 2.2). Once data is in a struct, the rest of the program can trust it.

---

## 5. Project Structure

### 5.1. File Organization
* **Avoid "mod.rs" clutter:** Use the modern file structure to keep your file tree clean.
    ```text
    // Old (avoid)
    src/
      auth/
        mod.rs

    // New (preferred)
    src/
      auth.rs
      auth/
        login.rs
    ```

### 5.2. Visibility
* **Private by Default:** Keep fields and functions private. Only make public (`pub`) what is absolutely necessary for the user of the module.
* **Crate Visibility:** Use `pub(crate)` for items shared within your project but hidden from the outside world.

---

## 6. Testing

* **Unit Tests:** Place unit tests in the same file as the code, inside a `mod tests` module.
* **Integration Tests:** Place high-level tests in the `tests/` directory at the project root.
* **Doc Tests:** Write examples in your documentation comments (`///`). `cargo test` runs these automatically to ensure your documentation never gets out of date.