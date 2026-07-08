//! Expression evaluator with unit awareness and dimensional analysis.
//!
//! Walks the AST and computes results. Handles:
//! - Arithmetic with automatic unit propagation
//! - Dimensional analysis (detecting incompatible unit operations)
//! - Function calls
//! - Variable lookup
//! - Comparison and logical operations

use crate::ast::*;
use crate::environment::Environment;
use crate::functions::FunctionRegistry;
use crate::units::UnitRegistry;

/// The evaluator computes the value of an expression.
pub struct Evaluator {
    pub functions: FunctionRegistry,
    pub units: UnitRegistry,
}

impl Evaluator {
    /// Create a new evaluator with default functions and units.
    pub fn new() -> Self {
        Evaluator {
            functions: FunctionRegistry::new(),
            units: UnitRegistry::new(),
        }
    }

    /// Evaluate an expression in the given environment.
    pub fn eval(&self, expr: &Expr, env: &mut Environment) -> Result<Value, EvalError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Variable(name) => {
                // Check constants first
                if let Some(c) = self.functions.get_constant(name) {
                    return Ok(Value::Number(c));
                }
                // Then check user variables
                env.get(name)
                    .cloned()
                    .ok_or_else(|| EvalError::new(format!("Undefined variable: {}", name)))
            }
            Expr::UnitValue(n, unit) => {
                let (base_value, dim) = self.units.to_base(*n, unit).map_err(EvalError::new)?;
                Ok(Value::UnitValue(base_value, dim))
            }
            Expr::Binary(left, op, right) => {
                let l = self.eval(left, env)?;
                let r = self.eval(right, env)?;
                self.eval_binary(op, l, r)
            }
            Expr::Unary(op, expr) => {
                let val = self.eval(expr, env)?;
                self.eval_unary(op, val)
            }
            Expr::Call(name, args) => {
                // Check if it's a function
                if let Some(func) = self.functions.get(name) {
                    let mut arg_values = Vec::new();
                    for arg in args {
                        let v = self.eval(arg, env)?;
                        // Functions take plain numbers; extract the numeric value
                        // but check for unit compatibility if needed
                        arg_values.push(v.as_number());
                    }
                    let result = func(&arg_values)?;
                    return Ok(Value::Number(result));
                }

                // Check if it's a constant called as a function (e.g., pi())
                if let Some(c) = self.functions.get_constant(name) {
                    if args.is_empty() {
                        return Ok(Value::Number(c));
                    }
                    return Err(EvalError::new(format!(
                        "Constant {} takes no arguments",
                        name
                    )));
                }

                Err(EvalError::new(format!("Unknown function: {}", name)))
            }
            Expr::Assign(name, expr) => {
                let value = self.eval(expr, env)?;
                env.set(name, value.clone());
                Ok(value)
            }
            Expr::Convert(expr, unit) => {
                let value = self.eval(expr, env)?;
                self.convert_to_unit(&value, unit)
            }
        }
    }

    /// Public wrapper for binary evaluation (used by RPN evaluator).
    pub fn eval_binary_pub(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
        self.eval_binary(op, l, r)
    }

    fn eval_binary(&self, op: &BinOp, l: Value, r: Value) -> Result<Value, EvalError> {
        match op {
            BinOp::Add | BinOp::Sub => {
                // For addition/subtraction, units must be compatible
                let l_dim = l.dimension();
                let r_dim = r.dimension();

                if l_dim.is_dimensionless() && r_dim.is_dimensionless() {
                    let result = match op {
                        BinOp::Add => l.as_number() + r.as_number(),
                        BinOp::Sub => l.as_number() - r.as_number(),
                        _ => unreachable!(),
                    };
                    Ok(Value::Number(result))
                } else if l_dim.is_compatible(&r_dim) {
                    let result = match op {
                        BinOp::Add => l.as_number() + r.as_number(),
                        BinOp::Sub => l.as_number() - r.as_number(),
                        _ => unreachable!(),
                    };
                    Ok(Value::UnitValue(result, l_dim))
                } else {
                    Err(EvalError::new(format!(
                        "Cannot {} values with incompatible dimensions: {} and {}",
                        op, l_dim, r_dim
                    )))
                }
            }
            BinOp::Mul => {
                // Multiplication: dimensions add
                let l_dim = l.dimension();
                let r_dim = r.dimension();
                let result = l.as_number() * r.as_number();
                let dim = l_dim.multiply(&r_dim);
                if dim.is_dimensionless() {
                    Ok(Value::Number(result))
                } else {
                    Ok(Value::UnitValue(result, dim))
                }
            }
            BinOp::Div => {
                // Division: dimensions subtract
                let l_dim = l.dimension();
                let r_dim = r.dimension();
                if r.as_number() == 0.0 {
                    return Err(EvalError::new("Division by zero"));
                }
                let result = l.as_number() / r.as_number();
                let dim = l_dim.divide(&r_dim);
                if dim.is_dimensionless() {
                    Ok(Value::Number(result))
                } else {
                    Ok(Value::UnitValue(result, dim))
                }
            }
            BinOp::Mod => {
                // Modulo: both must be dimensionless or same dimension
                let l_dim = l.dimension();
                let r_dim = r.dimension();
                if !l_dim.is_dimensionless() || !r_dim.is_dimensionless() {
                    return Err(EvalError::new(
                        "Modulo (%) requires dimensionless values".to_string(),
                    ));
                }
                if r.as_number() == 0.0 {
                    return Err(EvalError::new("Modulo by zero"));
                }
                Ok(Value::Number(l.as_number() % r.as_number()))
            }
            BinOp::Pow => {
                // Power: if exponent is an integer, multiply dimension
                let l_dim = l.dimension();
                let r_dim = r.dimension();
                if !r_dim.is_dimensionless() {
                    return Err(EvalError::new("Exponent must be dimensionless".to_string()));
                }
                let result = l.as_number().powf(r.as_number());
                // If the exponent is an integer, we can compute the resulting dimension
                if r.as_number() == r.as_number().trunc() {
                    let exp = r.as_number() as i32;
                    let dim = l_dim.power(exp);
                    if dim.is_dimensionless() {
                        Ok(Value::Number(result))
                    } else {
                        Ok(Value::UnitValue(result, dim))
                    }
                } else {
                    // Non-integer power: only valid for dimensionless values
                    if !l_dim.is_dimensionless() {
                        return Err(EvalError::new(
                            "Non-integer power requires dimensionless base".to_string(),
                        ));
                    }
                    Ok(Value::Number(result))
                }
            }
            BinOp::Eq => Ok(Value::Boolean(l.as_number() == r.as_number())),
            BinOp::Ne => Ok(Value::Boolean(l.as_number() != r.as_number())),
            BinOp::Lt => Ok(Value::Boolean(l.as_number() < r.as_number())),
            BinOp::Gt => Ok(Value::Boolean(l.as_number() > r.as_number())),
            BinOp::Le => Ok(Value::Boolean(l.as_number() <= r.as_number())),
            BinOp::Ge => Ok(Value::Boolean(l.as_number() >= r.as_number())),
            BinOp::And => {
                let lb = l.as_number() != 0.0;
                let rb = r.as_number() != 0.0;
                Ok(Value::Boolean(lb && rb))
            }
            BinOp::Or => {
                let lb = l.as_number() != 0.0;
                let rb = r.as_number() != 0.0;
                Ok(Value::Boolean(lb || rb))
            }
        }
    }

    fn eval_unary(&self, op: &UnaryOp, val: Value) -> Result<Value, EvalError> {
        match op {
            UnaryOp::Neg => {
                let dim = val.dimension();
                let n = -val.as_number();
                if dim.is_dimensionless() {
                    Ok(Value::Number(n))
                } else {
                    Ok(Value::UnitValue(n, dim))
                }
            }
            UnaryOp::Not => Ok(Value::Boolean(val.as_number() == 0.0)),
        }
    }

    /// Convert a value to a specific unit.
    pub fn convert_to_unit(&self, value: &Value, unit: &str) -> Result<Value, EvalError> {
        match value {
            Value::UnitValue(n, dim) => {
                let converted = self
                    .units
                    .from_base(*n, dim, unit)
                    .map_err(EvalError::new)?;
                Ok(Value::Number(converted))
            }
            Value::Number(n) => {
                // Converting a dimensionless number to a unit
                let (base_value, dim) = self.units.to_base(*n, unit).map_err(EvalError::new)?;
                Ok(Value::UnitValue(base_value, dim))
            }
            Value::Boolean(b) => Err(EvalError::new(format!(
                "Cannot convert boolean {} to unit {}",
                b, unit
            ))),
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to evaluate an expression string.
pub fn evaluate(input: &str) -> Result<Value, EvalError> {
    let expr = crate::parser::parse(input).map_err(|e| EvalError::new(e.to_string()))?;
    let evaluator = Evaluator::new();
    let mut env = Environment::new();
    evaluator.eval(&expr, &mut env)
}

/// Evaluate an expression string with a pre-existing environment.
pub fn evaluate_with_env(input: &str, env: &mut Environment) -> Result<Value, EvalError> {
    let expr = crate::parser::parse(input).map_err(|e| EvalError::new(e.to_string()))?;
    let evaluator = Evaluator::new();
    evaluator.eval(&expr, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_str(s: &str) -> Value {
        evaluate(s).unwrap_or_else(|e| panic!("Failed to evaluate '{}': {}", s, e))
    }

    fn eval_str_err(s: &str) -> EvalError {
        evaluate(s).unwrap_err()
    }

    #[test]
    fn test_simple_arithmetic() {
        assert_eq!(eval_str("1 + 2"), Value::Number(3.0));
        assert_eq!(eval_str("10 - 3"), Value::Number(7.0));
        assert_eq!(eval_str("4 * 5"), Value::Number(20.0));
        assert_eq!(eval_str("20 / 4"), Value::Number(5.0));
        assert_eq!(eval_str("2 ^ 3"), Value::Number(8.0));
        assert_eq!(eval_str("17 % 5"), Value::Number(2.0));
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval_str("1 + 2 * 3"), Value::Number(7.0));
        assert_eq!(eval_str("(1 + 2) * 3"), Value::Number(9.0));
        assert_eq!(eval_str("2 ^ 3 ^ 2"), Value::Number(512.0)); // right-associative
    }

    #[test]
    fn test_unary() {
        assert_eq!(eval_str("-5"), Value::Number(-5.0));
        assert_eq!(eval_str("--5"), Value::Number(5.0));
        assert_eq!(eval_str("!0"), Value::Boolean(true));
        assert_eq!(eval_str("!1"), Value::Boolean(false));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval_str("1 < 2"), Value::Boolean(true));
        assert_eq!(eval_str("2 < 1"), Value::Boolean(false));
        assert_eq!(eval_str("1 == 1"), Value::Boolean(true));
        assert_eq!(eval_str("1 != 2"), Value::Boolean(true));
        assert_eq!(eval_str("2 >= 2"), Value::Boolean(true));
        assert_eq!(eval_str("3 <= 2"), Value::Boolean(false));
    }

    #[test]
    fn test_logical() {
        assert_eq!(eval_str("true && false"), Value::Boolean(false));
        assert_eq!(eval_str("true || false"), Value::Boolean(true));
        assert_eq!(eval_str("!true"), Value::Boolean(false));
    }

    #[test]
    fn test_constants() {
        let result = eval_str("pi");
        assert!((result.as_number() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_functions() {
        let result = eval_str("sqrt(16)");
        assert!((result.as_number() - 4.0).abs() < 1e-10);

        let result = eval_str("abs(-5)");
        assert!((result.as_number() - 5.0).abs() < 1e-10);

        let result = eval_str("max(1, 2, 3)");
        assert!((result.as_number() - 3.0).abs() < 1e-10);

        let result = eval_str("sin(0)");
        assert!(result.as_number().abs() < 1e-10);
    }

    #[test]
    fn test_assignment() {
        let mut env = Environment::new();
        let result = evaluate_with_env("x = 5", &mut env).unwrap();
        assert_eq!(result, Value::Number(5.0));
        assert!(env.contains("x"));

        let result = evaluate_with_env("x + 3", &mut env).unwrap();
        assert_eq!(result, Value::Number(8.0));
    }

    #[test]
    fn test_unit_value() {
        let result = eval_str("5m");
        assert!(result.is_unit_value());
        assert!((result.as_number() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_unit_addition_compatible() {
        let result = eval_str("5m + 3m");
        assert!(result.is_unit_value());
        assert!((result.as_number() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_unit_addition_incompatible() {
        let err = eval_str_err("5m + 3kg");
        assert!(err.message.contains("incompatible"));
    }

    #[test]
    fn test_unit_multiplication() {
        let result = eval_str("5m * 3m");
        // 5m * 3m = 15 m^2
        assert!(result.is_unit_value());
        assert!((result.as_number() - 15.0).abs() < 1e-10);
        let dim = result.dimension();
        assert_eq!(dim.exponents[0], 2); // Length^2
    }

    #[test]
    fn test_unit_division() {
        let result = eval_str("10m / 2s");
        // 10m / 2s = 5 m/s
        assert!(result.is_unit_value());
        assert!((result.as_number() - 5.0).abs() < 1e-10);
        let dim = result.dimension();
        assert_eq!(dim.exponents[0], 1); // Length
        assert_eq!(dim.exponents[2], -1); // Time^-1
    }

    #[test]
    fn test_unit_power() {
        let result = eval_str("3m ^ 2");
        assert!(result.is_unit_value());
        assert!((result.as_number() - 9.0).abs() < 1e-10);
        let dim = result.dimension();
        assert_eq!(dim.exponents[0], 2); // Length^2
    }

    #[test]
    fn test_unit_conversion() {
        let evaluator = Evaluator::new();
        let value = eval_str("1m");
        let converted = evaluator.convert_to_unit(&value, "ft").unwrap();
        assert!((converted.as_number() - 3.28083989501).abs() < 1e-6);
    }

    #[test]
    fn test_temperature_conversion() {
        let evaluator = Evaluator::new();
        let value = eval_str("0C");
        let converted = evaluator.convert_to_unit(&value, "F").unwrap();
        assert!((converted.as_number() - 32.0).abs() < 1e-6);
    }

    #[test]
    fn test_division_by_zero() {
        // Division by zero returns an error
        let err = eval_str_err("1 / 0");
        assert!(err.to_string().contains("Division by zero"));
    }

    #[test]
    fn test_modulo_by_zero() {
        let err = eval_str_err("5 % 0");
        assert!(err.message.contains("zero"));
    }

    #[test]
    fn test_undefined_variable() {
        let err = eval_str_err("undefined_var");
        assert!(err.message.contains("Undefined variable"));
    }

    #[test]
    fn test_complex_expression() {
        let result = eval_str("2 * (3 + 4) ^ 2 - 1");
        // 2 * 7^2 - 1 = 2 * 49 - 1 = 98 - 1 = 97
        assert_eq!(result, Value::Number(97.0));
    }

    #[test]
    fn test_nested_functions() {
        let result = eval_str("sqrt(abs(-16))");
        assert!((result.as_number() - 4.0).abs() < 1e-10);
    }
}
