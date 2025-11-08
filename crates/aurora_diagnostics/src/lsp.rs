//! Language Server Protocol support

use serde::{Deserialize, Serialize};

/// LSP position (line, character)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// Zero-based line number
    pub line: u32,
    /// Zero-based character offset
    pub character: u32,
}

/// LSP range
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    /// Start position
    pub start: Position,
    /// End position
    pub end: Position,
}

/// Text edit for code actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// Range to edit
    pub range: Range,
    /// New text
    pub new_text: String,
}

/// Completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Label shown in UI
    pub label: String,
    /// Completion kind
    pub kind: CompletionKind,
    /// Detail information
    pub detail: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
    /// Text to insert
    pub insert_text: Option<String>,
}

/// Completion item kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionKind {
    /// Function
    Function,
    /// Variable
    Variable,
    /// Keyword
    Keyword,
    /// Type
    Type,
    /// Module
    Module,
    /// Constant
    Constant,
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    /// Hover contents
    pub contents: String,
    /// Optional range
    pub range: Option<Range>,
}

/// Code action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeAction {
    /// Action title
    pub title: String,
    /// Edit to apply
    pub edit: Vec<TextEdit>,
    /// Is preferred action
    pub is_preferred: bool,
}

/// Symbol information for document outline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol kind
    pub kind: SymbolKind,
    /// Symbol range
    pub range: Range,
    /// Selection range
    pub selection_range: Range,
    /// Children symbols
    #[serde(default)]
    pub children: Vec<DocumentSymbol>,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    /// File
    File,
    /// Module
    Module,
    /// Function
    Function,
    /// Type
    Type,
    /// Variable
    Variable,
    /// Constant
    Constant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position {
            line: 10,
            character: 5,
        };
        assert_eq!(pos.line, 10);
        assert_eq!(pos.character, 5);
    }

    #[test]
    fn test_range() {
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 10,
            },
        };
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.character, 10);
    }

    #[test]
    fn test_text_edit() {
        let edit = TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            new_text: "fixed".to_string(),
        };
        assert_eq!(edit.new_text, "fixed");
    }

    #[test]
    fn test_completion_item() {
        let item = CompletionItem {
            label: "test_fn".to_string(),
            kind: CompletionKind::Function,
            detail: Some("fn test_fn()".to_string()),
            documentation: Some("Test function".to_string()),
            insert_text: Some("test_fn()".to_string()),
        };
        assert_eq!(item.label, "test_fn");
        assert_eq!(item.kind, CompletionKind::Function);
    }

    #[test]
    fn test_hover() {
        let hover = Hover {
            contents: "Type: i32".to_string(),
            range: None,
        };
        assert_eq!(hover.contents, "Type: i32");
    }

    #[test]
    fn test_code_action() {
        let action = CodeAction {
            title: "Fix typo".to_string(),
            edit: vec![],
            is_preferred: true,
        };
        assert_eq!(action.title, "Fix typo");
        assert!(action.is_preferred);
    }

    #[test]
    fn test_document_symbol() {
        let symbol = DocumentSymbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 10,
                    character: 0,
                },
            },
            selection_range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 4,
                },
            },
            children: vec![],
        };
        assert_eq!(symbol.name, "main");
        assert_eq!(symbol.kind, SymbolKind::Function);
    }
}
