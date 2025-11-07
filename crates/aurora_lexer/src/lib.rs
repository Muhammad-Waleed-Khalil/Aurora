//! aurora_lexer - Aurora Lexer Agent
//!
//! This crate implements the lexical analysis phase of the Aurora compiler.
//! It uses a table-driven NFA with maximal-munch tokenization.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod tokens;
pub mod nfa;
pub mod lexer;

pub use tokens::{Token, TokenKind};
pub use nfa::{KeywordTable, MaximalMunch, LexError};
pub use lexer::Lexer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_kinds_exist() {
        // Verify all major token kinds are defined
        let _ = TokenKind::If;
        let _ = TokenKind::Fn;
        let _ = TokenKind::Plus;
        let _ = TokenKind::Ident;
    }
}
