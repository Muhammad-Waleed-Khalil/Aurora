# Aurora AST Stability Policy

**Version**: 1.0.0  
**Status**: Frozen for MVP  
**Last Updated**: 2025-11-07

---

## Overview

The Aurora Abstract Syntax Tree (AST) schema is a critical interface between the parser and all downstream compiler phases (name resolution, type checking, code generation). This document defines the stability guarantees and versioning policy for the AST schema.

---

## Versioning

The AST schema follows [Semantic Versioning 2.0.0](https://semver.org/):

- **Major version** (`1.x.x`): Breaking changes to existing node types
- **Minor version** (`x.1.x`): Non-breaking additions (new optional fields, new node types)
- **Patch version** (`x.x.1`): Bug fixes, documentation updates, implementation optimizations

---

## Breaking Changes

The following changes are **breaking** and require a major version bump:

1. **Removing a field** from an existing node type
2. **Changing the type** of an existing field
3. **Renaming a field** in an existing node type
4. **Removing a node variant** from an enum
5. **Changing the semantics** of a node type
6. **Reordering fields** in a way that breaks serialization compatibility

### Breaking Change Process

1. **Announce** the breaking change with a deprecation notice in the current version
2. **Document** the migration path in `MIGRATION.md`
3. **Bump** the major version number
4. **Update** all downstream code (name resolution, type checking, etc.)
5. **Test** the migration with the full test suite

---

## Non-Breaking Changes

The following changes are **non-breaking** and only require a minor version bump:

1. **Adding a new optional field** with a default value
2. **Adding a new node type variant** to an enum
3. **Adding a new traversal method** to the visitor
4. **Performance optimizations** that preserve semantics
5. **Documentation improvements**

---

## Frozen for MVP

For the MVP (v1.0.0), the AST schema is **frozen**. This means:

- ✅ **All node types are finalized**
- ✅ **All field names and types are stable**
- ✅ **Serialization format is fixed**
- ✅ **Backward compatibility is guaranteed**

---

## Schema Guarantees

### 1. Serialization Stability

All AST nodes are serializable to JSON via `serde`. The JSON format is stable and will not change without a major version bump.

**Example**:
```json
{
  "kind": {
    "Binary": {
      "op": "Add",
      "left": 0,
      "right": 1
    }
  },
  "span": {
    "file_id": 1,
    "start": 0,
    "end": 5,
    "line": 1,
    "column": 1
  },
  "hygiene": {"id": 0}
}
```

### 2. Arena Stability

- Node IDs (u32 indices) remain stable within a single compilation unit
- Parent links are computed once and cached
- Preorder/postorder indices enable O(1) ancestor queries

### 3. Traversal Stability

- The `Visitor` trait provides a stable interface for AST traversal
- Preorder and postorder iterators are guaranteed to visit all nodes exactly once
- Iterative traversal avoids stack overflow on deep ASTs

---

## Tooling Integration

The AST schema is designed for integration with external tools:

- **IDE plugins**: LSP server uses AST for completions, go-to-definition, etc.
- **Linters**: Static analysis tools traverse the AST for code quality checks
- **Code formatters**: Pretty-printer reconstructs source code from AST
- **Macro expanders**: Hygiene system prevents variable capture

---

## Testing Strategy

To ensure stability, the AST schema is tested with:

1. **Unit tests**: Each node type has construction and serialization tests
2. **Golden tests**: AST snapshots are compared against reference outputs
3. **Property tests**: Fuzzing generates random ASTs and checks invariants
4. **Differential tests**: Compare AST structure against reference parser

---

## Migration Guide

When a breaking change is necessary:

1. **Old schema**: Keep the old schema in `schema_v1.json`
2. **New schema**: Create the new schema in `schema_v2.json`
3. **Migration tool**: Provide a tool to convert v1 ASTs to v2 ASTs
4. **Documentation**: Document all changes in `MIGRATION.md`

---

## Schema File

The machine-readable schema is available at:

```
crates/aurora_ast/schema.json
```

This file is versioned and updated with each release.

---

## Questions?

For questions about the AST schema or stability policy, please open an issue on GitHub or consult the [Architecture Guide](architecture.md).

---

**Document Status**: Living policy, reviewed at major version bumps  
**Next Review**: Upon completion of MVP self-hosting
