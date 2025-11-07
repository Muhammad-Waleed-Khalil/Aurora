//! Expression AST nodes
//!
//! This module defines all expression forms in Aurora, from simple literals
//! to complex control flow expressions.

use crate::span::{HygieneId, Span};
use serde::{Deserialize, Serialize};

/// Expression node ID (index into arena)
pub type ExprId = u32;

/// An expression in the AST
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expr {
    /// The expression kind
    pub kind: ExprKind,
    /// Source location
    pub span: Span,
    /// Hygiene context for macro expansion
    pub hygiene: HygieneId,
}

/// Expression kinds
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExprKind {
    // Literals
    /// Integer literal (e.g., `42`, `0x2A`)
    Literal(Literal),

    /// Identifier reference (e.g., `x`, `foo`)
    Ident(String),

    /// Path reference (e.g., `std::io::Read`)
    Path(Path),

    // Operators
    /// Unary operation (e.g., `-x`, `!flag`, `~bits`)
    Unary {
        /// The operator
        op: UnaryOp,
        /// The operand expression ID
        operand: ExprId,
    },

    /// Binary operation (e.g., `a + b`, `x == y`)
    Binary {
        /// The operator
        op: BinaryOp,
        /// Left operand expression ID
        left: ExprId,
        /// Right operand expression ID
        right: ExprId,
    },

    // Calls and access
    /// Function call (e.g., `foo(a, b)`)
    Call {
        /// Function expression ID
        func: ExprId,
        /// Argument expression IDs
        args: Vec<ExprId>,
    },

    /// Method call (e.g., `x.method(a, b)`)
    MethodCall {
        /// Receiver expression ID
        receiver: ExprId,
        /// Method name
        method: String,
        /// Argument expression IDs
        args: Vec<ExprId>,
    },

    /// Field access (e.g., `obj.field`)
    Field {
        /// Object expression ID
        object: ExprId,
        /// Field name
        field: String,
    },

    /// Index access (e.g., `arr[i]`)
    Index {
        /// Collection expression ID
        collection: ExprId,
        /// Index expression ID
        index: ExprId,
    },

    // Pipelines
    /// Pipeline operator (e.g., `x |> f |> g`)
    Pipeline {
        /// Left operand expression ID
        left: ExprId,
        /// Right operand expression ID
        right: ExprId,
    },

    // Control flow
    /// If expression (e.g., `if cond { a } else { b }`)
    If {
        /// Condition expression ID
        condition: ExprId,
        /// Then block
        then_block: BlockId,
        /// Optional else block or else-if
        else_block: Option<BlockId>,
    },

    /// Match expression
    Match {
        /// Scrutinee expression ID
        scrutinee: ExprId,
        /// Match arms
        arms: Vec<MatchArm>,
    },

    /// Loop expression (e.g., `loop { ... }`)
    Loop {
        /// Loop body
        body: BlockId,
    },

    /// While loop (e.g., `while cond { ... }`)
    While {
        /// Condition expression ID
        condition: ExprId,
        /// Loop body
        body: BlockId,
    },

    /// For loop (e.g., `for x in iter { ... }`)
    For {
        /// Pattern to bind
        pattern: PatternId,
        /// Iterator expression ID
        iterator: ExprId,
        /// Loop body
        body: BlockId,
    },

    /// Return expression (e.g., `return x`)
    Return {
        /// Optional return value expression ID
        value: Option<ExprId>,
    },

    /// Break expression (e.g., `break`, `break value`)
    Break {
        /// Optional break value expression ID
        value: Option<ExprId>,
    },

    /// Continue expression
    Continue,

    /// Yield expression (for generators)
    Yield {
        /// Value to yield
        value: ExprId,
    },

    // Compound expressions
    /// Block expression (e.g., `{ stmt; stmt; expr }`)
    Block(BlockId),

    /// Tuple expression (e.g., `(a, b, c)`)
    Tuple(Vec<ExprId>),

    /// Array expression (e.g., `[a, b, c]`)
    Array(Vec<ExprId>),

    /// Struct literal (e.g., `Point { x: 1, y: 2 }`)
    Struct {
        /// Path to struct type
        path: Path,
        /// Field initializers
        fields: Vec<FieldInit>,
    },

    /// Range expression (e.g., `0..10`, `0..=10`)
    Range {
        /// Start expression ID (optional for `..end`)
        start: Option<ExprId>,
        /// End expression ID (optional for `start..`)
        end: Option<ExprId>,
        /// Whether range is inclusive (`..=`)
        inclusive: bool,
    },

    /// Try expression (e.g., `expr?`)
    Try {
        /// Expression to try
        expr: ExprId,
    },

    /// Await expression (e.g., `expr.await`)
    Await {
        /// Future expression
        expr: ExprId,
    },

    /// Unsafe block
    Unsafe {
        /// Unsafe block
        block: BlockId,
    },

    /// Comptime expression (compile-time evaluation)
    Comptime {
        /// Expression to evaluate at compile time
        expr: ExprId,
    },
}

