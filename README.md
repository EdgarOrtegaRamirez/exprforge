# ExprForge

A safe mathematical expression evaluator with unit awareness, written in Rust.

ExprForge parses and evaluates mathematical expressions with built-in dimensional analysis and unit conversion. It features a hand-written lexer, recursive descent parser with Pratt-style precedence, and a tree-walking evaluator — no external parsing libraries, no `eval()`, no code injection surface.

## Features

- **Safe evaluation** — Hand-written lexer and parser, no `eval()` or code injection
- **40+ built-in functions** — `sqrt`, `abs`, `sin`, `cos`, `tan`, `log`, `ln`, `fact`, `gcd`, `lcm`, `min`, `max`, `mean`, `median`, `std`, `clamp`, `lerp`, `hypot`, and more
- **10+ constants** — `pi`, `e`, `tau`, `phi`, `sqrt2`, `sqrt3`, `ln2`, `ln10`, `euler`, `inf`, `nan`
- **100+ unit conversions** across 14 categories — length, mass, time, temperature, angle, area, volume, data, energy, force, frequency, power, pressure, velocity
- **Dimensional analysis** — detects incompatible unit operations (`5 m + 3 kg` is an error)
- **Unit conversion syntax** — `100 cm -> m`, `1 km -> mi`, `0 C -> F`
- **Variable assignment** — `x = 42`, `y = x * 2`
- **Boolean expressions** — comparisons (`<`, `>`, `==`, `!=`, `<=`, `>=`), logical (`&&`, `||`, `!`)
- **RPN conversion** — convert infix expressions to Reverse Polish Notation
- **Interactive REPL** — `exprforge repl`
- **Batch processing** — evaluate expressions from a file
- **JSON output** — `--json` flag for programmatic use

## Quick Start

### Install

```bash
cargo install --path .
```

### CLI Usage

```bash
# Evaluate an expression
exprforge eval "2 + 3 * 4"
# Output: 14

# Unit arithmetic
exprforge eval "5 m + 3 ft"
# Output: 5.9144 m

# Unit conversion
exprforge eval "100 cm -> m"
# Output: 1

# Temperature conversion
exprforge eval "0 C -> F"
# Output: 32

# Functions and constants
exprforge eval "sin(pi/2)"
# Output: 1

# Factorial
exprforge eval "fact(5)"
# Output: 120

# Boolean expressions
exprforge eval "3 > 2 && 5 == 5"
# Output: true

# Variable assignment
exprforge eval "x = 42"
exprforge eval "x * 2"
# Output: 84

# JSON output
exprforge eval "sqrt(16)" --json
# Output: {"type":"number","value":4}

# View AST
exprforge ast "2 + 3 * 4"
# Output: (2 + (3 * 4))

# View RPN
exprforge rpn "2 + 3 * 4"
# Output: 2 3 4 * +

# List available units
exprforge units

# Convert directly
exprforge convert 100 m ft
# Output: 100 m = 328.0839895013 ft

# Interactive REPL
exprforge repl

# Batch processing
exprforge batch expressions.txt
```

### Library Usage

```rust
use exprforge::{evaluate, evaluate_with_env, Environment, Value};

// Simple evaluation
let result = evaluate("2 + 3 * 4").unwrap();
assert_eq!(result.as_number(), 14.0);

// Unit conversion
let result = evaluate("100 cm -> m").unwrap();
assert_eq!(result.as_number(), 1.0);

// Variables
let mut env = Environment::new();
evaluate_with_env("x = 42", &mut env).unwrap();
let result = evaluate_with_env("x * 2", &mut env).unwrap();
assert_eq!(result.as_number(), 84.0);
```

## Architecture

ExprForge is built from the following modules:

| Module | Description |
|--------|-------------|
| `lexer` | Tokenizer — converts strings to tokens |
| `parser` | Recursive descent parser with Pratt-style precedence |
| `ast` | Abstract syntax tree types (`Expr`, `Value`, `BinOp`, `UnaryOp`) |
| `evaluator` | Tree-walking evaluator with unit propagation |
| `units` | Unit registry with 100+ conversions across 14 categories |
| `functions` | Built-in function registry (40+ functions, 10+ constants) |
| `environment` | Variable storage for assignment and lookup |
| `rpn` | Shunting-yard algorithm for RPN conversion |
| `repl` | Interactive REPL with line editing |
| `cli` | Command-line interface definitions |

### Operator Precedence (lowest to highest)

1. Assignment (`=`)
2. Unit conversion (`->`)
3. Logical OR (`||`)
4. Logical AND (`&&`)
5. Comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`)
6. Addition / Subtraction (`+`, `-`)
7. Multiplication / Division / Modulo (`*`, `/`, `%`)
8. Unary (`-`, `!`)
9. Exponentiation (`^`, right-associative)
10. Function calls, unit annotations, parentheses

## Security

- **No `eval()`** — All expressions are parsed and evaluated by hand-written code
- **No code injection** — The evaluator cannot execute arbitrary code
- **Input validation** — All inputs are validated during lexing and parsing
- **Safe arithmetic** — Division by zero returns an error, not `inf` or `NaN`
- **Dimensional safety** — Incompatible unit operations are detected and rejected

See [SECURITY.md](SECURITY.md) for more details.

## Testing

```bash
cargo test
```

The test suite includes:
- 89 unit tests (lexer, parser, evaluator, units, functions, RPN, environment, AST)
- 20 integration tests (end-to-end expression evaluation)
- 1 doc test

## License

MIT — See [LICENSE](LICENSE)
