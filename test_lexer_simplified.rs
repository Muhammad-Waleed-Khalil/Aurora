// Test script to verify simplified syntax keywords are recognized

use aurora_lexer::{Lexer, TokenKind};

fn main() {
    let source = r#"
fun greet(name)
    print "Hello", name

fun main()
    x = 10
    y = 5
    if x > y and y > 0
        print "x is bigger"
    elif x == y or no
        print "Equal or false"
    else
        print "y is bigger"

    active = yes
    ret 0
"#;

    let mut lexer = Lexer::new(source, "test".to_string()).expect("Failed to create lexer");
    let mut found_keywords = Vec::new();

    loop {
        let token = lexer.next_token().expect("Failed to get token");
        match token.kind {
            TokenKind::Fun => found_keywords.push("fun"),
            TokenKind::Print => found_keywords.push("print"),
            TokenKind::Elif => found_keywords.push("elif"),
            TokenKind::AndKeyword => found_keywords.push("and"),
            TokenKind::OrKeyword => found_keywords.push("or"),
            TokenKind::NotKeyword => found_keywords.push("not"),
            TokenKind::Yes => found_keywords.push("yes"),
            TokenKind::No => found_keywords.push("no"),
            TokenKind::Ret => found_keywords.push("ret"),
            TokenKind::Var => found_keywords.push("var"),
            TokenKind::Eof => break,
            _ => {}
        }
    }

    println!("✓ Found simplified syntax keywords: {:?}", found_keywords);

    // Verify all keywords were recognized
    let expected = vec!["fun", "print", "fun", "and", "elif", "or", "no", "yes", "ret"];
    let mut all_found = true;

    for exp in &expected {
        if !found_keywords.contains(exp) {
            println!("✗ Missing keyword: {}", exp);
            all_found = false;
        }
    }

    if all_found {
        println!("✓ All simplified syntax keywords recognized correctly!");
    }
}
