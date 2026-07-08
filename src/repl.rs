//! Interactive REPL (Read-Eval-Print Loop) for ExprForge.
//!
//! Provides an interactive calculator session where users can define
//! variables, evaluate expressions, and convert units.

use crate::ast::Value;
use crate::environment::Environment;
use crate::evaluator::Evaluator;
use std::io::{self, BufRead, Write};

/// Run the interactive REPL.
pub fn run() {
    let evaluator = Evaluator::new();
    let mut env = Environment::new();

    println!("ExprForge REPL — Interactive Expression Evaluator");
    println!("Type 'help' for commands, 'quit' or 'exit' to exit.");
    println!();

    let stdin = io::stdin();
    let stdout = io::stdout();

    loop {
        // Print prompt
        print!("> ");
        if stdout.lock().flush().is_err() {
            break;
        }

        // Read line
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(_) => break,
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle REPL commands
        match line {
            "quit" | "exit" => break,
            "help" => {
                print_help();
                continue;
            }
            "vars" => {
                print_variables(&env);
                continue;
            }
            "functions" => {
                print_functions(&evaluator);
                continue;
            }
            "constants" => {
                print_constants(&evaluator);
                continue;
            }
            "units" => {
                print_units(&evaluator);
                continue;
            }
            "clear" => {
                env.clear();
                println!("Variables cleared.");
                continue;
            }
            _ => {}
        }

        // Evaluate the expression
        match crate::parser::parse(line) {
            Ok(expr) => match evaluator.eval(&expr, &mut env) {
                Ok(value) => println!("{}", format_value(&value)),
                Err(e) => println!("Error: {}", e),
            },
            Err(e) => println!("Parse error: {}", e),
        }
    }

    println!("Goodbye!");
}

/// Run the REPL reading from a file (batch mode).
pub fn run_file(path: &str) -> Result<(), String> {
    let evaluator = Evaluator::new();
    let mut env = Environment::new();
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match crate::parser::parse(line) {
            Ok(expr) => match evaluator.eval(&expr, &mut env) {
                Ok(value) => println!("{}: {}", line, format_value(&value)),
                Err(e) => println!("Error on line {}: {}", line_num + 1, e),
            },
            Err(e) => println!("Parse error on line {}: {}", line_num + 1, e),
        }
    }

    Ok(())
}

fn format_value(value: &Value) -> String {
    match value {
        Value::Number(n) => {
            if n.is_nan() {
                "NaN".to_string()
            } else if n.is_infinite() {
                if *n > 0.0 {
                    "inf".to_string()
                } else {
                    "-inf".to_string()
                }
            } else if *n == n.trunc() && n.abs() < 1e15 {
                format!("{}", *n as i64)
            } else {
                let s = format!("{:.10}", n);
                let s = s.trim_end_matches('0');
                let s = s.trim_end_matches('.');
                s.to_string()
            }
        }
        Value::Boolean(b) => b.to_string(),
        Value::UnitValue(n, dim) => {
            let num = format_value(&Value::Number(*n));
            format!("{} {}", num, dim)
        }
    }
}

fn print_help() {
    println!("ExprForge REPL Commands:");
    println!("  help       - Show this help");
    println!("  vars       - List defined variables");
    println!("  functions  - List available functions");
    println!("  constants  - List available constants");
    println!("  units      - List available units");
    println!("  clear      - Clear all variables");
    println!("  quit/exit  - Exit the REPL");
    println!();
    println!("Examples:");
    println!("  > 2 + 3 * 4");
    println!("  > x = 10");
    println!("  > x ^ 2");
    println!("  > 5m + 3ft");
    println!("  > sin(pi / 2)");
    println!("  > max(1, 2, 3)");
}

fn print_variables(env: &Environment) {
    if env.is_empty() {
        println!("No variables defined.");
        return;
    }
    println!("Variables:");
    for name in env.names() {
        if let Some(value) = env.get(&name) {
            println!("  {} = {}", name, format_value(value));
        }
    }
}

fn print_functions(evaluator: &Evaluator) {
    println!("Available functions:");
    let names = evaluator.functions.function_names();
    for chunk in names.chunks(6) {
        let row: Vec<String> = chunk.iter().map(|n| format!("{:<12}", n)).collect();
        println!("  {}", row.join(""));
    }
}

fn print_constants(evaluator: &Evaluator) {
    println!("Available constants:");
    let names = evaluator.functions.constant_names();
    for name in names {
        if let Some(val) = evaluator.functions.get_constant(&name) {
            println!("  {} = {}", name, val);
        }
    }
}

fn print_units(evaluator: &Evaluator) {
    println!("Available unit categories:");
    for cat in evaluator.units.categories() {
        let units = evaluator.units.by_category(&cat);
        let symbols: Vec<String> = units.iter().map(|u| u.symbol.clone()).collect();
        println!("  {}: {}", cat, symbols.join(", "));
    }
}
