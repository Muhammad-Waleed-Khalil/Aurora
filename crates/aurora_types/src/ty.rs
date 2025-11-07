//! Type Representation for Aurora
//!
//! This module defines the type system representation including:
//! - Primitive types (integers, floats, booleans, strings)
//! - Compound types (tuples, arrays, structs, enums)
//! - Function types with effects
//! - Generic types and type variables
//! - Typeclass constraints
//! - Type equality and subtyping

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Unique identifier for types
pub type TypeId = u32;

/// Unique identifier for type variables
pub type TypeVarId = u32;

/// Type representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// Primitive types
    Primitive(PrimitiveType),

    /// Type variable (for inference and generics)
    Var(TypeVarId),

    /// Named type (struct, enum, type alias)
    Named {
        /// Type name
        name: String,
        /// Type arguments (for generics)
        args: Vec<Type>,
    },

    /// Function type
    Function {
        /// Parameter types
        params: Vec<Type>,
        /// Return type
        ret: Box<Type>,
        /// Effect annotations
        effects: EffectSet,
    },

    /// Tuple type
    Tuple(Vec<Type>),

    /// Array type
    Array {
        /// Element type
        elem: Box<Type>,
        /// Size (None for slices)
        size: Option<usize>,
    },

    /// Reference type
    Ref {
        /// Referenced type
        inner: Box<Type>,
        /// Whether mutable
        mutable: bool,
        /// Lifetime (for borrow checking)
        lifetime: Option<Lifetime>,
    },

    /// Pointer type
    Ptr {
        /// Pointed type
        inner: Box<Type>,
        /// Whether mutable
        mutable: bool,
    },

    /// Option type (for null safety)
    Option(Box<Type>),

    /// Result type (for error handling)
    Result {
        /// Success type
        ok: Box<Type>,
        /// Error type
        err: Box<Type>,
    },

    /// Never type (!)
    Never,

    /// Unit type (())
    Unit,

    /// Universal quantification (forall)
    Forall {
        /// Bound type variables
        vars: Vec<TypeVarId>,
        /// Typeclass constraints
        constraints: Vec<Constraint>,
        /// Inner type
        inner: Box<Type>,
    },
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimitiveType {
    // Signed integers
    I8,
    I16,
    I32,
    I64,
    I128,
    ISize,

    // Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    USize,

    // Floating point
    F32,
    F64,

    // Other primitives
    Bool,
    Char,
    Str,
}

/// Effect set (bitflags for efficient representation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EffectSet {
    bits: u32,
}

impl EffectSet {
    /// Pure function (no effects)
    pub const PURE: Self = Self { bits: 0 };

    /// IO effect (file/network operations)
    pub const IO: Self = Self { bits: 1 << 0 };

    /// Allocation effect (heap allocation)
    pub const ALLOC: Self = Self { bits: 1 << 1 };

    /// Parallel effect (concurrency)
    pub const PARALLEL: Self = Self { bits: 1 << 2 };

    /// Unsafe effect (unsafe operations)
    pub const UNSAFE: Self = Self { bits: 1 << 3 };

    /// Create a new effect set
    pub fn new() -> Self {
        Self::PURE
    }

    /// Add an effect
    pub fn add(&mut self, effect: EffectSet) {
        self.bits |= effect.bits;
    }

    /// Check if has effect
    pub fn has(&self, effect: EffectSet) -> bool {
        (self.bits & effect.bits) != 0
    }

    /// Check if pure
    pub fn is_pure(&self) -> bool {
        self.bits == 0
    }

    /// Union of effects
    pub fn union(self, other: EffectSet) -> EffectSet {
        EffectSet {
            bits: self.bits | other.bits,
        }
    }

    /// Check if self is subeffect of other (self ⊆ other)
    pub fn is_subeffect_of(self, other: EffectSet) -> bool {
        (self.bits & !other.bits) == 0
    }
}

impl Default for EffectSet {
    fn default() -> Self {
        Self::PURE
    }
}

/// Lifetime (for borrow checking)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Lifetime {
    /// Lifetime name
    pub name: String,
}

impl Lifetime {
    /// Static lifetime
    pub fn static_lifetime() -> Self {
        Self {
            name: "static".to_string(),
        }
    }
}

/// Typeclass constraint
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Constraint {
    /// Typeclass name
    pub trait_name: String,
    /// Type being constrained
    pub ty: Type,
    /// Associated type bindings
    pub assoc_types: HashMap<String, Type>,
}

impl Type {
    /// Check if this is a type variable
    pub fn is_var(&self) -> bool {
        matches!(self, Type::Var(_))
    }

