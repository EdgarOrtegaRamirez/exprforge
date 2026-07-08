//! Built-in mathematical functions and constants.
//!
//! Provides a registry of functions that can be called from expressions.
//! Includes math, trigonometry, statistics, and utility functions.

use crate::ast::EvalError;
use std::collections::HashMap;

/// A function that can be called from an expression.
pub type BuiltinFunc = fn(&[f64]) -> Result<f64, EvalError>;

/// Registry of built-in functions and constants.
pub struct FunctionRegistry {
    functions: HashMap<String, BuiltinFunc>,
    constants: HashMap<String, f64>,
}

impl FunctionRegistry {
    /// Create a new function registry with all built-in functions.
    pub fn new() -> Self {
        let mut reg = FunctionRegistry {
            functions: HashMap::new(),
            constants: HashMap::new(),
        };
        reg.register_math_functions();
        reg.register_trig_functions();
        reg.register_hyperbolic_functions();
        reg.register_rounding_functions();
        reg.register_statistics_functions();
        reg.register_utility_functions();
        reg.register_constants();
        reg
    }

    /// Look up a function by name.
    pub fn get(&self, name: &str) -> Option<&BuiltinFunc> {
        self.functions.get(name)
    }

    /// Look up a constant by name.
    pub fn get_constant(&self, name: &str) -> Option<f64> {
        self.constants.get(name).copied()
    }

