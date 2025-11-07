//! Module graph for Aurora
//!
//! This module implements the module dependency graph, which tracks module
//! declarations, use statements, and detects cyclic dependencies.

use aurora_ast::decl::{ItemId, ModuleDecl, UseDecl, UseTree};
use aurora_ast::expr::Path;
use aurora_ast::Span;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Unique identifier for a module
pub type ModuleId = u32;

/// A module in the module graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    /// Unique identifier
    pub id: ModuleId,
    /// Module name
    pub name: String,
    /// Parent module (None for root/crate module)
    pub parent: Option<ModuleId>,
    /// Child modules
    pub children: Vec<ModuleId>,
    /// Items declared in this module
    pub items: Vec<ItemId>,
    /// Dependencies (modules this module imports from)
    pub dependencies: Vec<ModuleDependency>,
    /// Whether this is a public module
    pub is_pub: bool,
    /// Source span
    pub span: Span,
}

/// A dependency from one module to another
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDependency {
    /// The module being depended upon
    pub target: ModulePath,
    /// Whether this is a re-export (pub use)
    pub is_pub: bool,
    /// What is being imported
    pub kind: DependencyKind,
    /// Source span
    pub span: Span,
}

/// Module path (e.g., std::io::Read)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulePath {
    /// Path segments
    pub segments: Vec<String>,
}

impl ModulePath {
    /// Create a new module path
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create a module path from an AST Path
    pub fn from_ast_path(path: &Path) -> Self {
        Self {
            segments: path.segments.clone(),
        }
    }

    /// Get the last segment (the actual imported item)
    pub fn last(&self) -> Option<&str> {
        self.segments.last().map(|s| s.as_str())
    }

    /// Get all but the last segment (the module path)
    pub fn parent_path(&self) -> ModulePath {
        let mut segments = self.segments.clone();
        segments.pop();
        ModulePath { segments }
    }

    /// Check if this path is absolute (starts with crate, super, or self)
    pub fn is_absolute(&self) -> bool {
        self.segments.first().map_or(false, |s| {
            matches!(s.as_str(), "crate" | "super" | "self")
        })
    }
}

/// Kind of dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyKind {
    /// Specific item import (use std::io::Read)
    Item(String),
    /// Glob import (use std::io::*)
    Glob,
    /// Module declaration (mod foo)
    Module,
}

/// Module graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleGraph {
    /// All modules indexed by ID
    modules: HashMap<ModuleId, Module>,
    /// Module lookup by path
    path_to_module: HashMap<ModulePath, ModuleId>,
    /// The root module (crate root)
    root_module: ModuleId,
    /// Next available module ID
    next_id: ModuleId,
}

impl ModuleGraph {
    /// Create a new module graph with a root module
    pub fn new(crate_name: String) -> Self {
        let mut graph = Self {
            modules: HashMap::new(),
            path_to_module: HashMap::new(),
            root_module: 0,
            next_id: 0,
        };

        // Create root module
        let root = Module {
            id: 0,
            name: crate_name.clone(),
            parent: None,
            children: Vec::new(),
            items: Vec::new(),
            dependencies: Vec::new(),
            is_pub: true,
            span: Span::dummy(),
        };

        graph.modules.insert(0, root);
        graph.path_to_module.insert(
            ModulePath::new(vec![crate_name]),
            0,
        );
        graph.next_id = 1;

        graph
    }

