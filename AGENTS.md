# Agent Guidelines for Terrent

## Build/Test/Lint Commands
- Build: `cargo build` or `cargo build --release`
- Run: `cargo run`
- Test: `cargo test`
- Test single: `cargo test test_name`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style
- Edition: Rust 2024
- Imports: Group std, external crates, then local modules; alphabetically sorted within groups
- Formatting: Default rustfmt style (4-space indentation, 100-char line width)
- Types: Use explicit types for public APIs; prefer `pub struct` fields where appropriate
- Naming: snake_case for functions/variables, PascalCase for types/enums, SCREAMING_SNAKE_CASE for constants
- Error handling: Use `anyhow::Result` for error propagation; `unwrap()` acceptable in `main.rs` and interface code
- Modules: Use `mod.rs` for module organization; re-export public items
- Derives: Order as `Debug, Clone, PartialEq, Eq, PartialOrd, Ord` when applicable
- Architecture: Elm-like pattern for UI (Model-View-Update); separate concerns into modules