    /// Check if this is a function type
    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function { .. })
    }

    /// Check if this is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Primitive(_))
    }

    /// Get all type variables in this type
    pub fn free_vars(&self) -> HashSet<TypeVarId> {
        let mut vars = HashSet::new();
        self.collect_free_vars(&mut vars);
        vars
    }

    fn collect_free_vars(&self, vars: &mut HashSet<TypeVarId>) {
        match self {
            Type::Var(v) => {
                vars.insert(*v);
            }
            Type::Named { args, .. } => {
                for arg in args {
                    arg.collect_free_vars(vars);
                }
            }
            Type::Function { params, ret, .. } => {
                for param in params {
                    param.collect_free_vars(vars);
                }
                ret.collect_free_vars(vars);
            }
            Type::Tuple(types) => {
                for ty in types {
                    ty.collect_free_vars(vars);
                }
            }
            Type::Array { elem, .. } => {
                elem.collect_free_vars(vars);
            }
            Type::Ref { inner, .. } | Type::Ptr { inner, .. } => {
                inner.collect_free_vars(vars);
            }
            Type::Option(inner) => {
                inner.collect_free_vars(vars);
            }
            Type::Result { ok, err } => {
                ok.collect_free_vars(vars);
                err.collect_free_vars(vars);
            }
            Type::Forall { vars: bound, inner, constraints } => {
                // Remove bound variables
                let mut free = HashSet::new();
                inner.collect_free_vars(&mut free);
                for constraint in constraints {
                    constraint.ty.collect_free_vars(&mut free);
                    for assoc_ty in constraint.assoc_types.values() {
                        assoc_ty.collect_free_vars(&mut free);
                    }
                }
                for var in bound {
                    free.remove(var);
                }
                vars.extend(free);
            }
            Type::Primitive(_) | Type::Never | Type::Unit => {}
        }
    }

    /// Substitute type variables
    pub fn substitute(&self, subst: &HashMap<TypeVarId, Type>) -> Type {
        match self {
            Type::Var(v) => subst.get(v).cloned().unwrap_or_else(|| self.clone()),
            Type::Named { name, args } => Type::Named {
                name: name.clone(),
                args: args.iter().map(|arg| arg.substitute(subst)).collect(),
            },
            Type::Function { params, ret, effects } => Type::Function {
                params: params.iter().map(|p| p.substitute(subst)).collect(),
                ret: Box::new(ret.substitute(subst)),
                effects: *effects,
            },
            Type::Tuple(types) => Type::Tuple(
                types.iter().map(|ty| ty.substitute(subst)).collect(),
            ),
            Type::Array { elem, size } => Type::Array {
                elem: Box::new(elem.substitute(subst)),
                size: *size,
            },
            Type::Ref { inner, mutable, lifetime } => Type::Ref {
                inner: Box::new(inner.substitute(subst)),
                mutable: *mutable,
                lifetime: lifetime.clone(),
            },
            Type::Ptr { inner, mutable } => Type::Ptr {
                inner: Box::new(inner.substitute(subst)),
                mutable: *mutable,
            },
            Type::Option(inner) => Type::Option(Box::new(inner.substitute(subst))),
            Type::Result { ok, err } => Type::Result {
                ok: Box::new(ok.substitute(subst)),
                err: Box::new(err.substitute(subst)),
            },
            Type::Forall { vars, constraints, inner } => {
                // Remove bound variables from substitution
                let mut new_subst = subst.clone();
                for var in vars {
                    new_subst.remove(var);
                }
                Type::Forall {
                    vars: vars.clone(),
                    constraints: constraints
                        .iter()
                        .map(|c| Constraint {
                            trait_name: c.trait_name.clone(),
                            ty: c.ty.substitute(&new_subst),
                            assoc_types: c
                                .assoc_types
                                .iter()
                                .map(|(k, v)| (k.clone(), v.substitute(&new_subst)))
                                .collect(),
                        })
                        .collect(),
                    inner: Box::new(inner.substitute(&new_subst)),
                }
            }
            Type::Primitive(_) | Type::Never | Type::Unit => self.clone(),
        }
    }

    /// Occurs check: does this type variable occur in the type?
    /// This prevents infinite types like `T = List<T>`
    pub fn occurs(&self, var: TypeVarId) -> bool {
        match self {
            Type::Var(v) => *v == var,
            Type::Named { args, .. } => args.iter().any(|arg| arg.occurs(var)),
            Type::Function { params, ret, .. } => {
                params.iter().any(|p| p.occurs(var)) || ret.occurs(var)
            }
            Type::Tuple(types) => types.iter().any(|ty| ty.occurs(var)),
            Type::Array { elem, .. } => elem.occurs(var),
            Type::Ref { inner, .. } | Type::Ptr { inner, .. } => inner.occurs(var),
            Type::Option(inner) => inner.occurs(var),
            Type::Result { ok, err } => ok.occurs(var) || err.occurs(var),
            Type::Forall { vars, inner, constraints } => {
                if vars.contains(&var) {
                    false // Bound variable
                } else {
                    inner.occurs(var)
                        || constraints.iter().any(|c| {
                            c.ty.occurs(var)
                                || c.assoc_types.values().any(|t| t.occurs(var))
                        })
                }
            }
            Type::Primitive(_) | Type::Never | Type::Unit => false,
        }
    }
}

