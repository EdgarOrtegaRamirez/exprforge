//! Recursive descent parser with Pratt parsing for operator precedence.
//!
//! Consumes tokens from the lexer and produces an AST. Handles operator
//! precedence, associativity, function calls, and unit annotations.

use crate::ast::*;
use crate::lexer::{SpannedToken, Token};

/// The parser converts a token stream into an AST.
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token list.
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse a complete expression.
    pub fn parse(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_assignment()?;
        self.expect_eof()?;
        Ok(expr)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos].token
    }

    fn peek_pos(&self) -> usize {
        self.tokens[self.pos].pos
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].token.clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        token
    }

    fn expect_eof(&self) -> Result<(), ParseError> {
        match self.peek() {
            Token::Eof => Ok(()),
            t => Err(ParseError {
                message: format!("Unexpected token after expression: {}", t),
                pos: self.peek_pos(),
            }),
        }
    }

    /// assignment := identifier '=' convert_expr | convert_expr
    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        // Try to parse as assignment: identifier = expr
        if let Token::Identifier(name) = self.peek() {
            let name = name.clone();
            let pos = self.peek_pos();
            // Look ahead for '='
            if self.pos + 1 < self.tokens.len() {
                if let Token::Equal = self.tokens[self.pos + 1].token {
                    self.advance(); // identifier
                    self.advance(); // =
                    let value = self.parse_convert()?;
                    return Ok(Expr::Assign(name, Box::new(value)));
                }
            }
            let _ = pos;
        }

        self.parse_convert()
    }

    /// convert_expr := or_expr ('->' identifier)?
    fn parse_convert(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_or()?;

        if matches!(self.peek(), Token::Arrow) {
            self.advance(); // ->
            match self.peek() {
                Token::Identifier(name) => {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Convert(Box::new(expr), name))
                }
                Token::Unit(name) => {
                    let name = name.clone();
                    self.advance();
                    Ok(Expr::Convert(Box::new(expr), name))
                }
                t => Err(ParseError {
                    message: format!("Expected unit name after '->', got: {}", t),
                    pos: self.peek_pos(),
                }),
            }
        } else {
            Ok(expr)
        }
    }

    /// or_expr := and_expr ('||' and_expr)*
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        while matches!(self.peek(), Token::OrOr) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::Binary(Box::new(left), BinOp::Or, Box::new(right));
        }

        Ok(left)
    }

    /// and_expr := comparison ('&&' comparison)*
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;

        while matches!(self.peek(), Token::AndAnd) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::Binary(Box::new(left), BinOp::And, Box::new(right));
        }

        Ok(left)
    }

    /// comparison := addition (('==' | '!=' | '<' | '>' | '<=' | '>=') addition)*
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;

        loop {
            let op = match self.peek() {
                Token::EqualEqual => BinOp::Eq,
                Token::BangEqual => BinOp::Ne,
                Token::Less => BinOp::Lt,
                Token::Greater => BinOp::Gt,
                Token::LessEqual => BinOp::Le,
                Token::GreaterEqual => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.parse_addition()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// addition := multiplication (('+' | '-') multiplication)*
    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// multiplication := unary (('*' | '/' | '%') unary)*
    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }

        Ok(left)
    }

    /// unary := ('-' | '!') unary | exponent
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Neg, Box::new(expr)))
            }
            Token::Bang => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryOp::Not, Box::new(expr)))
            }
            _ => self.parse_exponent(),
        }
    }

    /// exponent := postfix ('^' unary)*  (right-associative)
    fn parse_exponent(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_postfix()?;

        if matches!(self.peek(), Token::Caret) {
            self.advance();
            let right = self.parse_unary()?; // right-associative, allows -2^-3
            return Ok(Expr::Binary(Box::new(left), BinOp::Pow, Box::new(right)));
        }

        Ok(left)
    }

    /// postfix := primary (unit_annotation)*
    /// A unit annotation is an identifier immediately following a number
    /// without an operator, e.g., "5m" or "3.14rad".
    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let expr = self.parse_primary()?;

        // Check for unit annotation: a number followed by an identifier
        if let Expr::Number(n) = &expr {
            if let Token::Identifier(unit) = self.peek() {
                // Make sure this isn't a function call (identifier followed by '(')
                let is_function_call = self.pos + 1 < self.tokens.len()
                    && matches!(self.tokens[self.pos + 1].token, Token::LeftParen);

                if !is_function_call {
                    let unit = unit.clone();
                    self.advance();
                    return Ok(Expr::UnitValue(*n, unit));
                }
            }
        }

        Ok(expr)
    }

    /// primary := number | boolean | identifier | function_call | '(' expression ')'
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let pos = self.peek_pos();
        let token = self.advance();

        match token {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Boolean(b) => Ok(Expr::Boolean(b)),
            Token::Identifier(name) => {
                // Check for function call
                if matches!(self.peek(), Token::LeftParen) {
                    self.advance(); // consume '('
                    let args = self.parse_args()?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Token::LeftParen => {
                let expr = self.parse_assignment()?;
                match self.advance() {
                    Token::RightParen => Ok(expr),
                    t => Err(ParseError {
                        message: format!("Expected ')', got {}", t),
                        pos: self.peek_pos(),
                    }),
                }
            }
            Token::Eof => Err(ParseError {
                message: "Unexpected end of input".to_string(),
                pos,
            }),
            t => Err(ParseError {
                message: format!("Unexpected token: {}", t),
                pos,
            }),
        }
    }

    /// Parse function arguments: (expr (',' expr)*)?
    fn parse_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();

        if matches!(self.peek(), Token::RightParen) {
            self.advance();
            return Ok(args);
        }

        // Parse first argument
        args.push(self.parse_assignment()?);

        // Parse remaining arguments
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            args.push(self.parse_assignment()?);
        }

        // Expect closing paren
        match self.advance() {
            Token::RightParen => Ok(args),
            t => Err(ParseError {
                message: format!("Expected ')' or ',', got {}", t),
                pos: self.peek_pos(),
            }),
        }
    }
}

