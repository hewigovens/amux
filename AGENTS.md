# Repository Guidelines

## Project Structure & Module Organization
The CLI entry point lives in `src/main.rs`, which wires Clap parsing into library routines. Core behavior is grouped under `src/lib.rs` with supporting modules: `agents.rs` (agent registry and defaults), `cli.rs` (flag parsing), `tmux.rs` (session orchestration), and `error.rs` (shared error types). Cargo build artifacts land in `target/` and should stay untracked; user-facing docs and licensing remain at the repository root.

## Build, Test, and Development Commands
- `cargo build`: compile the binary in debug mode; add `--release` before publishing or benchmarking.
- `cargo run -- status`: run the CLI locally; swap `status` for flows like `start codex -n review-123` while iterating.
- `cargo fmt`: format the workspace; run before committing to avoid churn.
- `cargo clippy -- -D warnings`: lint and treat warnings as errors so regressions fail fast.

## Coding Style & Naming Conventions
Follow Rust 2021 idioms with rustfmt’s default four-space indentation. Use snake_case for files, modules, and functions, reserving PascalCase for types and Clap enums. Prefer small helper functions for tmux command composition, and surface additive configuration through environment variables such as `CA_AGENT_CMD_<NAME>` or explicit CLI flags.

## Testing Guidelines
Add unit tests alongside modules via `#[cfg(test)]` blocks and run everything with `cargo test`. Introduce integration coverage in a `tests/` directory when validating multi-session flows or tmux interactions. Name tests descriptively (`status_lists_sessions`) and keep fixture setup reusable with helpers in `src/tmux.rs`.

## Commit & Pull Request Guidelines
Write commits in the imperative mood (`Improve tmux error handling`) and keep subject lines under ~70 characters, mirroring current history. Squash fixup commits before opening a pull request, and mention related issues or agent tickets in the description. PRs should note the change scope, confirm `cargo fmt`, `cargo clippy`, and `cargo test` have been run, and attach terminal captures whenever CLI output changes.

## Security & Configuration Tips
tmux commands execute directly on the host, so avoid baking secrets into agent definitions; prefer environment variables or credential managers. Document new agent commands by recording their `CA_AGENT_CMD_...` names and default parameters in `README.md`. When shelling out, continue parsing user-provided arguments with `shell-words` to guard against injection.
