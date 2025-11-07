//! Aurora Grammar - Context-Free Grammar (CFG) Rules
//!
//! Defines the complete BNF notation for Aurora's syntax.
//! The grammar is designed to be LL(1) for declarations and uses
//! Pratt parsing for expressions.

use serde::{Deserialize, Serialize};

/// Grammar rule representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrammarRule {
    /// Non-terminal name
    pub name: String,
    /// Production rules
    pub productions: Vec<Production>,
    /// Documentation
    pub doc: Option<String>,
}

/// A single production alternative
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Production {
    /// Symbols in this production
    pub symbols: Vec<Symbol>,
    /// Optional description
    pub description: Option<String>,
}

/// Grammar symbol (terminal or non-terminal)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Symbol {
    /// Terminal symbol (token)
    Terminal { value: String },
    /// Non-terminal symbol (grammar rule)
    NonTerminal { name: String },
    /// Optional symbol (?)
    Optional { symbol: Box<Symbol> },
    /// Zero or more repetitions (*)
    ZeroOrMore { symbol: Box<Symbol> },
    /// One or more repetitions (+)
    OneOrMore { symbol: Box<Symbol> },
    /// Group of symbols
    Group { symbols: Vec<Symbol> },
}

/// Aurora's complete grammar
pub struct AuroraGrammar {
    /// Grammar rules
    rules: Vec<GrammarRule>,
}

impl AuroraGrammar {
    /// Create the complete Aurora grammar
    pub fn new() -> Self {
        let mut grammar = Self { rules: Vec::new() };

        // Top-level structure
        grammar.add_program_rules();

        // Declarations
        grammar.add_declaration_rules();

        // Statements
        grammar.add_statement_rules();

        // Types
        grammar.add_type_rules();

        // Patterns
        grammar.add_pattern_rules();

        // Expressions (simplified - actual parsing uses Pratt)
        grammar.add_expression_rules();

        grammar
    }

