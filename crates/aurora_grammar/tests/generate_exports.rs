//! Generate JSON exports for grammar documentation

use aurora_grammar::{AuroraGrammar, ConflictAnalyzer, PrecedenceTable};
use std::fs;

#[test]
#[ignore] // Run explicitly with: cargo test --package aurora_grammar generate_exports -- --ignored
fn generate_precedence_json() {
    let table = PrecedenceTable::new();
    let json = table.to_json().unwrap();
    fs::write("crates/aurora_grammar/precedence.json", json).unwrap();
    println!("Generated precedence.json");
}

#[test]
#[ignore]
fn generate_conflict_report() {
    let grammar = AuroraGrammar::new();
    let analyzer = ConflictAnalyzer::new(grammar);
    let report = analyzer.analyze();

    let json = report.to_json().unwrap();
    fs::write("crates/aurora_grammar/conflict_report.json", json).unwrap();

    let report_text = report.to_report();
    fs::write("crates/aurora_grammar/conflict_report.txt", report_text).unwrap();

    println!("Generated conflict_report.json and conflict_report.txt");
}

#[test]
#[ignore]
fn generate_grammar_bnf() {
    let grammar = AuroraGrammar::new();
    let bnf = grammar.to_bnf();
    fs::write("docs/grammar.bnf", bnf).unwrap();
    println!("Generated grammar.bnf");
}

#[test]
#[ignore]
fn generate_grammar_json() {
    let grammar = AuroraGrammar::new();
    let json = grammar.to_json().unwrap();
    fs::write("crates/aurora_grammar/grammar.json", json).unwrap();
    println!("Generated grammar.json");
}
