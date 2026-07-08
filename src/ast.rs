//! Abstract Syntax Tree (AST) for mathematical expressions.
//!
//! Defines the node types that represent parsed expressions. The parser
//! produces these nodes, and the evaluator walks them to compute results.

use std::fmt;

/// Binary operators supported in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Mod => write!(f, "%"),
            BinOp::Pow => write!(f, "^"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
        }
    }
}

/// Unary operators supported in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

/// A value that can result from evaluating an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A plain number (dimensionless).
    Number(f64),
    /// A boolean value.
    Boolean(bool),
    /// A value with a unit (value in base SI units, unit dimension).
    UnitValue(f64, crate::units::Dimension),
}

impl Value {
    /// Get the numeric value, regardless of whether it has a unit.
    pub fn as_number(&self) -> f64 {
        match self {
            Value::Number(n) | Value::UnitValue(n, _) => *n,
            Value::Boolean(b) => {
                if *b {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    /// Get the dimension of this value (empty for plain numbers).
    pub fn dimension(&self) -> crate::units::Dimension {
        match self {
            Value::UnitValue(_, dim) => dim.clone(),
            _ => crate::units::Dimension::dimensionless(),
        }
    }

    /// Check if this is a boolean value.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    /// Get the boolean value. Returns false for non-boolean values.
    pub fn as_boolean(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::UnitValue(n, _) => *n != 0.0,
        }
    }

    /// Check if this is a unit value.
    pub fn is_unit_value(&self) -> bool {
        matches!(self, Value::UnitValue(_, _))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", format_number(*n)),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::UnitValue(n, dim) => {
                write!(f, "{} {}", format_number(*n), dim)
            }
        }
    }
}

/// Format a number nicely, avoiding unnecessary trailing zeros.
fn format_number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        // Use up to 10 decimal places, trimming trailing zeros
        let s = format!("{:.10}", n);
        let s = s.trim_end_matches('0');
        let s = s.trim_end_matches('.');
        s.to_string()
    }
}

/// AST node representing an expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal.
    Number(f64),
    /// A boolean literal.
    Boolean(bool),
    /// A variable reference.
    Variable(String),
    /// A value with a unit annotation (e.g., `5m`).
    UnitValue(f64, String),
    /// A binary operation.
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// A unary operation.
    Unary(UnaryOp, Box<Expr>),
    /// A function call.
    Call(String, Vec<Expr>),
    /// An assignment to a variable.
    Assign(String, Box<Expr>),
    /// A unit conversion (e.g., `5 m -> ft`).
    Convert(Box<Expr>, String),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(n) => write!(f, "{}", n),
            Expr::Boolean(b) => write!(f, "{}", b),
            Expr::Variable(s) => write!(f, "{}", s),
            Expr::UnitValue(n, u) => write!(f, "{}{}", n, u),
            Expr::Binary(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Expr::Unary(op, e) => write!(f, "({}{})", op, e),
            Expr::Call(name, args) => {
                let args_str: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                write!(f, "{}({})", name, args_str.join(", "))
            }
            Expr::Assign(name, e) => write!(f, "({} = {})", name, e),
            Expr::Convert(e, unit) => write!(f, "({} -> {})", e, unit),
        }
    }
}

/// Parse error with position information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub pos: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.pos, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Evaluation error.
#[derive(Debug, Clone)]
pub struct EvalError {
    pub message: String,
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Evaluation error: {}", self.message)
    }
}

impl std::error::Error for EvalError {}

impl EvalError {
    pub fn new(msg: impl Into<String>) -> Self {
        EvalError {
            message: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Number(42.0).to_string(), "42");
        assert_eq!(Value::Number(2.5).to_string(), "2.5");
        assert_eq!(Value::Boolean(true).to_string(), "true");
    }

    #[test]
    fn test_expr_display() {
        let expr = Expr::Binary(
            Box::new(Expr::Number(1.0)),
            BinOp::Add,
            Box::new(Expr::Number(2.0)),
        );
        assert_eq!(expr.to_string(), "(1 + 2)");
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(42.0), "42");
        assert_eq!(format_number(2.5), "2.5");
        assert_eq!(format_number(0.1), "0.1");
        assert_eq!(format_number(100.0), "100");
    }
}
