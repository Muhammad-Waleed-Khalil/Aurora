//! NFA State Machine for Aurora Lexer
//!
//! Implements a table-driven Non-deterministic Finite Automaton
//! with maximal-munch tokenization and UTF-8 validation.

use crate::tokens::TokenKind;
use std::collections::HashMap;
use thiserror::Error;

/// Lexer error types
#[derive(Debug, Clone, Error, PartialEq)]
pub enum LexError {
    /// Invalid UTF-8 sequence
    #[error("Invalid UTF-8 sequence at position {0}")]
    InvalidUtf8(usize),

    /// Invalid character
    #[error("Invalid character '{0}' at position {1}")]
    InvalidChar(char, usize),

    /// Unterminated string literal
    #[error("Unterminated string literal starting at position {0}")]
    UnterminatedString(usize),

    /// Unterminated block comment
    #[error("Unterminated block comment starting at position {0}")]
    UnterminatedBlockComment(usize),

    /// Invalid number literal
    #[error("Invalid number literal at position {0}")]
    InvalidNumber(usize),

    /// Unexpected end of file
    #[error("Unexpected end of file")]
    UnexpectedEof,
}

/// NFA state for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    /// Start state
    Start,
    /// Identifier or keyword
    Ident,
    /// Integer literal
    IntLiteral,
    /// Float literal (saw decimal point)
    FloatLiteral,
    /// String literal
    StringLiteral,
    /// Raw string literal
    RawStringLiteral,
    /// Character literal
    CharLiteral,
    /// Operator
    Operator,
    /// Comment
    Comment,
    /// Whitespace
    Whitespace,
    /// Accept state with token kind
    Accept(TokenKind),
}

/// Character classification for fast lexing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharClass {
    /// Alphabetic or underscore (identifier start)
    Alpha,
    /// Digit
    Digit,
    /// Whitespace
    Whitespace,
    /// Newline
    Newline,
    /// Operator character
    Operator,
    /// Delimiter
    Delimiter,
    /// Quote
    Quote,
    /// Other
    Other,
}

impl CharClass {
    /// Classify a character
    pub fn classify(c: char) -> Self {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => CharClass::Alpha,
            '0'..='9' => CharClass::Digit,
            ' ' | '\t' | '\r' => CharClass::Whitespace,
            '\n' => CharClass::Newline,
            '+' | '-' | '*' | '/' | '%' | '=' | '!' | '<' | '>' | '&' | '|' | '^' | '~' | '?' => {
                CharClass::Operator
            }
            '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';' | ':' | '.' => CharClass::Delimiter,
            '"' | '\'' | '`' => CharClass::Quote,
            _ => CharClass::Other,
        }
    }
}

/// Check if character is XID_Start (Unicode identifier start)
pub fn is_xid_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || (c > '\u{00A0}' && unicode_ident::is_xid_start(c))
}

/// Check if character is XID_Continue (Unicode identifier continue)
pub fn is_xid_continue(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || (c > '\u{00A0}' && unicode_ident::is_xid_continue(c))
}

/// Keyword lookup table for reserved word priority
pub struct KeywordTable {
    keywords: HashMap<&'static str, TokenKind>,
}

impl KeywordTable {
    /// Create a new keyword table
    pub fn new() -> Self {
        let mut keywords = HashMap::new();
        // Control flow keywords
        keywords.insert("if", TokenKind::If);
        keywords.insert("else", TokenKind::Else);
        keywords.insert("match", TokenKind::Match);
        keywords.insert("for", TokenKind::For);
        keywords.insert("while", TokenKind::While);
        keywords.insert("loop", TokenKind::Loop);
        keywords.insert("break", TokenKind::Break);
        keywords.insert("continue", TokenKind::Continue);
        keywords.insert("return", TokenKind::Return);
        keywords.insert("yield", TokenKind::Yield);

        // Declaration keywords
        keywords.insert("fn", TokenKind::Fn);
        keywords.insert("let", TokenKind::Let);
        keywords.insert("mut", TokenKind::Mut);
        keywords.insert("const", TokenKind::Const);
        keywords.insert("static", TokenKind::Static);
        keywords.insert("type", TokenKind::Type);
        keywords.insert("trait", TokenKind::Trait);
        keywords.insert("impl", TokenKind::Impl);
        keywords.insert("where", TokenKind::Where);
        keywords.insert("in", TokenKind::In);

        // Module keywords
        keywords.insert("use", TokenKind::Use);
        keywords.insert("mod", TokenKind::Mod);
        keywords.insert("pub", TokenKind::Pub);
        keywords.insert("as", TokenKind::As);

        // Self keywords
        keywords.insert("self", TokenKind::SelfLower);
        keywords.insert("Self", TokenKind::SelfUpper);
        keywords.insert("super", TokenKind::Super);
        keywords.insert("crate", TokenKind::Crate);

        // Async keywords
        keywords.insert("async", TokenKind::Async);
        keywords.insert("await", TokenKind::Await);

        // Memory/effect keywords
        keywords.insert("defer", TokenKind::Defer);
        keywords.insert("unsafe", TokenKind::Unsafe);
        keywords.insert("comptime", TokenKind::Comptime);

        // Boolean literals
        keywords.insert("true", TokenKind::True);
        keywords.insert("false", TokenKind::False);

        // Primitive types
        keywords.insert("i8", TokenKind::I8);
        keywords.insert("i16", TokenKind::I16);
        keywords.insert("i32", TokenKind::I32);
        keywords.insert("i64", TokenKind::I64);
        keywords.insert("i128", TokenKind::I128);
        keywords.insert("u8", TokenKind::U8);
        keywords.insert("u16", TokenKind::U16);
        keywords.insert("u32", TokenKind::U32);
        keywords.insert("u64", TokenKind::U64);
        keywords.insert("u128", TokenKind::U128);
        keywords.insert("f32", TokenKind::F32);
        keywords.insert("f64", TokenKind::F64);
        keywords.insert("bool", TokenKind::Bool);
        keywords.insert("char", TokenKind::Char);
        keywords.insert("str", TokenKind::Str);

        // Option/Result variants
        keywords.insert("Some", TokenKind::Some);
        keywords.insert("None", TokenKind::None);
        keywords.insert("Ok", TokenKind::Ok);
        keywords.insert("Err", TokenKind::Err);
        keywords.insert("unreachable", TokenKind::Unreachable);

        Self { keywords }
    }