    /// Add a module declaration to the graph
    pub fn add_module(
        &mut self,
        decl: &ModuleDecl,
        parent_id: ModuleId,
    ) -> Result<ModuleId, ModuleError> {
        // Check if module already exists in this parent
        if let Some(parent) = self.modules.get(&parent_id) {
            for &child_id in &parent.children {
                if let Some(child) = self.modules.get(&child_id) {
                    if child.name == decl.name {
                        return Err(ModuleError::DuplicateModule {
                            name: decl.name.clone(),
                            first_span: child.span,
                            second_span: decl.span,
                        });
                    }
                }
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        let module = Module {
            id,
            name: decl.name.clone(),
            parent: Some(parent_id),
            children: Vec::new(),
            items: decl.items.clone().unwrap_or_default(),
            dependencies: Vec::new(),
            is_pub: decl.is_pub,
            span: decl.span,
        };

        // Build path for this module
        let path = self.build_module_path(parent_id, &decl.name);

        self.modules.insert(id, module);
        self.path_to_module.insert(path, id);

        // Add to parent's children
        if let Some(parent) = self.modules.get_mut(&parent_id) {
            parent.add_child(id);
        }

        Ok(id)
    }

    /// Add a use declaration to the graph
    pub fn add_use(
        &mut self,
        decl: &UseDecl,
        module_id: ModuleId,
    ) -> Result<(), ModuleError> {
        let dependencies = self.extract_dependencies_from_use(&decl.tree, decl.is_pub, decl.span)?;

        if let Some(module) = self.modules.get_mut(&module_id) {
            module.dependencies.extend(dependencies);
        }

        Ok(())
    }

    /// Extract dependencies from a use tree
    fn extract_dependencies_from_use(
        &self,
        tree: &UseTree,
        is_pub: bool,
        span: Span,
    ) -> Result<Vec<ModuleDependency>, ModuleError> {
        let mut deps = Vec::new();

        match tree {
            UseTree::Path { path, alias: _ } => {
                let module_path = ModulePath::from_ast_path(path);
                let kind = if let Some(item) = module_path.last() {
                    DependencyKind::Item(item.to_string())
                } else {
                    DependencyKind::Module
                };

                deps.push(ModuleDependency {
                    target: module_path,
                    is_pub,
                    kind,
                    span,
                });
            }
            UseTree::Glob { path } => {
                let module_path = ModulePath::from_ast_path(path);
                deps.push(ModuleDependency {
                    target: module_path,
                    is_pub,
                    kind: DependencyKind::Glob,
                    span,
                });
            }
            UseTree::Nested { path, trees } => {
                for nested_tree in trees {
                    let mut nested_deps =
                        self.extract_dependencies_from_use(nested_tree, is_pub, span)?;
                    // Prepend the parent path to each nested dependency
                    for dep in &mut nested_deps {
                        let prefix = ModulePath::from_ast_path(path);
                        let mut segments = prefix.segments.clone();
                        segments.extend(dep.target.segments.clone());
                        dep.target = ModulePath::new(segments);
                    }
                    deps.extend(nested_deps);
                }
            }
        }

        Ok(deps)
    }

    /// Build the full module path for a module
    fn build_module_path(&self, parent_id: ModuleId, name: &str) -> ModulePath {
        let mut segments = Vec::new();

        // Walk up to root collecting names
        let mut current = Some(parent_id);
        while let Some(id) = current {
            if let Some(module) = self.modules.get(&id) {
                segments.push(module.name.clone());
                current = module.parent;
            } else {
                break;
            }
        }

        segments.reverse();
        segments.push(name.to_string());
        ModulePath::new(segments)
    }

    /// Get a module by ID
    pub fn get(&self, id: ModuleId) -> Option<&Module> {
        self.modules.get(&id)
    }

    /// Get a module by path
    pub fn get_by_path(&self, path: &ModulePath) -> Option<&Module> {
        self.path_to_module
            .get(path)
            .and_then(|id| self.modules.get(id))
    }

    /// Get the root module
    pub fn root(&self) -> &Module {
        self.modules.get(&self.root_module).unwrap()
    }

    /// Get the root module ID
    pub fn root_id(&self) -> ModuleId {
        self.root_module
    }

    /// Detect cycles in the module graph
    ///
    /// Returns Ok(()) if no cycles, or Err with cycle information if cycles exist.
    pub fn detect_cycles(&self) -> Result<(), ModuleError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &module_id in self.modules.keys() {
            if !visited.contains(&module_id) {
                if let Some(cycle) = self.dfs_cycle_detect(module_id, &mut visited, &mut rec_stack)
                {
                    return Err(ModuleError::CyclicDependency { cycle });
                }
            }
        }

        Ok(())
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detect(
        &self,
        module_id: ModuleId,
        visited: &mut HashSet<ModuleId>,
        rec_stack: &mut HashSet<ModuleId>,
    ) -> Option<Vec<ModuleId>> {
        visited.insert(module_id);
        rec_stack.insert(module_id);

        if let Some(module) = self.modules.get(&module_id) {
            // Check dependencies
            for dep in &module.dependencies {
                // Resolve dependency target to module ID
                if let Some(&target_id) = self.path_to_module.get(&dep.target) {
                    if !visited.contains(&target_id) {
                        if let Some(cycle) = self.dfs_cycle_detect(target_id, visited, rec_stack)
                        {
                            return Some(cycle);
                        }
                    } else if rec_stack.contains(&target_id) {
                        // Found a cycle
                        let cycle = vec![target_id, module_id];
                        return Some(cycle);
                    }
                }
            }
        }

        rec_stack.remove(&module_id);
        None
    }

    /// Get all modules in topological order (dependencies before dependents)
    ///
    /// Returns None if there are cycles.
    pub fn topological_sort(&self) -> Option<Vec<ModuleId>> {
        let mut in_degree: HashMap<ModuleId, usize> = HashMap::new();
        let mut adjacency: HashMap<ModuleId, Vec<ModuleId>> = HashMap::new();

        // Initialize
        for &id in self.modules.keys() {
            in_degree.insert(id, 0);
            adjacency.insert(id, Vec::new());
        }

        // Build graph
        for (&id, module) in &self.modules {
            for dep in &module.dependencies {
                if let Some(&target_id) = self.path_to_module.get(&dep.target) {
                    adjacency.entry(target_id).or_default().push(id);
                    *in_degree.entry(id).or_insert(0) += 1;
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<ModuleId> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(id) = queue.pop_front() {
            result.push(id);

            if let Some(neighbors) = adjacency.get(&id) {
                for &neighbor in neighbors {
                    let degree = in_degree.get_mut(&neighbor).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        if result.len() == self.modules.len() {
            Some(result)
        } else {
            None // Cycle detected
        }
    }

    /// Get the number of modules
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// Check if the graph is empty
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

impl Module {
    /// Add a child module
    fn add_child(&mut self, child_id: ModuleId) {
        if !self.children.contains(&child_id) {
            self.children.push(child_id);
        }
    }

    /// Check if this module is public
    pub fn is_public(&self) -> bool {
        self.is_pub
    }
}

/// Module-related errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleError {
    /// Duplicate module declaration
    DuplicateModule {
        /// Module name
        name: String,
        /// First declaration span
        first_span: Span,
        /// Second declaration span
        second_span: Span,
    },
    /// Cyclic module dependency
    CyclicDependency {
        /// The cycle (module IDs)
        cycle: Vec<ModuleId>,
    },
    /// Module not found
    ModuleNotFound {
        /// Module path
        path: ModulePath,
        /// Usage span
        span: Span,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_graph_creation() {
        let graph = ModuleGraph::new("my_crate".to_string());
        assert_eq!(graph.len(), 1);
        assert_eq!(graph.root().name, "my_crate");
    }

    #[test]
    fn test_add_module() {
        let mut graph = ModuleGraph::new("my_crate".to_string());

        let decl = ModuleDecl {
            name: "foo".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };

        let root_id = graph.root_id();
        let module_id = graph.add_module(&decl, root_id).unwrap();

        assert_eq!(graph.len(), 2);
        let module = graph.get(module_id).unwrap();
        assert_eq!(module.name, "foo");
        assert_eq!(module.parent, Some(root_id));
    }

    #[test]
    fn test_duplicate_module_error() {
        let mut graph = ModuleGraph::new("my_crate".to_string());
        let root_id = graph.root_id();

        let decl1 = ModuleDecl {
            name: "foo".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };

        let decl2 = ModuleDecl {
            name: "foo".to_string(),
            items: None,
            is_pub: false,
            span: Span::dummy(),
        };

        graph.add_module(&decl1, root_id).unwrap();
        let result = graph.add_module(&decl2, root_id);

        assert!(matches!(result, Err(ModuleError::DuplicateModule { .. })));
    }

    #[test]
    fn test_module_path() {
        let path = ModulePath::new(vec!["std".to_string(), "io".to_string(), "Read".to_string()]);
        assert_eq!(path.last(), Some("Read"));
        assert_eq!(path.parent_path().segments, vec!["std", "io"]);
        assert!(!path.is_absolute());
    }

    #[test]
    fn test_absolute_path() {
        let path = ModulePath::new(vec!["crate".to_string(), "foo".to_string()]);
        assert!(path.is_absolute());

        let path = ModulePath::new(vec!["super".to_string(), "bar".to_string()]);
        assert!(path.is_absolute());

        let path = ModulePath::new(vec!["foo".to_string(), "bar".to_string()]);
        assert!(!path.is_absolute());
    }

    #[test]
    fn test_nested_modules() {
        let mut graph = ModuleGraph::new("my_crate".to_string());
        let root_id = graph.root_id();

        // Add foo module
        let foo_decl = ModuleDecl {
            name: "foo".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };
        let foo_id = graph.add_module(&foo_decl, root_id).unwrap();

        // Add bar module under foo
        let bar_decl = ModuleDecl {
            name: "bar".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };
        let bar_id = graph.add_module(&bar_decl, foo_id).unwrap();

        assert_eq!(graph.len(), 3);

        let bar = graph.get(bar_id).unwrap();
        assert_eq!(bar.name, "bar");
        assert_eq!(bar.parent, Some(foo_id));

        let foo = graph.get(foo_id).unwrap();
        assert!(foo.children.contains(&bar_id));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = ModuleGraph::new("my_crate".to_string());
        let root_id = graph.root_id();

        // Add modules
        let a_decl = ModuleDecl {
            name: "a".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };
        let _a_id = graph.add_module(&a_decl, root_id).unwrap();

        let b_decl = ModuleDecl {
            name: "b".to_string(),
            items: None,
            is_pub: true,
            span: Span::dummy(),
        };
        let _b_id = graph.add_module(&b_decl, root_id).unwrap();

        // Should be able to sort without dependencies
        let sorted = graph.topological_sort();
        assert!(sorted.is_some());
        assert_eq!(sorted.unwrap().len(), 3); // root + a + b
    }
}