/// Type equality implementation
impl Type {
    /// Check if two types are equal (structural equality)
    pub fn equals(&self, other: &Type) -> bool {
        self == other
    }

    /// Check if self is a subtype of other
    pub fn is_subtype_of(&self, other: &Type) -> bool {
        match (self, other) {
            // Reflexivity
            (a, b) if a == b => true,

            // Never is subtype of everything
            (Type::Never, _) => true,

            // Function subtyping (contravariant in parameters, covariant in return)
            (
                Type::Function {
                    params: params1,
                    ret: ret1,
                    effects: eff1,
                },
                Type::Function {
                    params: params2,
                    ret: ret2,
                    effects: eff2,
                },
            ) => {
                // Contravariant in parameters
                params1.len() == params2.len()
                    && params1
                        .iter()
                        .zip(params2.iter())
                        .all(|(p1, p2)| p2.is_subtype_of(p1))
                    // Covariant in return
                    && ret1.is_subtype_of(ret2)
                    // Subeffecting
                    && eff1.is_subeffect_of(*eff2)
            }

            // Tuple subtyping (covariant)
            (Type::Tuple(types1), Type::Tuple(types2)) => {
                types1.len() == types2.len()
                    && types1
                        .iter()
                        .zip(types2.iter())
                        .all(|(t1, t2)| t1.is_subtype_of(t2))
            }

            // Array subtyping (covariant in element type, equal size)
            (
                Type::Array {
                    elem: elem1,
                    size: size1,
                },
                Type::Array {
                    elem: elem2,
                    size: size2,
                },
            ) => size1 == size2 && elem1.is_subtype_of(elem2),

            // Reference subtyping
            (
                Type::Ref {
                    inner: inner1,
                    mutable: mut1,
                    ..
                },
                Type::Ref {
                    inner: inner2,
                    mutable: mut2,
                    ..
                },
            ) => {
                // Immutable ref is covariant
                if !mut1 && !mut2 {
                    inner1.is_subtype_of(inner2)
                }
                // Mutable ref is invariant
                else if mut1 == mut2 {
                    inner1 == inner2
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}

/// Display implementation for types
impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => write!(f, "{}", p),
            Type::Var(v) => write!(f, "'{}", v),
            Type::Named { name, args } => {
                write!(f, "{}", name)?;
                if !args.is_empty() {
                    write!(f, "<")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", arg)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
            Type::Function { params, ret, effects } => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", ret)?;
                if !effects.is_pure() {
                    write!(f, " + {:?}", effects)?;
                }
                Ok(())
            }
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Array { elem, size } => {
                write!(f, "[")?;
                write!(f, "{}", elem)?;
                if let Some(size) = size {
                    write!(f, "; {}", size)?;
                }
                write!(f, "]")
            }
            Type::Ref { inner, mutable, .. } => {
                if *mutable {
                    write!(f, "&mut {}", inner)
                } else {
                    write!(f, "&{}", inner)
                }
            }
            Type::Ptr { inner, mutable } => {
                if *mutable {
                    write!(f, "*mut {}", inner)
                } else {
                    write!(f, "*const {}", inner)
                }
            }
            Type::Option(inner) => write!(f, "Option<{}>", inner),
            Type::Result { ok, err } => write!(f, "Result<{}, {}>", ok, err),
            Type::Never => write!(f, "!"),
            Type::Unit => write!(f, "()"),
            Type::Forall { vars, constraints, inner } => {
                write!(f, "forall")?;
                for var in vars {
                    write!(f, " '{}", var)?;
                }
                if !constraints.is_empty() {
                    write!(f, " where")?;
                    for (i, c) in constraints.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, " {}: {}", c.ty, c.trait_name)?;
                    }
                }
                write!(f, ". {}", inner)
            }
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveType::I8 => write!(f, "i8"),
            PrimitiveType::I16 => write!(f, "i16"),
            PrimitiveType::I32 => write!(f, "i32"),
            PrimitiveType::I64 => write!(f, "i64"),
            PrimitiveType::I128 => write!(f, "i128"),
            PrimitiveType::ISize => write!(f, "isize"),
            PrimitiveType::U8 => write!(f, "u8"),
            PrimitiveType::U16 => write!(f, "u16"),
            PrimitiveType::U32 => write!(f, "u32"),
            PrimitiveType::U64 => write!(f, "u64"),
            PrimitiveType::U128 => write!(f, "u128"),
            PrimitiveType::USize => write!(f, "usize"),
            PrimitiveType::F32 => write!(f, "f32"),
            PrimitiveType::F64 => write!(f, "f64"),
            PrimitiveType::Bool => write!(f, "bool"),
            PrimitiveType::Char => write!(f, "char"),
            PrimitiveType::Str => write!(f, "str"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_types() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let bool_ty = Type::Primitive(PrimitiveType::Bool);

        assert!(i32_ty.is_primitive());
        assert!(bool_ty.is_primitive());
        assert_eq!(i32_ty.to_string(), "i32");
        assert_eq!(bool_ty.to_string(), "bool");
    }

    #[test]
    fn test_type_equality() {
        let t1 = Type::Primitive(PrimitiveType::I32);
        let t2 = Type::Primitive(PrimitiveType::I32);
        let t3 = Type::Primitive(PrimitiveType::I64);

        assert!(t1.equals(&t2));
        assert!(!t1.equals(&t3));
    }

    #[test]
    fn test_occurs_check() {
        let var0 = Type::Var(0);
        let _var1 = Type::Var(1);
        let list = Type::Named {
            name: "List".to_string(),
            args: vec![var0.clone()],
        };

        assert!(var0.occurs(0));
        assert!(!var0.occurs(1));
        assert!(list.occurs(0));
        assert!(!list.occurs(1));
    }

    #[test]
    fn test_free_vars() {
        let var0 = Type::Var(0);
        let var1 = Type::Var(1);
        let func = Type::Function {
            params: vec![var0.clone()],
            ret: Box::new(var1.clone()),
            effects: EffectSet::PURE,
        };

        let vars = func.free_vars();
        assert_eq!(vars.len(), 2);
        assert!(vars.contains(&0));
        assert!(vars.contains(&1));
    }

    #[test]
    fn test_substitution() {
        let var0 = Type::Var(0);
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        let mut subst = HashMap::new();
        subst.insert(0, i32_ty.clone());

        let result = var0.substitute(&subst);
        assert_eq!(result, i32_ty);
    }

    #[test]
    fn test_subtyping_reflexivity() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        assert!(i32_ty.is_subtype_of(&i32_ty));
    }

    #[test]
    fn test_never_subtype() {
        let never = Type::Never;
        let i32_ty = Type::Primitive(PrimitiveType::I32);

        assert!(never.is_subtype_of(&i32_ty));
        assert!(!i32_ty.is_subtype_of(&never));
    }

    #[test]
    fn test_function_subtyping() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let i64_ty = Type::Primitive(PrimitiveType::I64);

        // fn(i32) -> i32
        let f1 = Type::Function {
            params: vec![i32_ty.clone()],
            ret: Box::new(i32_ty.clone()),
            effects: EffectSet::PURE,
        };

        // fn(i64) -> i32 is NOT a subtype of fn(i32) -> i32
        let f2 = Type::Function {
            params: vec![i64_ty.clone()],
            ret: Box::new(i32_ty.clone()),
            effects: EffectSet::PURE,
        };

        assert!(!f2.is_subtype_of(&f1));
    }

