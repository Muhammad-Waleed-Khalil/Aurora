# Aurora AST Stability Policy

**Version**: 1.0.0
**Status**: Frozen for MVP
**Last Updated**: 2025-11-05

---

## Overview

The Aurora AST schema is **frozen for the MVP release cycle**. This document defines the stability guarantees, breaking change policy, and versioning scheme for AST nodes.

---

## Stability Guarantees

### What is Stable (1.0.0)

The following are **guaranteed stable** and will not change in incompatible ways:

1. **Node Structure**:
   - All `ExprKind`, `StmtKind`, `ItemKind`, `TypeKind`, `PatternKind` variants
   - Field names and types for all node structs
   - Node ID types (`ExprId`, `StmtId`, `ItemId`, `TypeId`, `PatternId`)

2. **Serialization Format**:
   - JSON schema exported by `schema.json`
   - Serde-derived serialization/deserialization
   - Field ordering in serialized output

3. **Traversal API**:
   - `Visitor` trait signature
   - `Arena` allocation interface
   - Preorder/postorder iteration

4. **Span Tracking**:
   - `Span` structure (file_id, start, end, line, column)
   - `HygieneId` for macro expansion

### What May Change

The following are **not guaranteed stable** during MVP:

1. **Internal Optimizations**:
   - Arena allocation strategy (bump allocator specifics)
   - Parent link storage format
   - Traversal index computation algorithm

2. **Debugging/Display**:
   - Pretty-printer output format
   - Debug representation (`Debug` trait impl)
   - Display messages

3. **Non-Public Internals**:
   - Helper functions not in public API
   - Private fields in public structs

---

## Breaking Change Policy

### Minor Version Increments (1.x.0)

**Allowed**:
- Adding new node variants (e.g., new `ExprKind` variant)
- Adding new **optional** fields to existing nodes
- Adding new helper methods to public types
- Performance improvements to traversal
- Bug fixes that don't change semantics

**Not Allowed**:
- Removing node variants
- Removing or renaming fields
- Changing field types incompatibly
- Reordering fields (breaks binary compatibility)

### Major Version Increments (2.0.0)

**Allowed**:
- Any breaking change to public API
- Removing deprecated variants/fields
- Restructuring node hierarchy
- Changing serialization format

**Process**:
1. Deprecate old API in current major version
2. Provide migration path in documentation
3. Wait at least one minor version cycle
4. Increment major version with breaking change

---

## Versioning Scheme

Aurora AST follows **Semantic Versioning 2.0**:

```
MAJOR.MINOR.PATCH
```

- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible additions
- **PATCH**: Backward-compatible bug fixes

### Current Version: 1.0.0

- **1**: First stable AST schema
- **0**: No backward-compatible additions yet
- **0**: No patches yet

---

## Deprecation Process

When deprecating AST features:

1. **Mark as deprecated** in code:
   ```rust
   #[deprecated(since = "1.2.0", note = "Use NewVariant instead")]
   pub enum OldVariant { ... }
   ```

2. **Document in CHANGELOG**:
   - What is deprecated
   - Why it's deprecated
   - What to use instead
   - When it will be removed

3. **Maintain for one minor version**:
   - Keep deprecated code for at least one 1.x release
   - Emit compiler warnings for deprecated usage

4. **Remove in next major version**:
   - Only remove in 2.0.0 or later
   - Provide automated migration tools if possible

---

## Machine-Readable Schema

The AST schema is exported as JSON in `crates/aurora_ast/schema.json`.

**Schema Format**:
```json
{
  "version": "1.0.0",
  "description": "Aurora AST Schema - Frozen for MVP",
  "node_types": [
    {
      "name": "Expr",
      "description": "Expression nodes",
      "variants": [
        {
          "name": "Binary",
          "description": "Binary operation (e.g., a + b)",
          "fields": [
            { "name": "op", "type": "BinaryOp", "optional": false },
            { "name": "left", "type": "ExprId", "optional": false },
            { "name": "right", "type": "ExprId", "optional": false }
          ]
        }
      ]
    }
  ]
}
```

**Usage**:
- **Tooling**: Parse schema to generate visitors, analyzers, transformers
- **Validation**: Verify AST nodes match expected schema
- **Documentation**: Generate API docs from schema
- **Migration**: Detect schema changes between versions

**Regenerate Schema**:
```bash
cargo run --package aurora_ast --example export_schema
```

---

## Testing Strategy

### Schema Stability Tests

1. **Golden Tests**: Serialize/deserialize AST nodes, compare to golden snapshots
2. **Roundtrip Tests**: Ensure `serialize -> deserialize` preserves all data
3. **Version Tests**: Load old schema versions, verify forward compatibility

### Compatibility Tests

1. **Schema Diff**: Compare current schema.json with previous version
2. **Breaking Change Detection**: Fail CI if incompatible changes detected
3. **Deprecation Warnings**: Ensure deprecated items emit warnings

---

## Forward Compatibility

### Adding New Nodes

When adding a new node variant (e.g., new `ExprKind::NewFeature`):

1. **Add to enum** with documentation:
   ```rust
   pub enum ExprKind {
       // ... existing variants

       /// New feature added in 1.2.0
       NewFeature { field: ExprId },
   }
   ```

2. **Update schema** by running export:
   ```bash
   cargo run --package aurora_ast --example export_schema
   ```

3. **Document in CHANGELOG**:
   ```markdown
   ## [1.2.0] - 2025-XX-XX
   ### Added
   - `ExprKind::NewFeature` - Support for XYZ syntax
   ```

4. **Add golden test** for new variant

### Backward Compatibility

Old code must continue to work:

- **Deserializing old ASTs**: New code reads old serialized ASTs
- **Visitor trait**: Default implementations for new visit methods
- **Optional fields**: New optional fields default to `None`

---

## FAQ

### Q: Can I add a required field to an existing node?

**No**. This breaks backward compatibility. Instead:
- Add an optional field
- In next major version, make it required

### Q: Can I change a field from `ExprId` to `Option<ExprId>`?

**No** in minor version. **Yes** in major version.

**Workaround**: Add a new optional field, deprecate old field, remove in 2.0.0.

### Q: Can I rename a node variant?

**No** in minor version. **Yes** in major version.

**Workaround**: Add new variant with new name, deprecate old, alias old to new.

### Q: Can I reorder enum variants?

**No**. This breaks serialization. Always add new variants at the end.

### Q: What about internal struct changes?

Internal (non-serialized) changes are allowed if they don't affect public API.

---

## Compliance Checklist

Before releasing a new AST version:

- [ ] All new nodes documented
- [ ] Schema exported and committed (`schema.json`)
- [ ] CHANGELOG updated with changes
- [ ] Breaking changes only in major versions
- [ ] Deprecation warnings for removed features
- [ ] Golden tests updated
- [ ] Forward/backward compatibility tested
- [ ] Tooling updated (parser, visitors, pretty-printer)

---

## References

- **AST Schema**: `crates/aurora_ast/schema.json`
- **Semantic Versioning**: https://semver.org/
- **Serde Versioning**: https://serde.rs/enum-representations.html

---

**Document Status**: Normative - Defines AST stability contract
**Last Reviewed**: 2025-11-05
**Next Review**: Before any schema changes
