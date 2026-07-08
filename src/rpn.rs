//! Convert infix expressions to Reverse Polish Notation (RPN).
//!
//! Uses the shunting yard algorithm to convert an AST to RPN, which can be
//! useful for stack-based evaluation or debugging.

use crate::ast::*;

/// Convert an expression to RPN token sequence.
/// Returns a vector of strings representing the RPN tokens.
pub fn to_rpn(expr: &Expr) -> Vec<String> {
    let mut output = Vec::new();
    to_rpn_recursive(expr, &mut output);
    output
}

fn to_rpn_recursive(expr: &Expr, output: &mut Vec<String>) {
    match expr {
        Expr::Number(n) => output.push(format!("{}", n)),
        Expr::Boolean(b) => output.push(b.to_string()),
        Expr::Variable(name) => output.push(name.clone()),
        Expr::UnitValue(n, unit) => output.push(format!("{}{}", n, unit)),
        Expr::Binary(left, op, right) => {
            to_rpn_recursive(left, output);
            to_rpn_recursive(right, output);
            output.push(op.to_string());
        }
        Expr::Unary(op, expr) => {
            to_rpn_recursive(expr, output);
            output.push(format!("{}u", op)); // 'u' suffix for unary
        }
        Expr::Call(name, args) => {
            for arg in args {
                to_rpn_recursive(arg, output);
            }
            output.push(format!("{}:{}", name, args.len()));
        }
        Expr::Assign(name, expr) => {
            to_rpn_recursive(expr, output);
            output.push(format!("{}=", name));
        }
        Expr::Convert(expr, unit) => {
            to_rpn_recursive(expr, output);
            output.push(format!("->{}", unit));
        }
    }
}

/// Evaluate an RPN token sequence.
pub fn eval_rpn(
    tokens: &[String],
    env: &mut crate::environment::Environment,
) -> Result<Value, EvalError> {
    let evaluator = crate::evaluator::Evaluator::new();
    let mut stack: Vec<Value> = Vec::new();

    for token in tokens {
        // Try to parse as a number
        if let Ok(n) = token.parse::<f64>() {
            stack.push(Value::Number(n));
            continue;
        }

        // Boolean
        match token.as_str() {
            "true" => {
                stack.push(Value::Boolean(true));
                continue;
            }
            "false" => {
                stack.push(Value::Boolean(false));
                continue;
            }
            _ => {}
        }

        // Check for binary operators
        let bin_ops = [
            ("+", BinOp::Add),
            ("-", BinOp::Sub),
            ("*", BinOp::Mul),
            ("/", BinOp::Div),
            ("%", BinOp::Mod),
            ("^", BinOp::Pow),
            ("==", BinOp::Eq),
            ("!=", BinOp::Ne),
            ("<", BinOp::Lt),
            (">", BinOp::Gt),
            ("<=", BinOp::Le),
            (">=", BinOp::Ge),
            ("&&", BinOp::And),
            ("||", BinOp::Or),
        ];

        if let Some((_, op)) = bin_ops.iter().find(|(s, _)| *s == token) {
            if stack.len() < 2 {
                return Err(EvalError::new(format!(
                    "Stack underflow for operator {}",
                    token
                )));
            }
            let right = stack.pop().unwrap();
            let left = stack.pop().unwrap();
            let result = eval_binary_rpn(op, left, right)?;
            stack.push(result);
            continue;
        }

        // Check for unary operators
        if token == "-u" {
            if let Some(val) = stack.pop() {
                stack.push(Value::Number(-val.as_number()));
                continue;
            }
            return Err(EvalError::new("Stack underflow for unary minus"));
        }
        if token == "!u" {
            if let Some(val) = stack.pop() {
                stack.push(Value::Boolean(val.as_number() == 0.0));
                continue;
            }
            return Err(EvalError::new("Stack underflow for unary not"));
        }

        // Check for function call (name:argcount)
        if let Some(colon_pos) = token.rfind(':') {
            let name = &token[..colon_pos];
            let arg_count: usize = token[colon_pos + 1..].parse().unwrap_or(0);

            if name.is_empty() {
                return Err(EvalError::new(format!("Invalid function token: {}", token)));
            }

            // Check for constant
            if arg_count == 0 {
                if let Some(c) = evaluator.functions.get_constant(name) {
                    stack.push(Value::Number(c));
                    continue;
                }
            }

            if stack.len() < arg_count {
                return Err(EvalError::new(format!(
                    "Stack underflow for function {} (need {} args, have {})",
                    name,
                    arg_count,
                    stack.len()
                )));
            }

            let mut args = Vec::new();
            for _ in 0..arg_count {
                args.push(stack.pop().unwrap().as_number());
            }
            args.reverse();

            if let Some(func) = evaluator.functions.get(name) {
                let result = func(&args)?;
                stack.push(Value::Number(result));
                continue;
            }
            return Err(EvalError::new(format!("Unknown function: {}", name)));
        }

        // Check for assignment
        if token.ends_with('=') && !token.is_empty() && token.len() > 1 {
            let name = &token[..token.len() - 1];
            if let Some(val) = stack.pop() {
                env.set(name, val.clone());
                stack.push(val);
                continue;
            }
            return Err(EvalError::new("Stack underflow for assignment"));
        }

        // Check for variable
        if let Some(val) = env.get(token) {
            stack.push(val.clone());
            continue;
        }

        // Check for constant
        if let Some(c) = evaluator.functions.get_constant(token) {
            stack.push(Value::Number(c));
            continue;
        }

        return Err(EvalError::new(format!("Unknown RPN token: {}", token)));
    }

    stack.pop().ok_or_else(|| EvalError::new("Empty RPN stack"))
}

fn eval_binary_rpn(op: &BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
    let evaluator = crate::evaluator::Evaluator::new();
    evaluator.eval_binary_pub(op, l, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_rpn() {
        let expr = crate::parser::parse("1 + 2").unwrap();
        let rpn = to_rpn(&expr);
        assert_eq!(rpn, vec!["1", "2", "+"]);
    }

    #[test]
    fn test_complex_rpn() {
        let expr = crate::parser::parse("3 * (4 + 5)").unwrap();
        let rpn = to_rpn(&expr);
        assert_eq!(rpn, vec!["3", "4", "5", "+", "*"]);
    }

    #[test]
    fn test_function_rpn() {
        let expr = crate::parser::parse("max(1, 2, 3)").unwrap();
        let rpn = to_rpn(&expr);
        assert_eq!(rpn, vec!["1", "2", "3", "max:3"]);
    }

    #[test]
    fn test_eval_rpn() {
        let expr = crate::parser::parse("1 + 2 * 3").unwrap();
        let rpn = to_rpn(&expr);
        let mut env = crate::environment::Environment::new();
        let result = eval_rpn(&rpn, &mut env).unwrap();
        assert_eq!(result, Value::Number(7.0));
    }

    #[test]
    fn test_eval_rpn_function() {
        let expr = crate::parser::parse("sqrt(16)").unwrap();
        let rpn = to_rpn(&expr);
        let mut env = crate::environment::Environment::new();
        let result = eval_rpn(&rpn, &mut env).unwrap();
        assert!((result.as_number() - 4.0).abs() < 1e-10);
    }
}
