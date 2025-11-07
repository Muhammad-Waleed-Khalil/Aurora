# Aurora Grammar Specification

**Version**: 1.0.0
**Status**: Phase 1 Complete
**Last Updated**: 2025-11-05

---

## Overview

Aurora's grammar is designed to be:
- **LL(1)** for declarations and statements
- **Pratt-parsed** for expressions with 16 precedence levels
- **Unambiguous** with zero shift-reduce or reduce-reduce conflicts
- **Deterministic** for reliable parsing by agents and tools

---

## Grammar Notation

This specification uses Extended BNF (EBNF):
- `'keyword'` - Terminal symbol (literal token)
- `NonTerminal` - Non-terminal symbol (grammar rule)
- `[ optional ]` - Optional element (zero or one)
- `{ repeated }` - Zero or more repetitions
- `{ repeated }+` - One or more repetitions
- `( group )` - Grouping
- `|` - Alternation

---

## Top-Level Structure

```ebnf
Program ::=
      { Item }  // Zero or more top-level items

Item ::=
      FunctionDecl   // Function declaration
    | TypeDecl       // Type declaration
    | TraitDecl      // Trait declaration
    | ImplDecl       // Implementation declaration
    | ConstDecl      // Constant declaration
    | ModDecl        // Module declaration
    | UseDecl        // Use/import declaration
```

---

## Declarations

### Function Declaration

```ebnf
FunctionDecl ::=
      [ 'pub' ] [ 'async' ] 'fn' IDENT [ GenericParams ] '(' [ ParamList ] ')' [ '->' Type ] [ WhereClause ] Block

ParamList ::=
      Param { ',' Param } [ ',' ]

Param ::=
      [ 'mut' ] Pattern ':' Type

GenericParams ::=
      '<' IDENT { ',' IDENT } '>'

WhereClause ::=
      'where' TypeBound { ',' TypeBound }
```

**Example**:
```aurora
pub async fn fetch<T>(url: String) -> Result<T, Error> where T: Deserialize {
    // ...
}
```

### Type Declaration

```ebnf
TypeDecl ::=
      [ 'pub' ] 'type' IDENT [ GenericParams ] '=' Type ';'
```

**Example**:
```aurora
pub type UserId = i64;
type Point<T> = (T, T);
```

### Trait Declaration

```ebnf
TraitDecl ::=
      [ 'pub' ] 'trait' IDENT [ GenericParams ] [ WhereClause ] '{' { TraitItem } '}'

TraitItem ::=
      FunctionDecl
    | TypeDecl
    | ConstDecl
```

**Example**:
```aurora
pub trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}
```

### Implementation Declaration

```ebnf
ImplDecl ::=
      'impl' [ GenericParams ] Type [ 'for' Type ] [ WhereClause ] '{' { ImplItem } '}'

ImplItem ::=
      FunctionDecl
    | ConstDecl
```

**Example**:
```aurora
impl<T> Iterator<T> for Vec<T> {
    fn next(&mut self) -> Option<T> { ... }
}
```

### Constant Declaration

```ebnf
ConstDecl ::=
      [ 'pub' ] 'const' IDENT ':' Type '=' Expr ';'
```

**Example**:
```aurora
pub const MAX_SIZE: i64 = 1024;
```

### Module Declaration

```ebnf
ModDecl ::=
      [ 'pub' ] 'mod' IDENT ';'
    | [ 'pub' ] 'mod' IDENT '{' { Item } '}'
```

### Use Declaration

```ebnf
UseDecl ::=
      [ 'pub' ] 'use' Path ';'
```

---

## Statements

```ebnf
Statement ::=
      LetStmt
    | ExprStmt
    | Item

LetStmt ::=
      'let' [ 'mut' ] Pattern [ ':' Type ] [ '=' Expr ] ';'

ExprStmt ::=
      Expr [ ';' ]

Block ::=
      '{' { Statement } [ Expr ] '}'
```

**Examples**:
```aurora
let x: i64 = 42;
let mut y = vec![1, 2, 3];
x + y;
{
    let z = 10;
    z * 2
}
```

---

## Types

```ebnf
Type ::=
      PrimitiveType
    | PathType
    | TupleType
    | ArrayType
    | FunctionType
    | ReferenceType

PrimitiveType ::=
      'i8' | 'i16' | 'i32' | 'i64'
    | 'u8' | 'u16' | 'u32' | 'u64'
    | 'f32' | 'f64'
    | 'bool' | 'char' | 'str'

PathType ::=
      Path [ GenericArgs ]

TupleType ::=
      '(' [ Type { ',' Type } ] ')'

ArrayType ::=
      '[' Type ';' CONST_EXPR ']'
    | '[' Type ']'

FunctionType ::=
      'fn' '(' [ Type { ',' Type } ] ')' [ '->' Type ]
```

**Examples**:
```aurora
i64
String
(i64, String, bool)
[u8; 1024]
fn(i64, i64) -> i64
```

---

## Patterns

```ebnf
Pattern ::=
      IDENT                    // Identifier pattern
    | '_'                      // Wildcard pattern
    | LITERAL                  // Literal pattern
    | TuplePattern             // Tuple pattern
    | StructPattern            // Struct pattern
    | EnumPattern              // Enum variant pattern

TuplePattern ::=
      '(' Pattern { ',' Pattern } ')'

StructPattern ::=
      Path '{' [ FieldPattern { ',' FieldPattern } ] '}'

FieldPattern ::=
      IDENT [ ':' Pattern ]
```

