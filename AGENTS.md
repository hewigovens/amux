# Agent Guidelines for amux

## Build/Test Commands
- `cargo build` - compile in debug mode; add `--release` for production
- `cargo test` - run all tests
- `cargo test test_name` - run single test (e.g., `cargo test ensure_valid_identifier_accepts_expected_chars`)
- `cargo fmt -- --check` - check formatting
- `cargo clippy --all-targets --all-features -- -D warnings` - lint with warnings as errors

## Code Style Guidelines
- **Rust 2021 edition** with 4-space indentation (rustfmt default)
- **Naming**: snake_case for functions/variables/modules, PascalCase for types/enums
- **Imports**: Group std, then external crates, then local modules; sort alphabetically
- **Error handling**: Use custom `Result<T>` type with `?` operator; prefer `bail!()` for early returns
- **Types**: Use `BTreeMap` for ordered collections; derive `Debug` for structs
- **Functions**: Keep small and focused; prefer helper functions for complex logic
- **Tests**: Use `#[cfg(test)]` blocks; descriptive names like `status_lists_sessions`
