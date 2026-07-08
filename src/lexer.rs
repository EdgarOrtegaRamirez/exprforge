//! Tokenizer for mathematical expressions.
//!
//! Converts a string of characters into a sequence of tokens that can be
//! consumed by the parser. Supports numbers (integers, floats, scientific
//! notation), identifiers, operators, and punctuation.

use std::fmt;

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    Boolean(bool),

    // Identifiers and keywords
    Identifier(String),
    Unit(String),

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Bang,

    // Comparison
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,

    // Logical
    AndAnd,
    OrOr,

    // Assignment
    Equal,

    // Unit conversion
    Arrow,

    // Punctuation
    LeftParen,
    RightParen,
    Comma,

    // End
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::Boolean(b) => write!(f, "Boolean({})", b),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::Unit(s) => write!(f, "Unit({})", s),
            Token::Plus => write!(f, "Plus"),
            Token::Minus => write!(f, "Minus"),
            Token::Star => write!(f, "Star"),
            Token::Slash => write!(f, "Slash"),
            Token::Percent => write!(f, "Percent"),
            Token::Caret => write!(f, "Caret"),
            Token::Bang => write!(f, "Bang"),
            Token::EqualEqual => write!(f, "EqualEqual"),
            Token::BangEqual => write!(f, "BangEqual"),
            Token::Less => write!(f, "Less"),
            Token::Greater => write!(f, "Greater"),
            Token::LessEqual => write!(f, "LessEqual"),
            Token::GreaterEqual => write!(f, "GreaterEqual"),
            Token::AndAnd => write!(f, "AndAnd"),
            Token::OrOr => write!(f, "OrOr"),
            Token::Equal => write!(f, "Equal"),
            Token::Arrow => write!(f, "Arrow"),
            Token::LeftParen => write!(f, "LeftParen"),
            Token::RightParen => write!(f, "RightParen"),
            Token::Comma => write!(f, "Comma"),
            Token::Eof => write!(f, "Eof"),
        }
    }
}

/// A token with its position in the source text.
#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub pos: usize,
}

/// Lexical analysis error.
#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub pos: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lex error at position {}: {}", self.pos, self.message)
    }
}

impl std::error::Error for LexError {}

/// The lexer converts a string into a vector of spanned tokens.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Create a new lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    /// Tokenize the entire input string.
    pub fn tokenize(&mut self) -> Result<Vec<SpannedToken>, LexError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.chars.len() {
                tokens.push(SpannedToken {
                    token: Token::Eof,
                    pos: self.pos,
                });
                break;
            }

            let start = self.pos;
            let token = self.next_token()?;
            tokens.push(SpannedToken { token, pos: start });
        }

        Ok(tokens)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let c = self.peek().ok_or(LexError {
            message: "Unexpected end of input".to_string(),
            pos: self.pos,
        })?;

        // Numbers
        if c.is_ascii_digit() || (c == '.' && self.peek_next().is_some_and(|n| n.is_ascii_digit()))
        {
            return self.lex_number();
        }

        // Identifiers and keywords
        if c.is_alphabetic() || c == '_' {
            return self.lex_identifier();
        }

        // Operators and punctuation
        match c {
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    Ok(Token::Arrow)
                } else {
                    Ok(Token::Minus)
                }
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '/' => {
                self.advance();
                Ok(Token::Slash)
            }
            '%' => {
                self.advance();
                Ok(Token::Percent)
            }
            '^' => {
                self.advance();
                Ok(Token::Caret)
            }
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::EqualEqual)
                } else {
                    Ok(Token::Equal)
                }
            }
            '!' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::BangEqual)
                } else {
                    Ok(Token::Bang)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            '&' => {
                self.advance();
                if self.peek() == Some('&') {
                    self.advance();
                    Ok(Token::AndAnd)
                } else {
                    Err(LexError {
                        message: "Expected '&&', got '&'".to_string(),
                        pos: self.pos,
                    })
                }
            }
            '|' => {
                self.advance();
                if self.peek() == Some('|') {
                    self.advance();
                    Ok(Token::OrOr)
                } else {
                    Err(LexError {
                        message: "Expected '||', got '|'".to_string(),
                        pos: self.pos,
                    })
                }
            }
            _ => Err(LexError {
                message: format!("Unexpected character: '{}'", c),
                pos: self.pos,
            }),
        }
    }

    fn lex_number(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let mut s = String::new();
        let mut has_dot = false;
        let mut has_exp = false;

        // Integer part
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            s.push(self.chars[self.pos]);
            self.pos += 1;
        }

        // Fractional part
        if self.pos < self.chars.len() && self.chars[self.pos] == '.' {
            has_dot = true;
            s.push('.');
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                s.push(self.chars[self.pos]);
                self.pos += 1;
            }
        }

        // Exponent
        if self.pos < self.chars.len()
            && (self.chars[self.pos] == 'e' || self.chars[self.pos] == 'E')
        {
            // Make sure the 'e' is part of a number, not an identifier
            let next = self.chars.get(self.pos + 1).copied();
            if next.is_some_and(|c| c.is_ascii_digit() || c == '+' || c == '-') {
                has_exp = true;
                s.push(self.chars[self.pos]);
                self.pos += 1;

                if self.pos < self.chars.len()
                    && (self.chars[self.pos] == '+' || self.chars[self.pos] == '-')
                {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }

                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
            }
        }

        let _ = (has_dot, has_exp, start);

        s.parse::<f64>().map(Token::Number).map_err(|e| LexError {
            message: format!("Invalid number: {}", e),
            pos: start,
        })
    }

    fn lex_identifier(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let mut s = String::new();

        while self.pos < self.chars.len()
            && (self.chars[self.pos].is_alphanumeric() || self.chars[self.pos] == '_')
        {
            s.push(self.chars[self.pos]);
            self.pos += 1;
        }

        // Check for boolean literals
        match s.as_str() {
            "true" => return Ok(Token::Boolean(true)),
            "false" => return Ok(Token::Boolean(false)),
            _ => {}
        }

        // Check if this is a unit (follows a number without an operator)
        // The parser will disambiguate identifiers from units based on context.
        // For the lexer, we just produce an Identifier token.
        // The parser will check if an identifier immediately follows a number
        // and treat it as a unit.
        let _ = start;
        Ok(Token::Identifier(s))
    }
}

