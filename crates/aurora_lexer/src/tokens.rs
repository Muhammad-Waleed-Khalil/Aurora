//! Token Definitions for Aurora
//!
//! This module defines all token types recognized by the Aurora lexer.
//! The token catalog is machine-readable and follows the specification.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Token kind enumeration
///
/// All Aurora tokens are defined here. This enum is serializable for
/// machine-readable token catalog export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenKind {
    // Keywords (control flow)
    If,
    Else,
    Elif,       // elif (simplified syntax)
    Match,
    For,
    While,
    Loop,
    Break,
    Continue,
    Return,
    Ret,        // ret (simplified syntax)
    Yield,

    // Keywords (declarations)
    Fn,
    Fun,        // fun (simplified syntax)
    Let,
    Mut,
    Var,        // var (simplified syntax)
    Const,
    Static,
    Type,
    Trait,
    Impl,
    Where,
    In,

    // Keywords (special statements)
    Print,      // print statement (simplified syntax)

    // Keywords (modules and visibility)
    Use,
    Mod,
    Pub,
    As,

    // Keywords (self and types)
    SelfLower,  // self
    SelfUpper,  // Self
    Super,
    Crate,

    // Keywords (async/concurrency)
    Async,
    Await,

    // Keywords (memory and effects)
    Defer,
    Unsafe,
    Comptime,

    // Literals (boolean)
    True,
    False,
    Yes,        // yes (simplified syntax)
    No,         // no (simplified syntax)

    // Primitive types
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Bool,
    Char,
    Str,

    // Literals (special values)
    Some,
    None,
    Ok,
    Err,
    Unreachable,

    // Identifiers
    Ident,
    Underscore,  // _

    // Literals (numeric)
    IntLiteral,
    FloatLiteral,

    // Literals (string and char)
    StringLiteral,
    RawStringLiteral,
    CharLiteral,

    // Operators (arithmetic)
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    StarStar,   // **

    // Operators (comparison)
    EqEq,       // ==
    NotEq,      // !=
    Lt,         // <
    LtEq,       // <=
    Gt,         // >
    GtEq,       // >=

    // Operators (logical)
    AndAnd,     // &&
    OrOr,       // ||
    Not,        // !

    // Operators (logical word-based - simplified syntax)
    AndKeyword, // and
    OrKeyword,  // or
    NotKeyword, // not

    // Operators (bitwise)
    And,        // &
    Or,         // |
    Caret,      // ^
    LtLt,       // <<
    GtGt,       // >>
    Tilde,      // ~

    // Operators (assignment)
    Eq,         // =
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    AndEq,      // &=
    OrEq,       // |=
    CaretEq,    // ^=
    LtLtEq,     // <<=
    GtGtEq,     // >>=

    // Operators (access and special)
    Dot,        // .
    DotDot,     // ..
    DotDotEq,   // ..=
    DotDotDot,  // ...
    ColonColon, // ::
    RArrow,     // ->
    FatArrow,   // =>
    Question,   // ?
    QuestionQuestion, // ??

    // Operators (pipeline)
    PipeGt,     // |>
    LtPipe,     // <|

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Semicolon,  // ;
    Colon,      // :

    // Comments (tracked for doc extraction)
    LineComment,
    BlockComment,
    DocCommentOuter,  // ///
    DocCommentInner,  // //!

    // Special
    Whitespace,
    Newline,
    Eof,

    // Error token
    Error,
}