    fn add_program_rules(&mut self) {
        use Symbol::*;

        self.add_rule(GrammarRule {
            name: "Program".to_string(),
            productions: vec![Production {
                symbols: vec![ZeroOrMore {
                    symbol: Box::new(NonTerminal {
                        name: "Item".to_string(),
                    }),
                }],
                description: Some("Zero or more top-level items".to_string()),
            }],
            doc: Some("Top-level program structure".to_string()),
        });

        self.add_rule(GrammarRule {
            name: "Item".to_string(),
            productions: vec![
                Production {
                    symbols: vec![NonTerminal {
                        name: "FunctionDecl".to_string(),
                    }],
                    description: Some("Function declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "TypeDecl".to_string(),
                    }],
                    description: Some("Type declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "TraitDecl".to_string(),
                    }],
                    description: Some("Trait declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "ImplDecl".to_string(),
                    }],
                    description: Some("Implementation declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "ConstDecl".to_string(),
                    }],
                    description: Some("Constant declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "ModDecl".to_string(),
                    }],
                    description: Some("Module declaration".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "UseDecl".to_string(),
                    }],
                    description: Some("Use/import declaration".to_string()),
                },
            ],
            doc: Some("Top-level item (declaration)".to_string()),
        });
    }

    fn add_declaration_rules(&mut self) {
        use Symbol::*;

        // Function declaration
        self.add_rule(GrammarRule {
            name: "FunctionDecl".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "pub".to_string(),
                        }),
                    },
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "async".to_string(),
                        }),
                    },
                    Terminal {
                        value: "fn".to_string(),
                    },
                    Terminal {
                        value: "IDENT".to_string(),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "GenericParams".to_string(),
                        }),
                    },
                    Terminal {
                        value: "(".to_string(),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "ParamList".to_string(),
                        }),
                    },
                    Terminal {
                        value: ")".to_string(),
                    },
                    Optional {
                        symbol: Box::new(Group {
                            symbols: vec![
                                Terminal {
                                    value: "->".to_string(),
                                },
                                NonTerminal {
                                    name: "Type".to_string(),
                                },
                            ],
                        }),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "WhereClause".to_string(),
                        }),
                    },
                    NonTerminal {
                        name: "Block".to_string(),
                    },
                ],
                description: Some("Function declaration".to_string()),
            }],
            doc: Some("fn foo<T>(x: T) -> T where T: Clone { ... }".to_string()),
        });

        // Type declaration
        self.add_rule(GrammarRule {
            name: "TypeDecl".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "pub".to_string(),
                        }),
                    },
                    Terminal {
                        value: "type".to_string(),
                    },
                    Terminal {
                        value: "IDENT".to_string(),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "GenericParams".to_string(),
                        }),
                    },
                    Terminal {
                        value: "=".to_string(),
                    },
                    NonTerminal {
                        name: "Type".to_string(),
                    },
                    Terminal {
                        value: ";".to_string(),
                    },
                ],
                description: Some("Type alias".to_string()),
            }],
            doc: Some("type MyInt = i64;".to_string()),
        });

        // Trait declaration
        self.add_rule(GrammarRule {
            name: "TraitDecl".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "pub".to_string(),
                        }),
                    },
                    Terminal {
                        value: "trait".to_string(),
                    },
                    Terminal {
                        value: "IDENT".to_string(),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "GenericParams".to_string(),
                        }),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "WhereClause".to_string(),
                        }),
                    },
                    Terminal {
                        value: "{".to_string(),
                    },
                    ZeroOrMore {
                        symbol: Box::new(NonTerminal {
                            name: "TraitItem".to_string(),
                        }),
                    },
                    Terminal {
                        value: "}".to_string(),
                    },
                ],
                description: Some("Trait definition".to_string()),
            }],
            doc: Some("trait Clone { fn clone(&self) -> Self; }".to_string()),
        });

        // Use declaration
        self.add_rule(GrammarRule {
            name: "UseDecl".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "pub".to_string(),
                        }),
                    },
                    Terminal {
                        value: "use".to_string(),
                    },
                    NonTerminal {
                        name: "Path".to_string(),
                    },
                    Terminal {
                        value: ";".to_string(),
                    },
                ],
                description: Some("Use/import statement".to_string()),
            }],
            doc: Some("use std::collections::HashMap;".to_string()),
        });
    }

    fn add_statement_rules(&mut self) {
        use Symbol::*;

        self.add_rule(GrammarRule {
            name: "Statement".to_string(),
            productions: vec![
                Production {
                    symbols: vec![NonTerminal {
                        name: "LetStmt".to_string(),
                    }],
                    description: Some("Let binding".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "ExprStmt".to_string(),
                    }],
                    description: Some("Expression statement".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "Item".to_string(),
                    }],
                    description: Some("Item in statement position".to_string()),
                },
            ],
            doc: Some("Statement".to_string()),
        });

        self.add_rule(GrammarRule {
            name: "LetStmt".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Terminal {
                        value: "let".to_string(),
                    },
                    Optional {
                        symbol: Box::new(Terminal {
                            value: "mut".to_string(),
                        }),
                    },
                    NonTerminal {
                        name: "Pattern".to_string(),
                    },
                    Optional {
                        symbol: Box::new(Group {
                            symbols: vec![
                                Terminal {
                                    value: ":".to_string(),
                                },
                                NonTerminal {
                                    name: "Type".to_string(),
                                },
                            ],
                        }),
                    },
                    Optional {
                        symbol: Box::new(Group {
                            symbols: vec![
                                Terminal {
                                    value: "=".to_string(),
                                },
                                NonTerminal {
                                    name: "Expr".to_string(),
                                },
                            ],
                        }),
                    },
                    Terminal {
                        value: ";".to_string(),
                    },
                ],
                description: Some("Let binding".to_string()),
            }],
            doc: Some("let mut x: i64 = 42;".to_string()),
        });

        self.add_rule(GrammarRule {
            name: "Block".to_string(),
            productions: vec![Production {
                symbols: vec![
                    Terminal {
                        value: "{".to_string(),
                    },
                    ZeroOrMore {
                        symbol: Box::new(NonTerminal {
                            name: "Statement".to_string(),
                        }),
                    },
                    Optional {
                        symbol: Box::new(NonTerminal {
                            name: "Expr".to_string(),
                        }),
                    },
                    Terminal {
                        value: "}".to_string(),
                    },
                ],
                description: Some("Block expression".to_string()),
            }],
            doc: Some("{ stmt1; stmt2; expr }".to_string()),
        });
    }

    fn add_type_rules(&mut self) {
        use Symbol::*;

        self.add_rule(GrammarRule {
            name: "Type".to_string(),
            productions: vec![
                Production {
                    symbols: vec![NonTerminal {
                        name: "PrimitiveType".to_string(),
                    }],
                    description: Some("Primitive type".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "PathType".to_string(),
                    }],
                    description: Some("Named type".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "TupleType".to_string(),
                    }],
                    description: Some("Tuple type".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "ArrayType".to_string(),
                    }],
                    description: Some("Array type".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "FunctionType".to_string(),
                    }],
                    description: Some("Function type".to_string()),
                },
            ],
            doc: Some("Type expressions".to_string()),
        });

        self.add_rule(GrammarRule {
            name: "PrimitiveType".to_string(),
            productions: vec![
                Production {
                    symbols: vec![Terminal {
                        value: "i8|i16|i32|i64|u8|u16|u32|u64|f32|f64|bool|char|str".to_string(),
                    }],
                    description: Some("Primitive type keywords".to_string()),
                },
            ],
            doc: Some("Primitive types".to_string()),
        });
    }

    fn add_pattern_rules(&mut self) {
        use Symbol::*;

        self.add_rule(GrammarRule {
            name: "Pattern".to_string(),
            productions: vec![
                Production {
                    symbols: vec![Terminal {
                        value: "IDENT".to_string(),
                    }],
                    description: Some("Identifier pattern".to_string()),
                },
                Production {
                    symbols: vec![Terminal {
                        value: "_".to_string(),
                    }],
                    description: Some("Wildcard pattern".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "TuplePattern".to_string(),
                    }],
                    description: Some("Tuple pattern".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "StructPattern".to_string(),
                    }],
                    description: Some("Struct pattern".to_string()),
                },
            ],
            doc: Some("Pattern matching".to_string()),
        });
    }

    fn add_expression_rules(&mut self) {
        use Symbol::*;

        // Note: Expression parsing uses Pratt parser, so this is simplified
        self.add_rule(GrammarRule {
            name: "Expr".to_string(),
            productions: vec![Production {
                symbols: vec![NonTerminal {
                    name: "PrattExpr".to_string(),
                }],
                description: Some("Expression parsed by Pratt parser".to_string()),
            }],
            doc: Some("Expressions use Pratt parsing with precedence table".to_string()),
        });

        self.add_rule(GrammarRule {
            name: "PrimaryExpr".to_string(),
            productions: vec![
                Production {
                    symbols: vec![Terminal {
                        value: "LITERAL".to_string(),
                    }],
                    description: Some("Literal value".to_string()),
                },
                Production {
                    symbols: vec![Terminal {
                        value: "IDENT".to_string(),
                    }],
                    description: Some("Identifier".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "Block".to_string(),
                    }],
                    description: Some("Block expression".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "IfExpr".to_string(),
                    }],
                    description: Some("If expression".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "MatchExpr".to_string(),
                    }],
                    description: Some("Match expression".to_string()),
                },
                Production {
                    symbols: vec![NonTerminal {
                        name: "LoopExpr".to_string(),
                    }],
                    description: Some("Loop expression".to_string()),
                },
            ],
            doc: Some("Primary expressions".to_string()),
        });
    }

    fn add_rule(&mut self, rule: GrammarRule) {
        self.rules.push(rule);
    }

    /// Get all grammar rules
    pub fn rules(&self) -> &[GrammarRule] {
        &self.rules
    }

    /// Get a specific rule by name
    pub fn get_rule(&self, name: &str) -> Option<&GrammarRule> {
        self.rules.iter().find(|r| r.name == name)
    }

    /// Export grammar as JSON
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.rules)
    }

    /// Export grammar as BNF notation
    pub fn to_bnf(&self) -> String {
        let mut bnf = String::new();

        for rule in &self.rules {
            if let Some(doc) = &rule.doc {
                bnf.push_str(&format!("// {}\n", doc));
            }

            bnf.push_str(&format!("{} ::=\n", rule.name));

            for (i, prod) in rule.productions.iter().enumerate() {
                if i > 0 {
                    bnf.push_str("    | ");
                } else {
                    bnf.push_str("      ");
                }

                for symbol in &prod.symbols {
                    bnf.push_str(&self.symbol_to_bnf(symbol));
                    bnf.push(' ');
                }

                if let Some(desc) = &prod.description {
                    bnf.push_str(&format!("  // {}", desc));
                }

                bnf.push('\n');
            }

            bnf.push('\n');
        }

        bnf
    }

    fn symbol_to_bnf(&self, symbol: &Symbol) -> String {
        match symbol {
            Symbol::Terminal { value } => format!("'{}'", value),
            Symbol::NonTerminal { name } => name.clone(),
            Symbol::Optional { symbol } => format!("[ {} ]", self.symbol_to_bnf(symbol)),
            Symbol::ZeroOrMore { symbol } => format!("{{ {} }}", self.symbol_to_bnf(symbol)),
            Symbol::OneOrMore { symbol } => format!("{{ {} }}+", self.symbol_to_bnf(symbol)),
            Symbol::Group { symbols } => {
                let inner: Vec<_> = symbols.iter().map(|s| self.symbol_to_bnf(s)).collect();
                format!("( {} )", inner.join(" "))
            }
        }
    }
}

