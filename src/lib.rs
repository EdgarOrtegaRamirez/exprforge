//! ExprForge: Safe mathematical expression evaluator with unit awareness.
//!
//! A Rust library and CLI tool for evaluating mathematical expressions with
//! built-in dimensional analysis and unit conversion. Features include:
//!
//! - Hand-written lexer and recursive descent parser
//! - Safe expression evaluation (no code injection)
//! - 40+ built-in functions (math, trig, statistics, etc.)
//! - 10+ built-in constants (pi, e, tau, phi, etc.)
//! - 100+ unit conversions across 14 categories
//! - Dimensional analysis (detect incompatible unit operations)
//! - Variable assignment and lookup
//! - RPN (Reverse Polish Notation) conversion
//! - Interactive REPL mode
//! - Batch file processing
//!
//! # Example
//!
//! ```
//! use exprforge::evaluate;
//!
//! let result = evaluate("2 + 3 * 4").unwrap();
//! assert_eq!(result.as_number(), 14.0);
//!
//! let result = evaluate("5m + 3ft").unwrap();
//! assert!(result.is_unit_value());
//! ```

pub mod ast;
pub mod cli;
pub mod environment;
pub mod evaluator;
pub mod functions;
pub mod lexer;
pub mod parser;
pub mod repl;
pub mod rpn;
pub mod units;

// Re-export commonly used types
pub use ast::{BinOp, EvalError, Expr, ParseError, UnaryOp, Value};
pub use environment::Environment;
pub use evaluator::{evaluate, evaluate_with_env, Evaluator};
pub use functions::FunctionRegistry;
pub use lexer::{tokenize, LexError, SpannedToken, Token};
pub use parser::parse;
pub use rpn::{eval_rpn, to_rpn};
pub use units::{Dimension, UnitDef, UnitRegistry};
