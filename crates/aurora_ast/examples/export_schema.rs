//! Export AST schema to JSON
//!
//! This example generates a machine-readable JSON schema describing
//! all AST node types, their fields, and relationships.

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
struct SchemaExport {
    version: String,
    description: String,
    node_types: Vec<NodeType>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeType {
    name: String,
    description: String,
    variants: Vec<NodeVariant>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NodeVariant {
    name: String,
    description: String,
    fields: Vec<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Field {
    name: String,
    #[serde(rename = "type")]
    type_name: String,
    optional: bool,
}

fn main() {
    let schema = SchemaExport {
        version: "1.0.0".to_string(),
        description: "Aurora AST Schema - Frozen for MVP".to_string(),
        node_types: vec![
            NodeType {
                name: "Expr".to_string(),
                description: "Expression nodes".to_string(),
                variants: vec![
                    NodeVariant {
                        name: "Literal".to_string(),
                        description: "Literal value (int, float, string, char, bool)".to_string(),
                        fields: vec![Field {
                            name: "value".to_string(),
                            type_name: "Literal".to_string(),
                            optional: false,
                        }],
                    },
                    NodeVariant {
                        name: "Ident".to_string(),
                        description: "Identifier reference".to_string(),
                        fields: vec![Field {
                            name: "name".to_string(),
                            type_name: "String".to_string(),
                            optional: false,
                        }],
                    },
                    NodeVariant {
                        name: "Binary".to_string(),
                        description: "Binary operation (e.g., a + b)".to_string(),
                        fields: vec![
                            Field {
                                name: "op".to_string(),
                                type_name: "BinaryOp".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "left".to_string(),
                                type_name: "ExprId".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "right".to_string(),
                                type_name: "ExprId".to_string(),
                                optional: false,
                            },
                        ],
                    },
                    // Additional variants would be listed here
                ],
            },
            NodeType {
                name: "Stmt".to_string(),
                description: "Statement nodes".to_string(),
                variants: vec![
                    NodeVariant {
                        name: "Let".to_string(),
                        description: "Let binding".to_string(),
                        fields: vec![
                            Field {
                                name: "pattern".to_string(),
                                type_name: "PatternId".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "ty".to_string(),
                                type_name: "TypeId".to_string(),
                                optional: true,
                            },
                            Field {
                                name: "init".to_string(),
                                type_name: "ExprId".to_string(),
                                optional: true,
                            },
                            Field {
                                name: "mutable".to_string(),
                                type_name: "bool".to_string(),
                                optional: false,
                            },
                        ],
                    },
                ],
            },
            NodeType {
                name: "Type".to_string(),
                description: "Type nodes".to_string(),
                variants: vec![
                    NodeVariant {
                        name: "Int".to_string(),
                        description: "Signed integer type".to_string(),
                        fields: vec![Field {
                            name: "size".to_string(),
                            type_name: "IntType".to_string(),
                            optional: false,
                        }],
                    },
                    NodeVariant {
                        name: "Bool".to_string(),
                        description: "Boolean type".to_string(),
                        fields: vec![],
                    },
                ],
            },
            NodeType {
                name: "Pattern".to_string(),
                description: "Pattern nodes for destructuring".to_string(),
                variants: vec![
                    NodeVariant {
                        name: "Wildcard".to_string(),
                        description: "Wildcard pattern (_)".to_string(),
                        fields: vec![],
                    },
                    NodeVariant {
                        name: "Ident".to_string(),
                        description: "Identifier pattern".to_string(),
                        fields: vec![
                            Field {
                                name: "name".to_string(),
                                type_name: "String".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "is_mut".to_string(),
                                type_name: "bool".to_string(),
                                optional: false,
                            },
                        ],
                    },
                ],
            },
            NodeType {
                name: "Item".to_string(),
                description: "Top-level declaration nodes".to_string(),
                variants: vec![
                    NodeVariant {
                        name: "Function".to_string(),
                        description: "Function declaration".to_string(),
                        fields: vec![
                            Field {
                                name: "name".to_string(),
                                type_name: "String".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "is_pub".to_string(),
                                type_name: "bool".to_string(),
                                optional: false,
                            },
                            Field {
                                name: "is_async".to_string(),
                                type_name: "bool".to_string(),
                                optional: false,
                            },
                        ],
                    },
                ],
            },
        ],
    };

    let json = serde_json::to_string_pretty(&schema).unwrap();
    fs::write("schema.json", &json).unwrap();

    println!("✓ Generated schema.json ({} bytes)", json.len());
    println!("\nAST Schema:");
    println!("  Version: {}", schema.version);
    println!("  Node types: {}", schema.node_types.len());
    println!("\n✅ AST schema frozen for MVP v1.0.0");
}