/// Block ID (index into arena)
pub type BlockId = u32;

/// Pattern ID (index into arena)
pub type PatternId = u32;

/// Literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal
    Int(i64),
    /// Float literal
    Float(f64),
    /// String literal
    String(String),
    /// Character literal
    Char(char),
    /// Boolean literal
    Bool(bool),
}

/// Unary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Negation (`-`)
    Neg,
    /// Logical NOT (`!`)
    Not,
    /// Bitwise NOT (`~`)
    BitNot,
    /// Dereference (`*`)
    Deref,
    /// Reference (`&`)
    Ref,
    /// Mutable reference (`&mut`)
    RefMut,
}

/// Binary operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    // Arithmetic
    /// Addition (`+`)
    Add,
    /// Subtraction (`-`)
    Sub,
    /// Multiplication (`*`)
    Mul,
    /// Division (`/`)
    Div,
    /// Remainder (`%`)
    Rem,
    /// Exponentiation (`**`)
    Pow,

    // Comparison
    /// Equal (`==`)
    Eq,
    /// Not equal (`!=`)
    Ne,
    /// Less than (`<`)
    Lt,
    /// Less than or equal (`<=`)
    Le,
    /// Greater than (`>`)
    Gt,
    /// Greater than or equal (`>=`)
    Ge,

    // Logical
    /// Logical AND (`&&`)
    And,
    /// Logical OR (`||`)
    Or,

    // Bitwise
    /// Bitwise AND (`&`)
    BitAnd,
    /// Bitwise OR (`|`)
    BitOr,
    /// Bitwise XOR (`^`)
    BitXor,
    /// Left shift (`<<`)
    Shl,
    /// Right shift (`>>`)
    Shr,

    // Assignment
    /// Assignment (`=`)
    Assign,
    /// Add-assign (`+=`)
    AddAssign,
    /// Subtract-assign (`-=`)
    SubAssign,
    /// Multiply-assign (`*=`)
    MulAssign,
    /// Divide-assign (`/=`)
    DivAssign,
    /// Remainder-assign (`%=`)
    RemAssign,
    /// Bitwise AND-assign (`&=`)
    BitAndAssign,
    /// Bitwise OR-assign (`|=`)
    BitOrAssign,
    /// Bitwise XOR-assign (`^=`)
    BitXorAssign,
    /// Left shift-assign (`<<=`)
    ShlAssign,
    /// Right shift-assign (`>>=`)
    ShrAssign,
}

/// Path (e.g., `std::io::Read`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Path {
    /// Path segments
    pub segments: Vec<String>,
    /// Generic arguments (if any)
    pub generics: Vec<GenericArg>,
}

/// Generic argument
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GenericArg {
    /// Type argument
    Type(TypeId),
    /// Const argument
    Const(ExprId),
}

/// Type ID (index into arena)
pub type TypeId = u32;

/// Match arm
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    /// Pattern to match
    pub pattern: PatternId,
    /// Optional guard condition
    pub guard: Option<ExprId>,
    /// Arm body expression
    pub body: ExprId,
    /// Source span
    pub span: Span,
}

/// Field initializer for struct literals
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldInit {
    /// Field name
    pub name: String,
    /// Field value expression ID
    pub value: ExprId,
    /// Source span
    pub span: Span,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_creation() {
        let lit = Literal::Int(42);
        assert_eq!(lit, Literal::Int(42));

        let lit = Literal::String("hello".to_string());
        assert_eq!(lit, Literal::String("hello".to_string()));
    }

    #[test]
    fn test_binary_op() {
        let op = BinaryOp::Add;
        assert_eq!(op, BinaryOp::Add);
    }

    #[test]
    fn test_path() {
        let path = Path {
            segments: vec!["std".to_string(), "io".to_string()],
            generics: vec![],
        };
        assert_eq!(path.segments.len(), 2);
    }
}