    /// Lookup a keyword, returns TokenKind if found
    pub fn lookup(&self, ident: &str) -> Option<TokenKind> {
        self.keywords.get(ident).copied()
    }
}

impl Default for KeywordTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximal-munch tokenizer helper
///
/// This struct helps implement maximal-munch: always prefer the longest match.
/// For example, ".." should be recognized as DotDot, not two Dot tokens.
pub struct MaximalMunch {
    /// Current accumulated characters
    buffer: String,
    /// Best match so far (longest)
    best_match: Option<(TokenKind, usize)>,
}

impl MaximalMunch {
    /// Create new maximal-munch helper
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            best_match: None,
        }
    }

    /// Try to match operators with maximal-munch
    ///
    /// Returns (TokenKind, length) for the longest match
    pub fn match_operator(s: &str) -> Option<(TokenKind, usize)> {
        // Longest matches first (maximal-munch)
        if s.starts_with("<<=") {
            return Some((TokenKind::LtLtEq, 3));
        }
        if s.starts_with(">>=") {
            return Some((TokenKind::GtGtEq, 3));
        }
        if s.starts_with("...") {
            return Some((TokenKind::DotDotDot, 3));
        }

        // Two-character operators
        if s.starts_with("**") {
            return Some((TokenKind::StarStar, 2));
        }
        if s.starts_with("==") {
            return Some((TokenKind::EqEq, 2));
        }
        if s.starts_with("!=") {
            return Some((TokenKind::NotEq, 2));
        }
        if s.starts_with("<=") {
            return Some((TokenKind::LtEq, 2));
        }
        if s.starts_with(">=") {
            return Some((TokenKind::GtEq, 2));
        }
        if s.starts_with("&&") {
            return Some((TokenKind::AndAnd, 2));
        }
        if s.starts_with("||") {
            return Some((TokenKind::OrOr, 2));
        }
        if s.starts_with("<<") {
            return Some((TokenKind::LtLt, 2));
        }
        if s.starts_with(">>") {
            return Some((TokenKind::GtGt, 2));
        }
        if s.starts_with("+=") {
            return Some((TokenKind::PlusEq, 2));
        }
        if s.starts_with("-=") {
            return Some((TokenKind::MinusEq, 2));
        }
        if s.starts_with("*=") {
            return Some((TokenKind::StarEq, 2));
        }
        if s.starts_with("/=") {
            return Some((TokenKind::SlashEq, 2));
        }
        if s.starts_with("%=") {
            return Some((TokenKind::PercentEq, 2));
        }
        if s.starts_with("&=") {
            return Some((TokenKind::AndEq, 2));
        }
        if s.starts_with("|=") {
            return Some((TokenKind::OrEq, 2));
        }
        if s.starts_with("^=") {
            return Some((TokenKind::CaretEq, 2));
        }
        if s.starts_with("..=") {
            return Some((TokenKind::DotDotEq, 3));
        }
        if s.starts_with("..") {
            return Some((TokenKind::DotDot, 2));
        }
        if s.starts_with("::") {
            return Some((TokenKind::ColonColon, 2));
        }
        if s.starts_with("->") {
            return Some((TokenKind::RArrow, 2));
        }
        if s.starts_with("=>") {
            return Some((TokenKind::FatArrow, 2));
        }
        if s.starts_with("??") {
            return Some((TokenKind::QuestionQuestion, 2));
        }
        if s.starts_with("|>") {
            return Some((TokenKind::PipeGt, 2));
        }
        if s.starts_with("<|") {
            return Some((TokenKind::LtPipe, 2));
        }

        // Single-character operators
        if s.starts_with('+') {
            return Some((TokenKind::Plus, 1));
        }
        if s.starts_with('-') {
            return Some((TokenKind::Minus, 1));
        }
        if s.starts_with('*') {
            return Some((TokenKind::Star, 1));
        }
        if s.starts_with('/') {
            return Some((TokenKind::Slash, 1));
        }
        if s.starts_with('%') {
            return Some((TokenKind::Percent, 1));
        }
        if s.starts_with('=') {
            return Some((TokenKind::Eq, 1));
        }
        if s.starts_with('!') {
            return Some((TokenKind::Not, 1));
        }
        if s.starts_with('<') {
            return Some((TokenKind::Lt, 1));
        }
        if s.starts_with('>') {
            return Some((TokenKind::Gt, 1));
        }
        if s.starts_with('&') {
            return Some((TokenKind::And, 1));
        }
        if s.starts_with('|') {
            return Some((TokenKind::Or, 1));
        }
        if s.starts_with('^') {
            return Some((TokenKind::Caret, 1));
        }
        if s.starts_with('~') {
            return Some((TokenKind::Tilde, 1));
        }
        if s.starts_with('?') {
            return Some((TokenKind::Question, 1));
        }
        if s.starts_with('.') {
            return Some((TokenKind::Dot, 1));
        }

        None
    }

    /// Match delimiters
    pub fn match_delimiter(c: char) -> Option<TokenKind> {
        match c {
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            '{' => Some(TokenKind::LBrace),
            '}' => Some(TokenKind::RBrace),
            '[' => Some(TokenKind::LBracket),
            ']' => Some(TokenKind::RBracket),
            ',' => Some(TokenKind::Comma),
            ';' => Some(TokenKind::Semicolon),
            ':' => Some(TokenKind::Colon),
            _ => None,
        }
    }
}

