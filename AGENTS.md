# AGENTS.md

## Project: ExprForge

A safe mathematical expression evaluator with unit awareness in Rust.

## Build & Test Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets -- -D warnings

# Run the CLI
cargo run -- eval "2 + 3 * 4"
cargo run -- eval "100 cm -> m"
cargo run -- convert 100 m ft
cargo run -- units
cargo run -- repl
```

## Architecture

- `src/lexer.rs` — Tokenizer
- `src/parser.rs` — Recursive descent parser
- `src/ast.rs` — AST types (Expr, Value, BinOp, UnaryOp, errors)
- `src/evaluator.rs` — Tree-walking evaluator with unit propagation
- `src/units.rs` — Unit registry with dimensional analysis
- `src/functions.rs` — Built-in functions and constants
- `src/environment.rs` — Variable storage
- `src/rpn.rs` — RPN conversion (shunting yard)
- `src/repl.rs` — Interactive REPL
- `src/cli.rs` — CLI definitions (clap)
- `src/main.rs` — Entry point
- `src/lib.rs` — Library root
- `tests/integration.rs` — Integration tests

## Key Design Decisions

- Hand-written lexer and parser (no parser generators)
- Unary minus has lower precedence than `^` (mathematical convention: `-2^2 = -4`)
- Division by zero returns an error (not `inf`)
- Unit conversion via `->` operator: `100 cm -> m`
- The lexer produces `Identifier` tokens; the parser disambiguates units from identifiers based on context (number followed by identifier = unit annotation)
- Values are stored in base SI units internally; conversion happens on output

## Conventions

- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use `Result<T, E>` for fallible operations
- Document all public functions with `///` doc comments
- Write tests in the same file as the code (unit tests) and in `tests/` (integration tests)
- Run `cargo fmt --all` before committing
