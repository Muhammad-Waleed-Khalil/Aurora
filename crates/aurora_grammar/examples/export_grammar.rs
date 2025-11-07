//! Export grammar artifacts to JSON and text formats

use aurora_grammar::{AuroraGrammar, ConflictAnalyzer, PrecedenceTable};

fn main() {
    println!("Generating Aurora Grammar Exports...\n");

    // 1. Precedence table
    let table = PrecedenceTable::new();
    let json = table.to_json().unwrap();
    std::fs::write("crates/aurora_grammar/precedence.json", &json).unwrap();
    println!("✓ Generated precedence.json ({} bytes)", json.len());

    // 2. Grammar
    let grammar = AuroraGrammar::new();

    // 3. Conflict report
    let analyzer = ConflictAnalyzer::new(AuroraGrammar::new());
    let report = analyzer.analyze();

    let report_json = report.to_json().unwrap();
    std::fs::write("crates/aurora_grammar/conflict_report.json", &report_json).unwrap();
    println!("✓ Generated conflict_report.json ({} bytes)", report_json.len());

    let report_text = report.to_report();
    std::fs::write("crates/aurora_grammar/conflict_report.txt", &report_text).unwrap();
    println!("✓ Generated conflict_report.txt ({} bytes)", report_text.len());

    // 4. Grammar BNF
    let bnf = grammar.to_bnf();
    std::fs::write("docs/grammar.bnf", &bnf).unwrap();
    println!("✓ Generated grammar.bnf ({} bytes)", bnf.len());

    // 5. Grammar JSON
    let grammar_json = grammar.to_json().unwrap();
    std::fs::write("crates/aurora_grammar/grammar.json", &grammar_json).unwrap();
    println!("✓ Generated grammar.json ({} bytes)", grammar_json.len());

    println!("\n✅ All exports generated successfully!");
}