**Examples**:
```aurora
x
_
42
(x, y, _)
Point { x, y }
Some(value)
```

---

## Expressions

Expressions in Aurora use **Pratt parsing** with the precedence table defined below.

### Expression Forms

```ebnf
Expr ::=
      PrimaryExpr
    | UnaryExpr
    | BinaryExpr
    | CallExpr
    | IndexExpr
    | FieldExpr
    | IfExpr
    | MatchExpr
    | LoopExpr
    | BlockExpr

PrimaryExpr ::=
      LITERAL
    | IDENT
    | '(' Expr ')'
    | Block
    | IfExpr
    | MatchExpr
    | LoopExpr

IfExpr ::=
      'if' Expr Block [ 'else' ( Block | IfExpr ) ]

MatchExpr ::=
      'match' Expr '{' { MatchArm } '}'

MatchArm ::=
      Pattern [ 'if' Expr ] '=>' ( Expr ',' | Block )

LoopExpr ::=
      'loop' Block
    | 'while' Expr Block
    | 'for' Pattern 'in' Expr Block
```

---

## Operator Precedence

Aurora defines **16 precedence levels** (1 = lowest, 16 = highest):

| Level | Operators | Associativity | Description |
|-------|-----------|---------------|-------------|
| 1 | `=` `+=` `-=` `*=` `/=` `%=` `&=` `\|=` `^=` `<<=` `>>=` | Right | Assignment |
| 2 | `\|>` `<\|` | Left | Pipeline |
| 3 | `..` `..=` `...` | None | Range |
| 4 | `\|\|` | Left | Logical OR |
| 5 | `&&` | Left | Logical AND |
| 6 | `==` `!=` `<` `>` `<=` `>=` | None | Comparison |
| 7 | `\|` | Left | Bitwise OR |
| 8 | `^` | Left | Bitwise XOR |
| 9 | `&` | Left | Bitwise AND |
| 10 | `<<` `>>` | Left | Bit shift |
| 11 | `+` `-` | Left | Addition/Subtraction |
| 12 | `*` `/` `%` | Left | Multiplication/Division |
| 13 | `**` | Right | Exponentiation |
| 14 | `!` `-` `~` | Right | Unary |
| 15 | `.` `?` `??` `()` `[]` | Left | Postfix/Access |
| 16 | `::` `->` `=>` | Left | Path/Arrow |

### Precedence Examples

```aurora
a = b = c           // Right: a = (b = c)
a + b * c           // Left:  a + (b * c)
a ** b ** c         // Right: a ** (b ** c)
a < b < c           // Error: non-associative
x |> f |> g         // Left:  (x |> f) |> g
a && b || c         // Left:  (a && b) || c
```

---

## Comments

```ebnf
LineComment ::=
      '//' [any character except newline]* NEWLINE

BlockComment ::=
      '/*' [any character]* '*/'

DocCommentOuter ::=
      '///' [any character except newline]* NEWLINE

DocCommentInner ::=
      '//!' [any character except newline]* NEWLINE
```

**Example**:
```aurora
// This is a line comment

/* This is a
   block comment */

/// This documents the following item
pub fn documented() {}

//! This documents the containing module
```

---

## Keywords

### Control Flow
`if`, `else`, `match`, `for`, `while`, `loop`, `break`, `continue`, `return`, `yield`

### Declarations
`fn`, `let`, `mut`, `const`, `static`, `type`, `trait`, `impl`

### Modules & Visibility
`use`, `mod`, `pub`, `as`

### Special
`self`, `Self`, `super`, `crate`, `async`, `await`, `defer`, `unsafe`, `comptime`

### Literals & Values
`true`, `false`, `Some`, `None`, `Ok`, `Err`, `unreachable`

---

## Conflict Analysis

Aurora's grammar has been analyzed and verified to be **conflict-free**:

- ✅ **Zero shift-reduce conflicts**
- ✅ **Zero reduce-reduce conflicts**
- ✅ **Zero FIRST-FIRST conflicts** (LL(1) for declarations)
- ✅ **No left recursion** in declaration grammar
- ✅ **Pratt parsing** eliminates expression ambiguity

### Analysis Tools

- FIRST/FOLLOW set computation
- LL(1) conflict detector
- Left recursion checker
- Precedence validation

---

## Grammar Guarantees

Aurora guarantees:

1. **Deterministic parsing**: Every valid program has exactly one parse tree
2. **Unambiguous tokens**: Maximal-munch with priority rules
3. **LL(1) declarations**: Top-down parsing without backtracking
4. **Pratt expressions**: Efficient precedence climbing
5. **Agent-friendly**: Machine-readable JSON exports available

---

## Machine-Readable Exports

The grammar is available in multiple formats:

- **JSON**: Complete grammar rules with productions
- **BNF**: Human-readable EBNF notation
- **Precedence Table**: JSON export with all operators
- **Conflict Report**: FIRST/FOLLOW sets and analysis results

---

## References

- [Aurora Language Specification](spec.md)
- [Precedence Table JSON](../crates/aurora_grammar/precedence.json)
- [Conflict Analysis Report](../crates/aurora_grammar/conflict_report.json)
- [Token Catalog](../crates/aurora_lexer/token_catalog.json)

---

**Document Status**: Complete - Phase 1 Grammar Implementation