/// Convenience function to tokenize an expression string.
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    Lexer::new(input).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_numbers() {
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens.len(), 2); // Number + Eof
        assert_eq!(tokens[0].token, Token::Number(42.0));
    }

    #[test]
    fn test_float() {
        let tokens = tokenize("2.5").unwrap();
        assert_eq!(tokens[0].token, Token::Number(2.5));
    }

    #[test]
    fn test_scientific_notation() {
        let tokens = tokenize("1.5e10").unwrap();
        assert_eq!(tokens[0].token, Token::Number(1.5e10));
    }

    #[test]
    fn test_negative_exponent() {
        let tokens = tokenize("2.5e-3").unwrap();
        assert_eq!(tokens[0].token, Token::Number(2.5e-3));
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("1 + 2 * 3").unwrap();
        assert_eq!(tokens[0].token, Token::Number(1.0));
        assert_eq!(tokens[1].token, Token::Plus);
        assert_eq!(tokens[2].token, Token::Number(2.0));
        assert_eq!(tokens[3].token, Token::Star);
        assert_eq!(tokens[4].token, Token::Number(3.0));
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("a >= b").unwrap();
        assert_eq!(tokens[0].token, Token::Identifier("a".to_string()));
        assert_eq!(tokens[1].token, Token::GreaterEqual);
        assert_eq!(tokens[2].token, Token::Identifier("b".to_string()));
    }

    #[test]
    fn test_arrow_token() {
        let tokens = tokenize("5 m -> ft").unwrap();
        assert_eq!(tokens[0].token, Token::Number(5.0));
        assert_eq!(tokens[1].token, Token::Identifier("m".to_string()));
        assert_eq!(tokens[2].token, Token::Arrow);
        assert_eq!(tokens[3].token, Token::Identifier("ft".to_string()));
    }

    #[test]
    fn test_minus_not_arrow() {
        let tokens = tokenize("5 - 3").unwrap();
        assert_eq!(tokens[1].token, Token::Minus);
        assert_eq!(tokens[2].token, Token::Number(3.0));
    }

    #[test]
    fn test_logical_operators() {
        let tokens = tokenize("true && false || true").unwrap();
        assert_eq!(tokens[0].token, Token::Boolean(true));
        assert_eq!(tokens[1].token, Token::AndAnd);
        assert_eq!(tokens[2].token, Token::Boolean(false));
        assert_eq!(tokens[3].token, Token::OrOr);
        assert_eq!(tokens[4].token, Token::Boolean(true));
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("sin(x) + cos(y)").unwrap();
        assert_eq!(tokens[0].token, Token::Identifier("sin".to_string()));
        assert_eq!(tokens[1].token, Token::LeftParen);
        assert_eq!(tokens[2].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[3].token, Token::RightParen);
        assert_eq!(tokens[4].token, Token::Plus);
        assert_eq!(tokens[5].token, Token::Identifier("cos".to_string()));
    }

    #[test]
    fn test_assignment() {
        let tokens = tokenize("x = 5").unwrap();
        assert_eq!(tokens[0].token, Token::Identifier("x".to_string()));
        assert_eq!(tokens[1].token, Token::Equal);
        assert_eq!(tokens[2].token, Token::Number(5.0));
    }

    #[test]
    fn test_error_unexpected_char() {
        let result = tokenize("@");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token, Token::Eof);
    }

    #[test]
    fn test_complex_expression() {
        let tokens = tokenize("3 * (4 + 5) ^ 2").unwrap();
        assert_eq!(tokens[0].token, Token::Number(3.0));
        assert_eq!(tokens[1].token, Token::Star);
        assert_eq!(tokens[2].token, Token::LeftParen);
        assert_eq!(tokens[3].token, Token::Number(4.0));
        assert_eq!(tokens[4].token, Token::Plus);
        assert_eq!(tokens[5].token, Token::Number(5.0));
        assert_eq!(tokens[6].token, Token::RightParen);
        assert_eq!(tokens[7].token, Token::Caret);
        assert_eq!(tokens[8].token, Token::Number(2.0));
    }
}
