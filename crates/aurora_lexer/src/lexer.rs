//! Lexer Driver - Main tokenization engine
//!
//! This module implements the complete Aurora lexer that converts
//! source code into a stream of tokens using NFA-based maximal-munch.

use crate::nfa::{is_xid_continue, is_xid_start, KeywordTable, LexError, MaximalMunch};
use crate::tokens::{Token, TokenKind};

/// Aurora Lexer
///
/// Converts source code into a stream of tokens using table-driven NFA
/// with maximal-munch tokenization.
pub struct Lexer {
    /// Source code
    source: Vec<char>,
    /// Current position
    pos: usize,
    /// Current line (1-indexed)
    line: usize,
    /// Current column (1-indexed)
    column: usize,
    /// Source file name
    file: String,
    /// Keyword lookup table
    keywords: KeywordTable,
}

impl Lexer {
    /// Create a new lexer from source code
    pub fn new(source: &str, file: String) -> Result<Self, LexError> {
        // Validate UTF-8 (already done by Rust string, but we're explicit)
        let source: Vec<char> = source.chars().collect();

        Ok(Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
            file,
            keywords: KeywordTable::new(),
        })
    }

    /// Get the next token from the source
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Skip whitespace
        self.skip_whitespace();

        // Check for EOF
        if self.is_at_end() {
            return Ok(Token::eof(self.file.clone(), self.line, self.column));
        }

        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.pos;

        let c = self.peek();

        // Comments
        if c == '/' && self.peek_ahead(1) == Some('/') {
            return self.lex_line_comment(start_line, start_column);
        }
        if c == '/' && self.peek_ahead(1) == Some('*') {
            return self.lex_block_comment(start_line, start_column);
        }

        // Raw string literals (check BEFORE identifiers since 'r' is valid identifier start)
        if c == 'r' && self.peek_ahead(1) == Some('"') {
            return self.lex_raw_string(start_line, start_column);
        }

        // Identifiers and keywords
        if is_xid_start(c) {
            return self.lex_identifier(start_line, start_column);
        }

        // Numbers
        if c.is_ascii_digit() {
            return self.lex_number(start_line, start_column);
        }

        // String literals
        if c == '"' {
            return self.lex_string(start_line, start_column);
        }

        // Character literals
        if c == '\'' {
            return self.lex_char(start_line, start_column);
        }

        // Operators (with maximal-munch)
        if let Some((kind, len)) = MaximalMunch::match_operator(self.remaining_str()) {
            let lexeme: String = self.source[self.pos..self.pos + len].iter().collect();
            self.advance_n(len);
            return Ok(Token::new(kind, lexeme, self.file.clone(), start_line, start_column));
        }

        // Delimiters
        if let Some(kind) = MaximalMunch::match_delimiter(c) {
            let lexeme = c.to_string();
            self.advance();
            return Ok(Token::new(kind, lexeme, self.file.clone(), start_line, start_column));
        }

        // If we reach here, it's an invalid character
        Err(LexError::InvalidChar(c, start_pos))
    }

    /// Lex all tokens from source
    pub fn lex_all(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    /// Lex an identifier or keyword
    fn lex_identifier(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;

        // First character already validated as XID_Start
        self.advance();

        // Continue with XID_Continue characters
        while !self.is_at_end() && is_xid_continue(self.peek()) {
            self.advance();
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();

        // Check if it's a keyword
        let kind = self.keywords.lookup(&lexeme).unwrap_or(TokenKind::Ident);

        Ok(Token::new(kind, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a number (integer or float)
    fn lex_number(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;

        // Handle hex, binary, octal prefixes
        if self.peek() == '0' {
            if let Some(next) = self.peek_ahead(1) {
                match next {
                    'x' | 'X' => return self.lex_hex_number(start_line, start_column),
                    'b' | 'B' => return self.lex_binary_number(start_line, start_column),
                    'o' | 'O' => return self.lex_octal_number(start_line, start_column),
                    _ => {}
                }
            }
        }

        // Decimal number
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
            self.advance();
        }

        // Check for float (decimal point followed by digits)
        let mut is_float = false;
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to distinguish from range operator
            if let Some(next) = self.peek_ahead(1) {
                if next.is_ascii_digit() {
                    is_float = true;
                    self.advance(); // consume '.'
                    while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                        self.advance();
                    }
                }
            }
        }

        // Check for exponent
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            is_float = true;
            self.advance();
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                self.advance();
            }
            while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '_') {
                self.advance();
            }
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        let kind = if is_float {
            TokenKind::FloatLiteral
        } else {
            TokenKind::IntLiteral
        };

        Ok(Token::new(kind, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a hexadecimal number
    fn lex_hex_number(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // '0'
        self.advance(); // 'x' or 'X'

        while !self.is_at_end() && (self.peek().is_ascii_hexdigit() || self.peek() == '_') {
            self.advance();
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::IntLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a binary number
    fn lex_binary_number(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // '0'
        self.advance(); // 'b' or 'B'

        while !self.is_at_end() && (matches!(self.peek(), '0' | '1' | '_')) {
            self.advance();
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::IntLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex an octal number
    fn lex_octal_number(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // '0'
        self.advance(); // 'o' or 'O'

        while !self.is_at_end() && (matches!(self.peek(), '0'..='7' | '_')) {
            self.advance();
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::IntLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a string literal
    fn lex_string(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // opening '"'

        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance(); // escape character
                if !self.is_at_end() {
                    self.advance(); // escaped character
                }
            } else {
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(start_pos));
        }

        self.advance(); // closing '"'

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::StringLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a raw string literal
    fn lex_raw_string(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // 'r'
        self.advance(); // '"'

        while !self.is_at_end() && self.peek() != '"' {
            self.advance();
        }

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(start_pos));
        }

        self.advance(); // closing '"'

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::RawStringLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a character literal
    fn lex_char(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // opening '\''

        if self.is_at_end() {
            return Err(LexError::UnterminatedString(start_pos));
        }

        if self.peek() == '\\' {
            self.advance(); // escape
            if !self.is_at_end() {
                self.advance(); // escaped char
            }
        } else {
            self.advance(); // the character
        }

        if self.is_at_end() || self.peek() != '\'' {
            return Err(LexError::UnterminatedString(start_pos));
        }

        self.advance(); // closing '\''

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::CharLiteral, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a line comment
    fn lex_line_comment(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // '/'
        self.advance(); // '/'

        // Check for doc comment
        let is_doc = !self.is_at_end() && (self.peek() == '/' || self.peek() == '!');
        let kind = if is_doc {
            if self.peek() == '!' {
                TokenKind::DocCommentInner
            } else {
                TokenKind::DocCommentOuter
            }
        } else {
            TokenKind::LineComment
        };

        // Consume until end of line
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(kind, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Lex a block comment
    fn lex_block_comment(&mut self, start_line: usize, start_column: usize) -> Result<Token, LexError> {
        let start_pos = self.pos;
        self.advance(); // '/'
        self.advance(); // '*'

        let mut depth = 1;

        while !self.is_at_end() && depth > 0 {
            if self.peek() == '/' && self.peek_ahead(1) == Some('*') {
                depth += 1;
                self.advance();
                self.advance();
            } else if self.peek() == '*' && self.peek_ahead(1) == Some('/') {
                depth -= 1;
                self.advance();
                self.advance();
            } else {
                self.advance();
            }
        }

        if depth > 0 {
            return Err(LexError::UnterminatedBlockComment(start_pos));
        }

        let lexeme: String = self.source[start_pos..self.pos].iter().collect();
        Ok(Token::new(TokenKind::BlockComment, lexeme, self.file.clone(), start_line, start_column))
    }

    /// Skip whitespace
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\t' | '\r' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                _ => break,
            }
        }
    }

    /// Peek at current character
    fn peek(&self) -> char {
        self.source[self.pos]
    }

    /// Peek ahead n characters
    fn peek_ahead(&self, n: usize) -> Option<char> {
        let pos = self.pos + n;
        if pos < self.source.len() {
            Some(self.source[pos])
        } else {
            None
        }
    }

    /// Get remaining source as string (for operator matching)
    fn remaining_str(&self) -> &str {
        // This is a bit hacky but works for our purposes
        static mut BUFFER: String = String::new();
        unsafe {
            BUFFER.clear();
            BUFFER.extend(self.source[self.pos..].iter());
            &BUFFER
        }
    }

    /// Advance by one character
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
            self.column += 1;
        }
    }

    /// Advance by n characters
    fn advance_n(&mut self, n: usize) {
        for _ in 0..n {
            self.advance();
        }
    }

    /// Check if at end of source
    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_keywords() {
        let source = "fn if else while";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::If);
        assert_eq!(tokens[2].kind, TokenKind::Else);
        assert_eq!(tokens[3].kind, TokenKind::While);
        assert_eq!(tokens[4].kind, TokenKind::Eof);
    }

    #[test]
    fn test_lexer_identifiers() {
        let source = "foo bar_baz hello123";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].lexeme, "foo");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].lexeme, "bar_baz");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].lexeme, "hello123");
    }

    #[test]
    fn test_lexer_numbers() {
        let source = "42 3.14 0xFF 0b1010";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "3.14");
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[2].lexeme, "0xFF");
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[3].lexeme, "0b1010");
    }

    #[test]
    fn test_lexer_strings() {
        let source = r#""hello" r"raw string" 'c'"#;
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[1].kind, TokenKind::RawStringLiteral);
        assert_eq!(tokens[2].kind, TokenKind::CharLiteral);
    }

    #[test]
    fn test_lexer_operators() {
        let source = "+ == != .. ... ..=";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::EqEq);
        assert_eq!(tokens[2].kind, TokenKind::NotEq);
        assert_eq!(tokens[3].kind, TokenKind::DotDot);
        assert_eq!(tokens[4].kind, TokenKind::DotDotDot);
        assert_eq!(tokens[5].kind, TokenKind::DotDotEq);
    }

    #[test]
    fn test_lexer_comments() {
        let source = "// line comment\n/* block comment */";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::LineComment);
        assert_eq!(tokens[1].kind, TokenKind::BlockComment);
    }

    #[test]
    fn test_lexer_full_program() {
        let source = r#"
            fn main() {
                let x = 42;
                if x > 10 {
                    println("Hello");
                }
            }
        "#;
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        // Should successfully tokenize a complete program
        assert!(tokens.len() > 10);
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }
}
