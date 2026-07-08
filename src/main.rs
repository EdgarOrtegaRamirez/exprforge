//! ExprForge CLI entry point.

use clap::Parser;
use exprforge::cli::{Cli, Commands};
use exprforge::{evaluate, Environment, Evaluator};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // No subcommand: show help
        None => {
            Cli::parse_from(["exprforge", "--help"]);
        }

        Some(Commands::Eval {
            expression,
            to,
            json,
        }) => {
            cmd_eval(&expression, to.as_deref(), json);
        }

        Some(Commands::Repl) => {
            exprforge::repl::run();
        }

        Some(Commands::Batch { file }) => {
            if let Err(e) = exprforge::repl::run_file(&file) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        Some(Commands::Convert { value, from, to }) => {
            cmd_convert(&value, &from, &to);
        }

        Some(Commands::Ast { expression }) => {
            cmd_ast(&expression);
        }

        Some(Commands::Rpn { expression }) => {
            cmd_rpn(&expression);
        }

        Some(Commands::Units { category }) => {
            cmd_units(category.as_deref());
        }

        Some(Commands::Functions) => {
            cmd_functions();
        }

        Some(Commands::Constants) => {
            cmd_constants();
        }

        Some(Commands::Dimension { expression }) => {
            cmd_dimension(&expression);
        }
    }
}

fn cmd_eval(expression: &str, to: Option<&str>, json: bool) {
    match exprforge::parse(expression) {
        Ok(expr) => {
            let evaluator = Evaluator::new();
            let mut env = Environment::new();
            match evaluator.eval(&expr, &mut env) {
                Ok(value) => {
                    if let Some(unit) = to {
                        match evaluator.convert_to_unit(&value, unit) {
                            Ok(converted) => {
                                if json {
                                    print_json_value(&converted);
                                } else {
                                    println!("{}", format_value(&converted));
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else if json {
                        print_json_value(&value);
                    } else {
                        println!("{}", format_value(&value));
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_convert(value: &str, from: &str, to: &str) {
    let value: f64 = match value.parse() {
        Ok(v) => v,
        Err(_) => {
            // Try evaluating as an expression
            match evaluate(value) {
                Ok(v) => v.as_number(),
                Err(e) => {
                    eprintln!("Error parsing value: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    let evaluator = Evaluator::new();
    match evaluator.units.convert(value, from, to) {
        Ok(result) => {
            println!("{} {} = {} {}", value, from, format_number(result), to);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_ast(expression: &str) {
    match exprforge::parse(expression) {
        Ok(expr) => {
            println!("{}", expr);
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_rpn(expression: &str) {
    match exprforge::parse(expression) {
        Ok(expr) => {
            let rpn = exprforge::to_rpn(&expr);
            println!("{}", rpn.join(" "));
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_units(category: Option<&str>) {
    let evaluator = Evaluator::new();
    if let Some(cat) = category {
        let units = evaluator.units.by_category(cat);
        if units.is_empty() {
            eprintln!("No units found in category: {}", cat);
            std::process::exit(1);
        }
        println!("Units in category '{}':", cat);
        for u in units {
            println!("  {:<10} {:<25} factor={}", u.symbol, u.name, u.factor);
        }
    } else {
        println!("Unit categories:");
        for cat in evaluator.units.categories() {
            let units = evaluator.units.by_category(&cat);
            let symbols: Vec<String> = units.iter().map(|u| u.symbol.clone()).collect();
            println!("  {:<15} {}", cat, symbols.join(", "));
        }
    }
}

fn cmd_functions() {
    let evaluator = Evaluator::new();
    println!("Available functions:");
    let names = evaluator.functions.function_names();
    for chunk in names.chunks(5) {
        let row: Vec<String> = chunk.iter().map(|n| format!("{:<14}", n)).collect();
        println!("  {}", row.join(""));
    }
}

fn cmd_constants() {
    let evaluator = Evaluator::new();
    println!("Available constants:");
    for name in evaluator.functions.constant_names() {
        if let Some(val) = evaluator.functions.get_constant(&name) {
            println!("  {:<10} = {}", name, val);
        }
    }
}

fn cmd_dimension(expression: &str) {
    match exprforge::parse(expression) {
        Ok(expr) => {
            let evaluator = Evaluator::new();
            let mut env = Environment::new();
            match evaluator.eval(&expr, &mut env) {
                Ok(value) => {
                    let dim = value.dimension();
                    if dim.is_dimensionless() {
                        println!("dimensionless");
                    } else {
                        println!("{}", dim);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    }
}

fn format_value(value: &exprforge::Value) -> String {
    match value {
        exprforge::Value::Number(n) => format_number(*n),
        exprforge::Value::Boolean(b) => b.to_string(),
        exprforge::Value::UnitValue(n, dim) => {
            let num = format_number(*n);
            if dim.is_dimensionless() {
                num
            } else {
                format!("{} {}", num, dim)
            }
        }
    }
}

fn format_number(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        }
    } else if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.10}", n);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

fn print_json_value(value: &exprforge::Value) {
    match value {
        exprforge::Value::Number(n) => {
            println!(r#"{{"type":"number","value":{}}}"#, n);
        }
        exprforge::Value::Boolean(b) => {
            println!(r#"{{"type":"boolean","value":{}}}"#, b);
        }
        exprforge::Value::UnitValue(n, dim) => {
            println!(r#"{{"type":"unit","value":{},"dimension":"{}"}}"#, n, dim);
        }
    }
}
