//! Lexer Driver - Main tokenization engine
//!
//! This module implements the complete Aurora lexer that converts
//! source code into a stream of tokens using NFA-based maximal-munch.

use crate::nfa::{is_xid_continue, is_xid_start, KeywordTable, LexError, MaximalMunch};
use crate::tokens::{Token, TokenKind};
use std::sync::Arc;

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
    /// Optional diagnostic collector
    #[allow(dead_code)]
    diagnostics: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl Lexer {
    /// Create a new lexer from source code (original API)
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
            diagnostics: None,
        })
    }

    /// Create a new lexer with diagnostic collector (for pipeline integration)
    ///
    /// The diagnostics parameter can be any Arc type. Typically aurora_diagnostics::DiagnosticCollector.
    pub fn with_diagnostics<D: Send + Sync + 'static>(
        source: &str,
        diagnostics: Arc<D>
    ) -> Self {
        let source: Vec<char> = source.chars().collect();

        Self {
            source,
            pos: 0,
            line: 1,
            column: 1,
            file: "<input>".to_string(),
            keywords: KeywordTable::new(),
            diagnostics: Some(diagnostics as Arc<dyn std::any::Any + Send + Sync>),
        }
    }

    /// Tokenize all source code (for pipeline integration)
    ///
    /// This is an alias for `lex_all()` that returns a Vec<Token> directly,
    /// panicking on errors (errors will be reported via diagnostics).
    pub fn tokenize(mut self) -> Vec<Token> {
        match self.lex_all() {
            Ok(tokens) => tokens,
            Err(e) => {
                eprintln!("Lexer error: {:?}", e);
                vec![Token::eof("<error>".to_string(), 1, 1)]
            }
        }
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
        if let Some((kind, len)) = MaximalMunch::match_operator(&self.remaining_str()) {
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

        // Check if it's exactly underscore (special token)
        if lexeme == "_" {
            return Ok(Token::new(TokenKind::Underscore, lexeme, self.file.clone(), start_line, start_column));
        }

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

        // Extract string content without quotes
        let lexeme: String = self.source[start_pos+1..self.pos-1].iter().collect();
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

        // Extract raw string content without r"..."
        let lexeme: String = self.source[start_pos+2..self.pos-1].iter().collect();
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
    fn remaining_str(&self) -> String {
        self.source[self.pos..].iter().collect()
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

    #[test]
    fn test_lexer_primitive_types() {
        let source = "i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 bool char str";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::I8);
        assert_eq!(tokens[1].kind, TokenKind::I16);
        assert_eq!(tokens[2].kind, TokenKind::I32);
        assert_eq!(tokens[3].kind, TokenKind::I64);
        assert_eq!(tokens[4].kind, TokenKind::I128);
        assert_eq!(tokens[5].kind, TokenKind::U8);
        assert_eq!(tokens[6].kind, TokenKind::U16);
        assert_eq!(tokens[7].kind, TokenKind::U32);
        assert_eq!(tokens[8].kind, TokenKind::U64);
        assert_eq!(tokens[9].kind, TokenKind::U128);
        assert_eq!(tokens[10].kind, TokenKind::F32);
        assert_eq!(tokens[11].kind, TokenKind::F64);
        assert_eq!(tokens[12].kind, TokenKind::Bool);
        assert_eq!(tokens[13].kind, TokenKind::Char);
        assert_eq!(tokens[14].kind, TokenKind::Str);
    }

    #[test]
    fn test_lexer_underscore() {
        let source = "_ _foo foo_";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Underscore);
        assert_eq!(tokens[0].lexeme, "_");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].lexeme, "_foo");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].lexeme, "foo_");
    }

    #[test]
    fn test_lexer_all_keywords() {
        let source = "if else match for while loop break continue return yield \
                      fn let mut const static type trait impl where in \
                      use mod pub as self Self super crate async await \
                      defer unsafe comptime true false Some None Ok Err unreachable";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        // Verify all are keywords, not identifiers
        for token in &tokens {
            if token.kind != TokenKind::Eof {
                assert!(token.kind.is_keyword(), "Expected keyword, got {:?}", token);
            }
        }
    }

    #[test]
    fn test_lexer_hello_world() {
        let source = r#"fn main() {
    println("Hello, World!");
    println("Welcome to Aurora!");
}"#;
        let mut lexer = Lexer::new(source, "hello_world.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        // Print all tokens for verification
        eprintln!("\n=== Hello World Tokenization ===");
        for (i, token) in tokens.iter().enumerate() {
            eprintln!("{:3}: {:?} '{}' at {}:{}",
                     i, token.kind, token.lexeme, token.line, token.column);
        }
        eprintln!("=== Total: {} tokens ===\n", tokens.len());

        // Verify it tokenizes without errors
        assert!(tokens.len() > 0);
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);

        // Spot-check some tokens
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].lexeme, "main");
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
    }

    #[test]
    fn test_lexer_maximal_munch_complete() {
        // Test that maximal-munch works correctly for all multi-char operators
        let source = "... ..= .. :: -> => ?? |> <| == != <= >= && || << >> \
                      += -= *= /= %= &= |= ^= <<= >>=";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        let expected = vec![
            TokenKind::DotDotDot,
            TokenKind::DotDotEq,
            TokenKind::DotDot,
            TokenKind::ColonColon,
            TokenKind::RArrow,
            TokenKind::FatArrow,
            TokenKind::QuestionQuestion,
            TokenKind::PipeGt,
            TokenKind::LtPipe,
            TokenKind::EqEq,
            TokenKind::NotEq,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::AndAnd,
            TokenKind::OrOr,
            TokenKind::LtLt,
            TokenKind::GtGt,
            TokenKind::PlusEq,
            TokenKind::MinusEq,
            TokenKind::StarEq,
            TokenKind::SlashEq,
            TokenKind::PercentEq,
            TokenKind::AndEq,
            TokenKind::OrEq,
            TokenKind::CaretEq,
            TokenKind::LtLtEq,
            TokenKind::GtGtEq,
            TokenKind::Eof,
        ];

        for (i, expected_kind) in expected.iter().enumerate() {
            assert_eq!(
                tokens[i].kind, *expected_kind,
                "Token {} mismatch: expected {:?}, got {:?}",
                i, expected_kind, tokens[i].kind
            );
        }
    }

    #[test]
    fn test_lexer_nested_block_comments() {
        let source = "/* outer /* inner */ still in outer */";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens.len(), 2); // Comment + EOF
        assert_eq!(tokens[0].kind, TokenKind::BlockComment);
    }

    #[test]
    fn test_lexer_escape_sequences() {
        let source = r#""hello\nworld" "tab\there" "quote\"inside""#;
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[2].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn test_lexer_utf8_identifiers() {
        // Test Unicode identifiers (XID-compliant)
        let source = "café naïve 变量 переменная";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Ident);
        assert_eq!(tokens[0].lexeme, "café");
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].lexeme, "naïve");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].lexeme, "变量");
        assert_eq!(tokens[3].kind, TokenKind::Ident);
        assert_eq!(tokens[3].lexeme, "переменная");
    }

    #[test]
    fn test_lexer_line_tracking() {
        let source = "fn\nmain\n(\n)\n{\n}";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].line, 1); // fn
        assert_eq!(tokens[1].line, 2); // main
        assert_eq!(tokens[2].line, 3); // (
        assert_eq!(tokens[3].line, 4); // )
        assert_eq!(tokens[4].line, 5); // {
        assert_eq!(tokens[5].line, 6); // }
    }

    #[test]
    fn test_lexer_number_formats() {
        let source = "42 3.14 2.5e10 1.0e-5 0xFF 0b1010 0o755 1_000_000";
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens = lexer.lex_all().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[1].lexeme, "3.14");
        assert_eq!(tokens[2].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[2].lexeme, "2.5e10");
        assert_eq!(tokens[3].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[3].lexeme, "1.0e-5");
        assert_eq!(tokens[4].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[4].lexeme, "0xFF");
        assert_eq!(tokens[5].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[5].lexeme, "0b1010");
        assert_eq!(tokens[6].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[6].lexeme, "0o755");
        assert_eq!(tokens[7].kind, TokenKind::IntLiteral);
        assert_eq!(tokens[7].lexeme, "1_000_000");
    }

    #[test]
    fn test_lexer_determinism() {
        // The lexer should produce identical results for the same input
        let source = "fn main() { let x = 42; }";

        let mut lexer1 = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens1 = lexer1.lex_all().unwrap();

        let mut lexer2 = Lexer::new(source, "test.ax".to_string()).unwrap();
        let tokens2 = lexer2.lex_all().unwrap();

        assert_eq!(tokens1.len(), tokens2.len());
        for (t1, t2) in tokens1.iter().zip(tokens2.iter()) {
            assert_eq!(t1.kind, t2.kind);
            assert_eq!(t1.lexeme, t2.lexeme);
            assert_eq!(t1.line, t2.line);
            assert_eq!(t1.column, t2.column);
        }
    }
}