/// Convenience function to parse an expression string.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = crate::lexer::tokenize(input).map_err(|e| ParseError {
        message: e.message,
        pos: e.pos,
    })?;
    Parser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(s: &str) -> Expr {
        parse(s).unwrap_or_else(|e| panic!("Failed to parse '{}': {}", s, e))
    }

    #[test]
    fn test_number() {
        assert_eq!(parse_str("42"), Expr::Number(42.0));
    }

    #[test]
    fn test_boolean() {
        assert_eq!(parse_str("true"), Expr::Boolean(true));
        assert_eq!(parse_str("false"), Expr::Boolean(false));
    }

    #[test]
    fn test_variable() {
        assert_eq!(parse_str("x"), Expr::Variable("x".to_string()));
    }

    #[test]
    fn test_unit_value() {
        assert_eq!(parse_str("5m"), Expr::UnitValue(5.0, "m".to_string()));
        assert_eq!(parse_str("2.5rad"), Expr::UnitValue(2.5, "rad".to_string()));
    }

    #[test]
    fn test_addition() {
        let expected = Expr::Binary(
            Box::new(Expr::Number(1.0)),
            BinOp::Add,
            Box::new(Expr::Number(2.0)),
        );
        assert_eq!(parse_str("1 + 2"), expected);
    }

    #[test]
    fn test_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let expected = Expr::Binary(
            Box::new(Expr::Number(1.0)),
            BinOp::Add,
            Box::new(Expr::Binary(
                Box::new(Expr::Number(2.0)),
                BinOp::Mul,
                Box::new(Expr::Number(3.0)),
            )),
        );
        assert_eq!(parse_str("1 + 2 * 3"), expected);
    }

    #[test]
    fn test_parentheses() {
        // (1 + 2) * 3 should parse as (1 + 2) * 3
        let expected = Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Number(1.0)),
                BinOp::Add,
                Box::new(Expr::Number(2.0)),
            )),
            BinOp::Mul,
            Box::new(Expr::Number(3.0)),
        );
        assert_eq!(parse_str("(1 + 2) * 3"), expected);
    }

    #[test]
    fn test_exponent_right_assoc() {
        // 2 ^ 3 ^ 2 should parse as 2 ^ (3 ^ 2) (right-associative)
        let expected = Expr::Binary(
            Box::new(Expr::Number(2.0)),
            BinOp::Pow,
            Box::new(Expr::Binary(
                Box::new(Expr::Number(3.0)),
                BinOp::Pow,
                Box::new(Expr::Number(2.0)),
            )),
        );
        assert_eq!(parse_str("2 ^ 3 ^ 2"), expected);
    }

    #[test]
    fn test_unary_minus() {
        let expected = Expr::Unary(UnaryOp::Neg, Box::new(Expr::Number(5.0)));
        assert_eq!(parse_str("-5"), expected);
    }

    #[test]
    fn test_unary_minus_with_exponent() {
        // -2 ^ 2 should parse as -(2 ^ 2), not (-2) ^ 2
        let expected = Expr::Unary(
            UnaryOp::Neg,
            Box::new(Expr::Binary(
                Box::new(Expr::Number(2.0)),
                BinOp::Pow,
                Box::new(Expr::Number(2.0)),
            )),
        );
        assert_eq!(parse_str("-2 ^ 2"), expected);
    }

    #[test]
    fn test_convert_expression() {
        let expected = Expr::Convert(
            Box::new(Expr::UnitValue(5.0, "m".to_string())),
            "ft".to_string(),
        );
        assert_eq!(parse_str("5m -> ft"), expected);
    }

    #[test]
    fn test_comparison() {
        let expected = Expr::Binary(
            Box::new(Expr::Number(1.0)),
            BinOp::Lt,
            Box::new(Expr::Number(2.0)),
        );
        assert_eq!(parse_str("1 < 2"), expected);
    }

    #[test]
    fn test_function_call() {
        let expected = Expr::Call("sin".to_string(), vec![Expr::Variable("x".to_string())]);
        assert_eq!(parse_str("sin(x)"), expected);
    }

    #[test]
    fn test_function_call_no_args() {
        let expected = Expr::Call("pi".to_string(), vec![]);
        assert_eq!(parse_str("pi()"), expected);
    }

    #[test]
    fn test_function_call_multiple_args() {
        let expected = Expr::Call(
            "max".to_string(),
            vec![Expr::Number(1.0), Expr::Number(2.0), Expr::Number(3.0)],
        );
        assert_eq!(parse_str("max(1, 2, 3)"), expected);
    }

    #[test]
    fn test_assignment() {
        let expected = Expr::Assign("x".to_string(), Box::new(Expr::Number(5.0)));
        assert_eq!(parse_str("x = 5"), expected);
    }

    #[test]
    fn test_complex_expression() {
        // 3 * (4 + 5) ^ 2 - 1
        let expected = Expr::Binary(
            Box::new(Expr::Binary(
                Box::new(Expr::Number(3.0)),
                BinOp::Mul,
                Box::new(Expr::Binary(
                    Box::new(Expr::Binary(
                        Box::new(Expr::Number(4.0)),
                        BinOp::Add,
                        Box::new(Expr::Number(5.0)),
                    )),
                    BinOp::Pow,
                    Box::new(Expr::Number(2.0)),
                )),
            )),
            BinOp::Sub,
            Box::new(Expr::Number(1.0)),
        );
        assert_eq!(parse_str("3 * (4 + 5) ^ 2 - 1"), expected);
    }

    #[test]
    fn test_error_unexpected_token() {
        let result = parse("+ 1");
        assert!(result.is_err());
    }

    #[test]
    fn test_error_unclosed_paren() {
        let result = parse("(1 + 2");
        assert!(result.is_err());
    }

    #[test]
    fn test_logical_operators() {
        let expected = Expr::Binary(
            Box::new(Expr::Boolean(true)),
            BinOp::And,
            Box::new(Expr::Boolean(false)),
        );
        assert_eq!(parse_str("true && false"), expected);
    }
}