impl Default for AuroraGrammar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grammar_creation() {
        let grammar = AuroraGrammar::new();
        assert!(!grammar.rules().is_empty());
    }

    #[test]
    fn test_get_rule() {
        let grammar = AuroraGrammar::new();
        let program_rule = grammar.get_rule("Program");
        assert!(program_rule.is_some());
        assert_eq!(program_rule.unwrap().name, "Program");
    }

    #[test]
    fn test_bnf_export() {
        let grammar = AuroraGrammar::new();
        let bnf = grammar.to_bnf();

        assert!(bnf.contains("Program ::="));
        assert!(bnf.contains("FunctionDecl ::="));
        assert!(bnf.contains("'fn'"));
    }

    #[test]
    fn test_json_export() {
        let grammar = AuroraGrammar::new();
        let json = grammar.to_json().unwrap();

        assert!(json.contains("\"name\""));
        assert!(json.contains("\"productions\""));
    }

    #[test]
    fn test_all_major_constructs_defined() {
        let grammar = AuroraGrammar::new();

        let required_rules = vec![
            "Program",
            "Item",
            "FunctionDecl",
            "TypeDecl",
            "TraitDecl",
            "UseDecl",
            "Statement",
            "LetStmt",
            "Block",
            "Type",
            "Pattern",
            "Expr",
        ];

        for rule_name in required_rules {
            assert!(
                grammar.get_rule(rule_name).is_some(),
                "Missing rule: {}",
                rule_name
            );
        }
    }
}
