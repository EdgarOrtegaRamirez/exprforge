//! CLI command definitions using clap.

use clap::{Parser, Subcommand};

/// ExprForge: Safe mathematical expression evaluator with unit awareness.
#[derive(Parser, Debug)]
#[command(
    name = "exprforge",
    version,
    about = "Safe mathematical expression evaluator with unit awareness",
    long_about = "ExprForge is a CLI tool and library for evaluating mathematical expressions\n\
                  with built-in unit awareness and dimensional analysis. It supports\n\
                  variables, functions, constants, and over 100 unit conversions."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Evaluate an expression
    Eval {
        /// The expression to evaluate
        expression: String,

        /// Convert the result to a specific unit
        #[arg(long)]
        to: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Start interactive REPL mode
    Repl,

    /// Evaluate expressions from a file (one per line)
    Batch {
        /// File containing expressions (one per line)
        file: String,
    },

    /// Convert a value from one unit to another
    Convert {
        /// The value to convert (e.g., "100")
        value: String,

        /// Source unit (e.g., "C")
        from: String,

        /// Target unit (e.g., "F")
        to: String,
    },

    /// Show the AST for an expression
    Ast {
        /// The expression to parse
        expression: String,
    },

    /// Convert an expression to Reverse Polish Notation
    Rpn {
        /// The expression to convert
        expression: String,
    },

    /// List available units, optionally filtered by category
    Units {
        /// Category to filter by (e.g., "length", "mass")
        #[arg(long)]
        category: Option<String>,
    },

    /// List available functions
    Functions,

    /// List available constants
    Constants,

    /// Show the dimension of an expression
    Dimension {
        /// The expression to analyze
        expression: String,
    },
}