impl Default for MaximalMunch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_classification() {
        assert_eq!(CharClass::classify('a'), CharClass::Alpha);
        assert_eq!(CharClass::classify('5'), CharClass::Digit);
        assert_eq!(CharClass::classify(' '), CharClass::Whitespace);
        assert_eq!(CharClass::classify('\n'), CharClass::Newline);
        assert_eq!(CharClass::classify('+'), CharClass::Operator);
        assert_eq!(CharClass::classify('('), CharClass::Delimiter);
        assert_eq!(CharClass::classify('"'), CharClass::Quote);
    }

    #[test]
    fn test_xid_identifiers() {
        assert!(is_xid_start('a'));
        assert!(is_xid_start('_'));
        assert!(!is_xid_start('5'));
        assert!(is_xid_continue('a'));
        assert!(is_xid_continue('5'));
        assert!(is_xid_continue('_'));
    }

    #[test]
    fn test_keyword_lookup() {
        let table = KeywordTable::new();
        assert_eq!(table.lookup("if"), Some(TokenKind::If));
        assert_eq!(table.lookup("fn"), Some(TokenKind::Fn));
        assert_eq!(table.lookup("not_a_keyword"), None);
    }

    #[test]
    fn test_maximal_munch_operators() {
        // Three-character operators
        assert_eq!(
            MaximalMunch::match_operator("..."),
            Some((TokenKind::DotDotDot, 3))
        );
        assert_eq!(
            MaximalMunch::match_operator("<<="),
            Some((TokenKind::LtLtEq, 3))
        );

        // Two-character operators (maximal-munch over single-char)
        assert_eq!(
            MaximalMunch::match_operator("=="),
            Some((TokenKind::EqEq, 2))
        );
        assert_eq!(
            MaximalMunch::match_operator("..="),
            Some((TokenKind::DotDotEq, 3))
        );
        assert_eq!(
            MaximalMunch::match_operator(".."),
            Some((TokenKind::DotDot, 2))
        );

        // Single-character operators
        assert_eq!(
            MaximalMunch::match_operator("+"),
            Some((TokenKind::Plus, 1))
        );
        assert_eq!(MaximalMunch::match_operator("."), Some((TokenKind::Dot, 1)));
    }

    #[test]
    fn test_delimiter_matching() {
        assert_eq!(MaximalMunch::match_delimiter('('), Some(TokenKind::LParen));
        assert_eq!(MaximalMunch::match_delimiter(')'), Some(TokenKind::RParen));
        assert_eq!(MaximalMunch::match_delimiter('{'), Some(TokenKind::LBrace));
        assert_eq!(MaximalMunch::match_delimiter('a'), None);
    }

    #[test]
    fn test_maximal_munch_priority() {
        // ".." should be recognized as DotDot, not two Dot tokens
        let s = "..";
        assert_eq!(
            MaximalMunch::match_operator(s),
            Some((TokenKind::DotDot, 2))
        );

        // "..." should be DotDotDot, not DotDot + Dot
        let s = "...";
        assert_eq!(
            MaximalMunch::match_operator(s),
            Some((TokenKind::DotDotDot, 3))
        );

        // "..=" should be DotDotEq
        let s = "..=";
        assert_eq!(
            MaximalMunch::match_operator(s),
            Some((TokenKind::DotDotEq, 3))
        );
    }
}