    #[test]
    fn test_effect_set() {
        let mut effects = EffectSet::new();
        assert!(effects.is_pure());

        effects.add(EffectSet::IO);
        assert!(effects.has(EffectSet::IO));
        assert!(!effects.is_pure());

        effects.add(EffectSet::ALLOC);
        assert!(effects.has(EffectSet::ALLOC));

        // Subeffecting
        let pure = EffectSet::PURE;
        let io_only = EffectSet::IO;
        let io_alloc = EffectSet::IO.union(EffectSet::ALLOC);

        assert!(pure.is_subeffect_of(io_only));
        assert!(io_only.is_subeffect_of(io_alloc));
        assert!(!io_alloc.is_subeffect_of(io_only));
    }

    #[test]
    fn test_tuple_type() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let bool_ty = Type::Primitive(PrimitiveType::Bool);

        let tuple = Type::Tuple(vec![i32_ty, bool_ty]);
        assert_eq!(tuple.to_string(), "(i32, bool)");
    }

    #[test]
    fn test_array_type() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let array = Type::Array {
            elem: Box::new(i32_ty),
            size: Some(10),
        };

        assert_eq!(array.to_string(), "[i32; 10]");
    }

    #[test]
    fn test_option_type() {
        let i32_ty = Type::Primitive(PrimitiveType::I32);
        let option = Type::Option(Box::new(i32_ty));

        assert_eq!(option.to_string(), "Option<i32>");
    }
}