    /// Get all function names.
    pub fn function_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.functions.keys().cloned().collect();
        names.sort();
        names
    }

    /// Get all constant names.
    pub fn constant_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.constants.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if a name is a function.
    pub fn is_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Check if a name is a constant.
    pub fn is_constant(&self, name: &str) -> bool {
        self.constants.contains_key(name)
    }

    fn register(&mut self, name: &str, func: BuiltinFunc) {
        self.functions.insert(name.to_string(), func);
    }

    fn register_const(&mut self, name: &str, value: f64) {
        self.constants.insert(name.to_string(), value);
    }

    fn register_math_functions(&mut self) {
        self.register("abs", |args| simple1(args, "abs", f64::abs));
        self.register("sqrt", |args| simple1(args, "sqrt", f64::sqrt));
        self.register("cbrt", |args| simple1(args, "cbrt", f64::cbrt));
        self.register("exp", |args| simple1(args, "exp", f64::exp));
        self.register("ln", |args| simple1(args, "ln", f64::ln));
        self.register("log", |args| simple1(args, "log", f64::log10));
        self.register("log2", |args| simple1(args, "log2", f64::log2));
        self.register("log10", |args| simple1(args, "log10", f64::log10));
        self.register("fact", |args| {
            one_arg(args, "fact", |x| {
                if x < 0.0 || x != x.trunc() {
                    return Err(EvalError::new(format!(
                        "fact() requires a non-negative integer, got {}",
                        x
                    )));
                }
                let n = x as u64;
                let mut result = 1.0;
                for i in 1..=n {
                    result *= i as f64;
                }
                Ok(result)
            })
        });
        self.register("gcd", |args| {
            two_args(args, "gcd", |a, b| {
                let a = a as i64;
                let b = b as i64;
                if a < 0 || b < 0 {
                    return Err(EvalError::new("gcd() requires non-negative integers"));
                }
                Ok(gcd(a, b) as f64)
            })
        });
        self.register("lcm", |args| {
            two_args(args, "lcm", |a, b| {
                let a = a as i64;
                let b = b as i64;
                if a < 0 || b < 0 || (a == 0 && b == 0) {
                    return Err(EvalError::new(
                        "lcm() requires non-negative integers, not both zero",
                    ));
                }
                Ok(lcm(a, b) as f64)
            })
        });
    }

    fn register_trig_functions(&mut self) {
        self.register("sin", |args| simple1(args, "sin", f64::sin));
        self.register("cos", |args| simple1(args, "cos", f64::cos));
        self.register("tan", |args| simple1(args, "tan", f64::tan));
        self.register("asin", |args| simple1(args, "asin", f64::asin));
        self.register("acos", |args| simple1(args, "acos", f64::acos));
        self.register("atan", |args| simple1(args, "atan", f64::atan));
        self.register("atan2", |args| {
            two_args(args, "atan2", |y, x| Ok(y.atan2(x)))
        });
        self.register("sind", |args| {
            simple1(args, "sind", |x| (x * std::f64::consts::PI / 180.0).sin())
        });
        self.register("cosd", |args| {
            simple1(args, "cosd", |x| (x * std::f64::consts::PI / 180.0).cos())
        });
        self.register("tand", |args| {
            simple1(args, "tand", |x| (x * std::f64::consts::PI / 180.0).tan())
        });
    }

    fn register_hyperbolic_functions(&mut self) {
        self.register("sinh", |args| simple1(args, "sinh", f64::sinh));
        self.register("cosh", |args| simple1(args, "cosh", f64::cosh));
        self.register("tanh", |args| simple1(args, "tanh", f64::tanh));
        self.register("asinh", |args| simple1(args, "asinh", f64::asinh));
        self.register("acosh", |args| simple1(args, "acosh", f64::acosh));
        self.register("atanh", |args| simple1(args, "atanh", f64::atanh));
    }

    fn register_rounding_functions(&mut self) {
        self.register("ceil", |args| simple1(args, "ceil", f64::ceil));
        self.register("floor", |args| simple1(args, "floor", f64::floor));
        self.register("round", |args| simple1(args, "round", f64::round));
        self.register("trunc", |args| simple1(args, "trunc", f64::trunc));
        self.register("sign", |args| simple1(args, "sign", f64::signum));
    }

    fn register_statistics_functions(&mut self) {
        self.register("min", |args| {
            if args.is_empty() {
                return Err(EvalError::new("min() requires at least one argument"));
            }
            Ok(args.iter().copied().fold(f64::INFINITY, f64::min))
        });
        self.register("max", |args| {
            if args.is_empty() {
                return Err(EvalError::new("max() requires at least one argument"));
            }
            Ok(args.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        });
        self.register("sum", |args| Ok(args.iter().sum()));
        self.register("mean", |args| {
            if args.is_empty() {
                return Err(EvalError::new("mean() requires at least one argument"));
            }
            Ok(args.iter().sum::<f64>() / args.len() as f64)
        });
        self.register("median", |args| {
            if args.is_empty() {
                return Err(EvalError::new("median() requires at least one argument"));
            }
            let mut sorted = args.to_vec();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mid = sorted.len() / 2;
            if sorted.len() % 2 == 0 {
                Ok((sorted[mid - 1] + sorted[mid]) / 2.0)
            } else {
                Ok(sorted[mid])
            }
        });
        self.register("var", |args| {
            if args.is_empty() {
                return Err(EvalError::new("var() requires at least one argument"));
            }
            let mean = args.iter().sum::<f64>() / args.len() as f64;
            let variance = args.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / args.len() as f64;
            Ok(variance)
        });
        self.register("std", |args| {
            if args.is_empty() {
                return Err(EvalError::new("std() requires at least one argument"));
            }
            let mean = args.iter().sum::<f64>() / args.len() as f64;
            let variance = args.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / args.len() as f64;
            Ok(variance.sqrt())
        });
        self.register("prod", |args| Ok(args.iter().fold(1.0, |a, &b| a * b)));
    }

    fn register_utility_functions(&mut self) {
        self.register("clamp", |args| {
            if args.len() != 3 {
                return Err(EvalError::new(
                    "clamp() requires 3 arguments: value, min, max",
                ));
            }
            Ok(args[0].max(args[1]).min(args[2]))
        });
        self.register("lerp", |args| {
            if args.len() != 3 {
                return Err(EvalError::new("lerp() requires 3 arguments: a, b, t"));
            }
            Ok(args[0] + (args[1] - args[0]) * args[2])
        });
        self.register("hypot", |args| {
            if args.len() != 2 {
                return Err(EvalError::new("hypot() requires 2 arguments"));
            }
            Ok(args[0].hypot(args[1]))
        });
        self.register("deg2rad", |args| {
            simple1(args, "deg2rad", |x| x * std::f64::consts::PI / 180.0)
        });
        self.register("rad2deg", |args| {
            simple1(args, "rad2deg", |x| x * 180.0 / std::f64::consts::PI)
        });
    }

    fn register_constants(&mut self) {
        self.register_const("pi", std::f64::consts::PI);
        self.register_const("e", std::f64::consts::E);
        self.register_const("tau", std::f64::consts::TAU);
        self.register_const("phi", 1.618_033_988_749_895); // golden ratio
        self.register_const("sqrt2", std::f64::consts::SQRT_2);
        self.register_const("sqrt3", 3.0_f64.sqrt());
        self.register_const("ln2", std::f64::consts::LN_2);
        self.register_const("ln10", std::f64::consts::LN_10);
        self.register_const("euler", std::f64::consts::EULER_GAMMA);
        self.register_const("inf", f64::INFINITY);
        self.register_const("nan", f64::NAN);
    }
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for argument validation

/// Simple one-argument function that returns f64 directly.
fn simple1<F>(args: &[f64], name: &str, f: F) -> Result<f64, EvalError>
where
    F: Fn(f64) -> f64,
{
    if args.len() != 1 {
        return Err(EvalError::new(format!(
            "{}() requires exactly 1 argument, got {}",
            name,
            args.len()
        )));
    }
    Ok(f(args[0]))
}

/// One-argument function that may return an error.
fn one_arg<F>(args: &[f64], name: &str, f: F) -> Result<f64, EvalError>
where
    F: Fn(f64) -> Result<f64, EvalError>,
{
    if args.len() != 1 {
        return Err(EvalError::new(format!(
            "{}() requires exactly 1 argument, got {}",
            name,
            args.len()
        )));
    }
    f(args[0])
}

fn two_args<F>(args: &[f64], name: &str, f: F) -> Result<f64, EvalError>
where
    F: Fn(f64, f64) -> Result<f64, EvalError>,
{
    if args.len() != 2 {
        return Err(EvalError::new(format!(
            "{}() requires exactly 2 arguments, got {}",
            name,
            args.len()
        )));
    }
    f(args[0], args[1])
}

fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 0;
    }
    (a.abs() / gcd(a, b)) * b.abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(reg: &FunctionRegistry, name: &str, args: &[f64]) -> f64 {
        let func = reg
            .get(name)
            .unwrap_or_else(|| panic!("Function {} not found", name));
        func(args).unwrap_or_else(|e| panic!("Error calling {}: {}", name, e))
    }

    #[test]
    fn test_math_functions() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "abs", &[-5.0]) - 5.0).abs() < 1e-10);
        assert!((call(&reg, "sqrt", &[16.0]) - 4.0).abs() < 1e-10);
        assert!((call(&reg, "exp", &[0.0]) - 1.0).abs() < 1e-10);
        assert!((call(&reg, "ln", &[1.0]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_trig_functions() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "sin", &[0.0]) - 0.0).abs() < 1e-10);
        assert!((call(&reg, "cos", &[0.0]) - 1.0).abs() < 1e-10);
        assert!((call(&reg, "sind", &[90.0]) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_statistics_functions() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "min", &[3.0, 1.0, 2.0]) - 1.0).abs() < 1e-10);
        assert!((call(&reg, "max", &[3.0, 1.0, 2.0]) - 3.0).abs() < 1e-10);
        assert!((call(&reg, "sum", &[1.0, 2.0, 3.0]) - 6.0).abs() < 1e-10);
        assert!((call(&reg, "mean", &[1.0, 2.0, 3.0]) - 2.0).abs() < 1e-10);
        assert!((call(&reg, "median", &[1.0, 3.0, 2.0]) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_constants() {
        let reg = FunctionRegistry::new();
        assert!((reg.get_constant("pi").unwrap() - std::f64::consts::PI).abs() < 1e-10);
        assert!((reg.get_constant("e").unwrap() - std::f64::consts::E).abs() < 1e-10);
        assert!((reg.get_constant("tau").unwrap() - std::f64::consts::TAU).abs() < 1e-10);
    }

    #[test]
    fn test_factorial() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "fact", &[5.0]) - 120.0).abs() < 1e-10);
        assert!((call(&reg, "fact", &[0.0]) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gcd_lcm() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "gcd", &[12.0, 8.0]) - 4.0).abs() < 1e-10);
        assert!((call(&reg, "lcm", &[4.0, 6.0]) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_clamp() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "clamp", &[5.0, 0.0, 10.0]) - 5.0).abs() < 1e-10);
        assert!((call(&reg, "clamp", &[-5.0, 0.0, 10.0]) - 0.0).abs() < 1e-10);
        assert!((call(&reg, "clamp", &[15.0, 0.0, 10.0]) - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_lerp() {
        let reg = FunctionRegistry::new();
        assert!((call(&reg, "lerp", &[0.0, 10.0, 0.5]) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_error_wrong_args() {
        let reg = FunctionRegistry::new();
        let func = reg.get("sqrt").unwrap();
        let result = func(&[1.0, 2.0]);
        assert!(result.is_err());
    }

    #[test]
    fn test_error_no_args() {
        let reg = FunctionRegistry::new();
        let func = reg.get("min").unwrap();
        let result = func(&[]);
        assert!(result.is_err());
    }
}