impl TokenKind {
    /// Check if this token is a keyword
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Yield
                | TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Const
                | TokenKind::Static
                | TokenKind::Type
                | TokenKind::Trait
                | TokenKind::Impl
                | TokenKind::Where
                | TokenKind::In
                | TokenKind::Use
                | TokenKind::Mod
                | TokenKind::Pub
                | TokenKind::As
                | TokenKind::SelfLower
                | TokenKind::SelfUpper
                | TokenKind::Super
                | TokenKind::Crate
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Defer
                | TokenKind::Unsafe
                | TokenKind::Comptime
                | TokenKind::True
                | TokenKind::False
                | TokenKind::I8
                | TokenKind::I16
                | TokenKind::I32
                | TokenKind::I64
                | TokenKind::I128
                | TokenKind::U8
                | TokenKind::U16
                | TokenKind::U32
                | TokenKind::U64
                | TokenKind::U128
                | TokenKind::F32
                | TokenKind::F64
                | TokenKind::Bool
                | TokenKind::Char
                | TokenKind::Str
                | TokenKind::Some
                | TokenKind::None
                | TokenKind::Ok
                | TokenKind::Err
                | TokenKind::Unreachable
        )
    }

    /// Check if this token is a literal
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            TokenKind::IntLiteral
                | TokenKind::FloatLiteral
                | TokenKind::StringLiteral
                | TokenKind::RawStringLiteral
                | TokenKind::CharLiteral
                | TokenKind::True
                | TokenKind::False
        )
    }

    /// Check if this token is an operator
    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::StarStar
                | TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::LtEq
                | TokenKind::Gt
                | TokenKind::GtEq
                | TokenKind::AndAnd
                | TokenKind::OrOr
                | TokenKind::Not
                | TokenKind::And
                | TokenKind::Or
                | TokenKind::Caret
                | TokenKind::LtLt
                | TokenKind::GtGt
                | TokenKind::Tilde
                | TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
                | TokenKind::PercentEq
                | TokenKind::AndEq
                | TokenKind::OrEq
                | TokenKind::CaretEq
                | TokenKind::LtLtEq
                | TokenKind::GtGtEq
                | TokenKind::Dot
                | TokenKind::DotDot
                | TokenKind::DotDotEq
                | TokenKind::ColonColon
                | TokenKind::RArrow
                | TokenKind::FatArrow
                | TokenKind::Question
                | TokenKind::QuestionQuestion
                | TokenKind::PipeGt
                | TokenKind::LtPipe
        )
    }

    /// Get the string representation of this keyword (if it is one)
    pub fn keyword_str(&self) -> Option<&'static str> {
        match self {
            TokenKind::If => Some("if"),
            TokenKind::Else => Some("else"),
            TokenKind::Match => Some("match"),
            TokenKind::For => Some("for"),
            TokenKind::While => Some("while"),
            TokenKind::Loop => Some("loop"),
            TokenKind::Break => Some("break"),
            TokenKind::Continue => Some("continue"),
            TokenKind::Return => Some("return"),
            TokenKind::Yield => Some("yield"),
            TokenKind::Fn => Some("fn"),
            TokenKind::Let => Some("let"),
            TokenKind::Mut => Some("mut"),
            TokenKind::Const => Some("const"),
            TokenKind::Static => Some("static"),
            TokenKind::Type => Some("type"),
            TokenKind::Trait => Some("trait"),
            TokenKind::Impl => Some("impl"),
            TokenKind::Where => Some("where"),
            TokenKind::In => Some("in"),
            TokenKind::Use => Some("use"),
            TokenKind::Mod => Some("mod"),
            TokenKind::Pub => Some("pub"),
            TokenKind::As => Some("as"),
            TokenKind::SelfLower => Some("self"),
            TokenKind::SelfUpper => Some("Self"),
            TokenKind::Super => Some("super"),
            TokenKind::Crate => Some("crate"),
            TokenKind::Async => Some("async"),
            TokenKind::Await => Some("await"),
            TokenKind::Defer => Some("defer"),
            TokenKind::Unsafe => Some("unsafe"),
            TokenKind::Comptime => Some("comptime"),
            TokenKind::True => Some("true"),
            TokenKind::False => Some("false"),
            TokenKind::I8 => Some("i8"),
            TokenKind::I16 => Some("i16"),
            TokenKind::I32 => Some("i32"),
            TokenKind::I64 => Some("i64"),
            TokenKind::I128 => Some("i128"),
            TokenKind::U8 => Some("u8"),
            TokenKind::U16 => Some("u16"),
            TokenKind::U32 => Some("u32"),
            TokenKind::U64 => Some("u64"),
            TokenKind::U128 => Some("u128"),
            TokenKind::F32 => Some("f32"),
            TokenKind::F64 => Some("f64"),
            TokenKind::Bool => Some("bool"),
            TokenKind::Char => Some("char"),
            TokenKind::Str => Some("str"),
            TokenKind::Some => Some("Some"),
            TokenKind::None => Some("None"),
            TokenKind::Ok => Some("Ok"),
            TokenKind::Err => Some("Err"),
            TokenKind::Unreachable => Some("unreachable"),
            _ => None,
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::StarStar => "**",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::Lt => "<",
            TokenKind::LtEq => "<=",
            TokenKind::Gt => ">",
            TokenKind::GtEq => ">=",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::Not => "!",
            TokenKind::And => "&",
            TokenKind::Or => "|",
            TokenKind::Caret => "^",
            TokenKind::LtLt => "<<",
            TokenKind::GtGt => ">>",
            TokenKind::Tilde => "~",
            TokenKind::Eq => "=",
            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::StarEq => "*=",
            TokenKind::SlashEq => "/=",
            TokenKind::PercentEq => "%=",
            TokenKind::AndEq => "&=",
            TokenKind::OrEq => "|=",
            TokenKind::CaretEq => "^=",
            TokenKind::LtLtEq => "<<=",
            TokenKind::GtGtEq => ">>=",
            TokenKind::Dot => ".",
            TokenKind::DotDot => "..",
            TokenKind::DotDotEq => "..=",
            TokenKind::DotDotDot => "...",
            TokenKind::ColonColon => "::",
            TokenKind::RArrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Question => "?",
            TokenKind::QuestionQuestion => "??",
            TokenKind::PipeGt => "|>",
            TokenKind::LtPipe => "<|",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Comma => ",",
            TokenKind::Semicolon => ";",
            TokenKind::Colon => ":",
            _ => {
                if let Some(kw) = self.keyword_str() {
                    kw
                } else {
                    return write!(f, "{:?}", self);
                }
            }
        };
        write!(f, "{}", s)
    }
}

