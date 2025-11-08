//! Documentation Generator for Aurora
//!
//! Generates API documentation, language reference, and guides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Documentation errors
#[derive(Debug, Error)]
pub enum DocError {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Format error
    #[error("Format error: {0}")]
    FormatError(#[from] std::fmt::Error),

    /// Invalid documentation
    #[error("Invalid documentation: {0}")]
    InvalidDoc(String),
}

/// Result type
pub type Result<T> = std::result::Result<T, DocError>;

/// Documentation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocItem {
    /// Item name
    pub name: String,
    /// Item kind (function, type, module, etc.)
    pub kind: DocKind,
    /// Description
    pub description: String,
    /// Code examples
    pub examples: Vec<String>,
    /// See also links
    pub see_also: Vec<String>,
}

/// Documentation kind
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DocKind {
    /// Function
    Function,
    /// Type
    Type,
    /// Module
    Module,
    /// Constant
    Constant,
    /// Trait
    Trait,
}

impl DocItem {
    /// Create a new doc item
    pub fn new(name: impl Into<String>, kind: DocKind, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind,
            description: description.into(),
            examples: Vec::new(),
            see_also: Vec::new(),
        }
    }

    /// Add an example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Add a see-also link
    pub fn with_see_also(mut self, link: impl Into<String>) -> Self {
        self.see_also.push(link.into());
        self
    }

    /// Generate markdown
    pub fn to_markdown(&self) -> Result<String> {
        let mut output = String::new();

        // Title
        writeln!(output, "# {}", self.name)?;
        writeln!(output)?;

        // Kind badge
        let kind_str = match self.kind {
            DocKind::Function => "Function",
            DocKind::Type => "Type",
            DocKind::Module => "Module",
            DocKind::Constant => "Constant",
            DocKind::Trait => "Trait",
        };
        writeln!(output, "*{}*", kind_str)?;
        writeln!(output)?;

        // Description
        writeln!(output, "{}", self.description)?;
        writeln!(output)?;

        // Examples
        if !self.examples.is_empty() {
            writeln!(output, "## Examples")?;
            writeln!(output)?;
            for example in &self.examples {
                writeln!(output, "```aurora")?;
                writeln!(output, "{}", example)?;
                writeln!(output, "```")?;
                writeln!(output)?;
            }
        }

        // See also
        if !self.see_also.is_empty() {
            writeln!(output, "## See Also")?;
            writeln!(output)?;
            for link in &self.see_also {
                writeln!(output, "- {}", link)?;
            }
        }

        Ok(output)
    }
}

/// Documentation generator
pub struct DocGenerator {
    /// Documentation items
    items: HashMap<String, DocItem>,
    /// Output directory
    output_dir: PathBuf,
}

impl DocGenerator {
    /// Create a new doc generator
    pub fn new(output_dir: impl AsRef<Path>) -> Self {
        Self {
            items: HashMap::new(),
            output_dir: output_dir.as_ref().to_path_buf(),
        }
    }

    /// Add a documentation item
    pub fn add_item(&mut self, item: DocItem) {
        self.items.insert(item.name.clone(), item);
    }

    /// Generate all documentation
    pub fn generate(&self) -> Result<()> {
        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Generate index
        self.generate_index()?;

        // Generate individual pages
        for item in self.items.values() {
            self.generate_item_page(item)?;
        }

        Ok(())
    }

    /// Generate index page
    fn generate_index(&self) -> Result<()> {
        let mut output = String::new();

        writeln!(output, "# Aurora Documentation")?;
        writeln!(output)?;
        writeln!(output, "Welcome to the Aurora programming language documentation.")?;
        writeln!(output)?;

        // Group by kind
        let mut by_kind: HashMap<DocKind, Vec<&DocItem>> = HashMap::new();
        for item in self.items.values() {
            by_kind.entry(item.kind.clone()).or_default().push(item);
        }

        // Functions
        if let Some(functions) = by_kind.get(&DocKind::Function) {
            writeln!(output, "## Functions")?;
            writeln!(output)?;
            for item in functions {
                writeln!(output, "- [{}]({}.md)", item.name, item.name)?;
            }
            writeln!(output)?;
        }

        // Types
        if let Some(types) = by_kind.get(&DocKind::Type) {
            writeln!(output, "## Types")?;
            writeln!(output)?;
            for item in types {
                writeln!(output, "- [{}]({}.md)", item.name, item.name)?;
            }
            writeln!(output)?;
        }

        // Modules
        if let Some(modules) = by_kind.get(&DocKind::Module) {
            writeln!(output, "## Modules")?;
            writeln!(output)?;
            for item in modules {
                writeln!(output, "- [{}]({}.md)", item.name, item.name)?;
            }
            writeln!(output)?;
        }

        // Write index
        let index_path = self.output_dir.join("index.md");
        fs::write(index_path, output)?;

        Ok(())
    }

    /// Generate individual item page
    fn generate_item_page(&self, item: &DocItem) -> Result<()> {
        let content = item.to_markdown()?;
        let path = self.output_dir.join(format!("{}.md", item.name));
        fs::write(path, content)?;
        Ok(())
    }

    /// Get number of documented items
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_item_creation() {
        let item = DocItem::new("test_func", DocKind::Function, "A test function");
        assert_eq!(item.name, "test_func");
        assert_eq!(item.kind, DocKind::Function);
        assert_eq!(item.description, "A test function");
    }

    #[test]
    fn test_with_example() {
        let item = DocItem::new("func", DocKind::Function, "desc")
            .with_example("fn test() {}");

        assert_eq!(item.examples.len(), 1);
        assert_eq!(item.examples[0], "fn test() {}");
    }

    #[test]
    fn test_with_see_also() {
        let item = DocItem::new("func", DocKind::Function, "desc")
            .with_see_also("related_func");

        assert_eq!(item.see_also.len(), 1);
        assert_eq!(item.see_also[0], "related_func");
    }

    #[test]
    fn test_to_markdown() {
        let item = DocItem::new("add", DocKind::Function, "Adds two numbers")
            .with_example("add(1, 2) // Returns 3");

        let md = item.to_markdown().unwrap();
        assert!(md.contains("# add"));
        assert!(md.contains("Function"));
        assert!(md.contains("Adds two numbers"));
        assert!(md.contains("```aurora"));
    }

    #[test]
    fn test_doc_generator() {
        let temp_dir = std::env::temp_dir().join("aurora_docs_test");
        let mut gen = DocGenerator::new(&temp_dir);

        let item = DocItem::new("test", DocKind::Function, "Test function");
        gen.add_item(item);

        assert_eq!(gen.count(), 1);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_doc_kinds() {
        assert_eq!(DocKind::Function, DocKind::Function);
        assert_ne!(DocKind::Function, DocKind::Type);
    }
}
