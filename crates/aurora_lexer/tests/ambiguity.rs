//! Ambiguity Tests for Aurora Lexer
//!
//! Verifies that the lexer correctly handles ambiguous cases using maximal-munch

use aurora_lexer::{Lexer, TokenKind};

#[test]
fn test_dot_vs_dotdot_vs_dotdoteq() {
    // Test: .  vs  ..  vs  ..=
    let source = ". .. ..=";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    let tok1 = lexer.next_token().unwrap();
    assert_eq!(tok1.kind, TokenKind::Dot);

    let tok2 = lexer.next_token().unwrap();
    assert_eq!(tok2.kind, TokenKind::DotDot);

    let tok3 = lexer.next_token().unwrap();
    assert_eq!(tok3.kind, TokenKind::DotDotEq);
}

#[test]
fn test_range_literal() {
    // Test: 1..2  vs  1 .. 2  vs  1..=2
    let source = "1..2 1 .. 2 1..=2";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    // 1..2
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::DotDot);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);

    // 1 .. 2
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::DotDot);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);

    // 1..=2
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::DotDotEq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);
}

#[test]
fn test_operator_maximal_munch() {
    // Test: +  vs  +=  vs  ++
    let source = "+ += ++";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::PlusEq);
    // ++ should lex as + +
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
}

#[test]
fn test_comparison_operators() {
    // Test: <  <=  <<  <<=  <|
    let source = "< <= << <<= <|";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Lt);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LtEq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LtLt);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LtLtEq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::LtPipe);
}

#[test]
fn test_arrow_operators() {
    // Test: -  ->  =>
    let source = "- -> =>";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Minus);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::RArrow);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::FatArrow);
}

#[test]
fn test_colon_vs_coloncolon() {
    // Test: :  vs  ::
    let source = ": ::";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Colon);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::ColonColon);
}

#[test]
fn test_keyword_vs_identifier() {
    // Keywords should take precedence over identifiers
    let source = "if iff self self_value";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::If);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::SelfLower);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Ident);
}

#[test]
fn test_question_operators() {
    // Test: ?  vs  ??
    let source = "? ??";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Question);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::QuestionQuestion);
}

#[test]
fn test_pipe_operators() {
    // Test: |  |>  |=  ||
    let source = "| |> |= ||";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Or);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::PipeGt);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::OrEq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::OrOr);
}

#[test]
fn test_star_operators() {
    // Test: *  **  *=
    let source = "* ** *=";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Star);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::StarStar);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::StarEq);
}

#[test]
fn test_float_vs_range() {
    // Test: 1.0  vs  1..0  (float vs range)
    let source1 = "1.0";
    let source2 = "1..0";

    let mut lexer1 = Lexer::new(source1, "test.ax".to_string()).unwrap();
    assert_eq!(lexer1.next_token().unwrap().kind, TokenKind::FloatLiteral);

    let mut lexer2 = Lexer::new(source2, "test.ax".to_string()).unwrap();
    assert_eq!(lexer2.next_token().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer2.next_token().unwrap().kind, TokenKind::DotDot);
    assert_eq!(lexer2.next_token().unwrap().kind, TokenKind::IntLiteral);
}

#[test]
fn test_and_operators() {
    // Test: &  &&  &=
    let source = "& && &=";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::And);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AndAnd);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::AndEq);
}

#[test]
fn test_equals_operators() {
    // Test: =  ==  =>
    let source = "= == =>";
    let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();

    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::EqEq);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::FatArrow);
}

/// Generate comprehensive ambiguity report
#[test]
fn test_generate_ambiguity_report() {
    // This test verifies that no ambiguities exist in the lexer
    // by testing all edge cases

    let test_cases = vec![
        (".", vec![TokenKind::Dot]),
        ("..", vec![TokenKind::DotDot]),
        ("..=", vec![TokenKind::DotDotEq]),
        ("...", vec![TokenKind::DotDotDot]),
        ("+", vec![TokenKind::Plus]),
        ("+=", vec![TokenKind::PlusEq]),
        ("-", vec![TokenKind::Minus]),
        ("->", vec![TokenKind::RArrow]),
        (":", vec![TokenKind::Colon]),
        ("::", vec![TokenKind::ColonColon]),
        ("=", vec![TokenKind::Eq]),
        ("==", vec![TokenKind::EqEq]),
        ("=>", vec![TokenKind::FatArrow]),
        ("<", vec![TokenKind::Lt]),
        ("<=", vec![TokenKind::LtEq]),
        ("<<", vec![TokenKind::LtLt]),
        ("<<=", vec![TokenKind::LtLtEq]),
        ("<|", vec![TokenKind::LtPipe]),
        (">", vec![TokenKind::Gt]),
        (">=", vec![TokenKind::GtEq]),
        (">>", vec![TokenKind::GtGt]),
        (">>=", vec![TokenKind::GtGtEq]),
        ("|", vec![TokenKind::Or]),
        ("|>", vec![TokenKind::PipeGt]),
        ("|=", vec![TokenKind::OrEq]),
        ("||", vec![TokenKind::OrOr]),
        ("&", vec![TokenKind::And]),
        ("&&", vec![TokenKind::AndAnd]),
        ("&=", vec![TokenKind::AndEq]),
        ("*", vec![TokenKind::Star]),
        ("**", vec![TokenKind::StarStar]),
        ("*=", vec![TokenKind::StarEq]),
        ("?", vec![TokenKind::Question]),
        ("??", vec![TokenKind::QuestionQuestion]),
    ];

    let mut failures = Vec::new();

    for (source, expected) in test_cases {
        let mut lexer = Lexer::new(source, "test.ax".to_string()).unwrap();
        let mut tokens = Vec::new();

        loop {
            let tok = lexer.next_token().unwrap();
            if tok.kind == TokenKind::Eof {
                break;
            }
            tokens.push(tok.kind);
        }

        if tokens != expected {
            failures.push(format!(
                "Source: {:?}, Expected: {:?}, Got: {:?}",
                source, expected, tokens
            ));
        }
    }

    if !failures.is_empty() {
        panic!("Ambiguity failures:\n{}", failures.join("\n"));
    }
}