/// A single token with location information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// Token kind
    pub kind: TokenKind,
    /// Lexeme (the actual text)
    pub lexeme: String,
    /// Source file
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Length in bytes
    pub len: usize,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, lexeme: String, file: String, line: usize, column: usize) -> Self {
        let len = lexeme.len();
        Self {
            kind,
            lexeme,
            file,
            line,
            column,
            len,
        }
    }

    /// Create an EOF token
    pub fn eof(file: String, line: usize, column: usize) -> Self {
        Self {
            kind: TokenKind::Eof,
            lexeme: String::new(),
            file,
            line,
            column,
            len: 0,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?}('{}') at {}:{}:{}",
            self.kind, self.lexeme, self.file, self.line, self.column
        )
    }
}

/// Export token catalog as JSON
pub fn export_token_catalog() -> serde_json::Result<String> {
    let catalog: Vec<(String, TokenKind)> = vec![
        ("if".to_string(), TokenKind::If),
        ("else".to_string(), TokenKind::Else),
        ("match".to_string(), TokenKind::Match),
        ("for".to_string(), TokenKind::For),
        ("while".to_string(), TokenKind::While),
        ("loop".to_string(), TokenKind::Loop),
        ("break".to_string(), TokenKind::Break),
        ("continue".to_string(), TokenKind::Continue),
        ("return".to_string(), TokenKind::Return),
        ("yield".to_string(), TokenKind::Yield),
        ("fn".to_string(), TokenKind::Fn),
        ("let".to_string(), TokenKind::Let),
        ("mut".to_string(), TokenKind::Mut),
        ("const".to_string(), TokenKind::Const),
        ("static".to_string(), TokenKind::Static),
        ("type".to_string(), TokenKind::Type),
        ("trait".to_string(), TokenKind::Trait),
        ("impl".to_string(), TokenKind::Impl),
        ("use".to_string(), TokenKind::Use),
        ("mod".to_string(), TokenKind::Mod),
        ("pub".to_string(), TokenKind::Pub),
        ("as".to_string(), TokenKind::As),
        ("self".to_string(), TokenKind::SelfLower),
        ("Self".to_string(), TokenKind::SelfUpper),
        ("super".to_string(), TokenKind::Super),
        ("crate".to_string(), TokenKind::Crate),
        ("async".to_string(), TokenKind::Async),
        ("await".to_string(), TokenKind::Await),
        ("defer".to_string(), TokenKind::Defer),
        ("unsafe".to_string(), TokenKind::Unsafe),
        ("comptime".to_string(), TokenKind::Comptime),
        ("true".to_string(), TokenKind::True),
        ("false".to_string(), TokenKind::False),
        ("Some".to_string(), TokenKind::Some),
        ("None".to_string(), TokenKind::None),
        ("Ok".to_string(), TokenKind::Ok),
        ("Err".to_string(), TokenKind::Err),
        ("unreachable".to_string(), TokenKind::Unreachable),
    ];

    serde_json::to_string_pretty(&catalog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_detection() {
        assert!(TokenKind::If.is_keyword());
        assert!(TokenKind::Fn.is_keyword());
        assert!(!TokenKind::Plus.is_keyword());
        assert!(!TokenKind::Ident.is_keyword());
    }

    #[test]
    fn test_literal_detection() {
        assert!(TokenKind::IntLiteral.is_literal());
        assert!(TokenKind::StringLiteral.is_literal());
        assert!(TokenKind::True.is_literal());
        assert!(!TokenKind::Plus.is_literal());
    }

    #[test]
    fn test_operator_detection() {
        assert!(TokenKind::Plus.is_operator());
        assert!(TokenKind::EqEq.is_operator());
        assert!(TokenKind::PipeGt.is_operator());
        assert!(!TokenKind::If.is_operator());
    }

    #[test]
    fn test_keyword_str() {
        assert_eq!(TokenKind::If.keyword_str(), Some("if"));
        assert_eq!(TokenKind::Fn.keyword_str(), Some("fn"));
        assert_eq!(TokenKind::Plus.keyword_str(), None);
    }

    #[test]
    fn test_token_creation() {
        let token = Token::new(
            TokenKind::Fn,
            "fn".to_string(),
            "test.ax".to_string(),
            1,
            1,
        );
        assert_eq!(token.kind, TokenKind::Fn);
        assert_eq!(token.lexeme, "fn");
        assert_eq!(token.line, 1);
        assert_eq!(token.column, 1);
    }

    #[test]
    fn test_token_catalog_export() {
        let catalog = export_token_catalog().unwrap();
        assert!(catalog.contains("\"if\""));
        assert!(catalog.contains("\"fn\""));
    }
}
