//! Typeclass System for Aurora
//!
//! This module implements typeclasses (traits) with:
//! - Typeclass definitions with associated types
//! - Implementation tracking
//! - Coherence checking (one impl per type-trait pair)
//! - Typeclass resolution
//! - Supertraits
//!
//! # Coherence
//!
//! Aurora enforces the orphan rule and overlap checking:
//! - Each (trait, type) pair can have at most one implementation
//! - Either the trait or the type must be defined in the current crate

use crate::ty::{Type, TypeVarId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Unique identifier for typeclasses
pub type TraitId = u32;

/// Unique identifier for implementations
pub type ImplId = u32;

/// Typeclass (trait) definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trait {
    /// Trait ID
    pub id: TraitId,
    /// Trait name
    pub name: String,
    /// Type parameters
    pub type_params: Vec<TypeVarId>,
    /// Supertraits
    pub supertraits: Vec<TraitId>,
    /// Associated types
    pub assoc_types: Vec<AssocType>,
    /// Required methods
    pub methods: Vec<MethodSignature>,
}

/// Associated type in a trait
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssocType {
    /// Type name
    pub name: String,
    /// Optional default
    pub default: Option<Type>,
    /// Bounds on the associated type
    pub bounds: Vec<TraitBound>,
}

/// Trait bound
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitBound {
    /// Trait being required
    pub trait_id: TraitId,
    /// Type arguments for the trait
    pub args: Vec<Type>,
}

/// Method signature in a trait
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MethodSignature {
    /// Method name
    pub name: String,
    /// Type parameters
    pub type_params: Vec<TypeVarId>,
    /// Parameter types
    pub params: Vec<Type>,
    /// Return type
    pub ret: Type,
}

/// Trait implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitImpl {
    /// Implementation ID
    pub id: ImplId,
    /// Trait being implemented
    pub trait_id: TraitId,
    /// Type being implemented for
    pub self_type: Type,
    /// Type parameter instantiations
    pub type_args: Vec<Type>,
    /// Associated type definitions
    pub assoc_type_defs: HashMap<String, Type>,
    /// Where the impl is defined
    pub defining_crate: String,
}

/// Typeclass resolution error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TraitError {
    /// No implementation found
    #[error("No implementation of trait {0} for type {1}")]
    NoImpl(String, String),

    /// Ambiguous implementation
    #[error("Ambiguous implementation of trait {0} for type {1}")]
    Ambiguous(String, String),

    /// Coherence violation (overlapping impls)
    #[error("Coherence violation: overlapping implementations of {0}")]
    Coherence(String),

    /// Orphan rule violation
    #[error("Orphan rule violation: cannot implement {0} for {1} in this crate")]
    OrphanRule(String, String),

    /// Missing associated type
    #[error("Missing associated type {0} in implementation")]
    MissingAssocType(String),

    /// Supertrait not satisfied
    #[error("Supertrait {0} not satisfied for type {1}")]
    SupertraitNotSatisfied(String, String),
}

/// Trait resolution result
pub type TraitResult<T> = Result<T, TraitError>;

/// Trait registry
#[derive(Debug, Clone)]
pub struct TraitRegistry {
    /// All trait definitions
    traits: HashMap<TraitId, Trait>,
    /// All implementations
    impls: HashMap<ImplId, TraitImpl>,
    /// Index: "trait_id:type_string" -> impl_ids
    impl_index: HashMap<String, Vec<ImplId>>,
    /// Next trait ID
    next_trait_id: TraitId,
    /// Next impl ID
    next_impl_id: ImplId,
    /// Current crate name
    current_crate: String,
}

impl TraitRegistry {
    /// Create a new trait registry
    pub fn new(current_crate: String) -> Self {
        Self {
            traits: HashMap::new(),
            impls: HashMap::new(),
            impl_index: HashMap::new(),
            next_trait_id: 0,
            next_impl_id: 0,
            current_crate,
        }
    }

    /// Create index key from trait ID and type
    fn make_key(trait_id: TraitId, ty: &Type) -> String {
        format!("{}:{}", trait_id, ty.to_string())
    }

    /// Register a new trait
    pub fn register_trait(&mut self, mut trait_def: Trait) -> TraitId {
        let id = self.next_trait_id;
        self.next_trait_id += 1;
        trait_def.id = id;
        self.traits.insert(id, trait_def);
        id
    }

    /// Register a trait implementation
    pub fn register_impl(&mut self, mut impl_def: TraitImpl) -> TraitResult<ImplId> {
        // Check orphan rule
        self.check_orphan_rule(&impl_def)?;

        // Check for overlapping implementations
        self.check_coherence(&impl_def)?;

        // Assign ID
        let id = self.next_impl_id;
        self.next_impl_id += 1;
        impl_def.id = id;

        // Index the implementation
        let key = Self::make_key(impl_def.trait_id, &impl_def.self_type);
        self.impl_index.entry(key).or_default().push(id);

        self.impls.insert(id, impl_def);
        Ok(id)
    }

    /// Find implementation of a trait for a type
    pub fn find_impl(&self, trait_id: TraitId, ty: &Type) -> TraitResult<&TraitImpl> {
        let key = Self::make_key(trait_id, ty);

        if let Some(impl_ids) = self.impl_index.get(&key) {
            match impl_ids.len() {
                0 => {
                    let trait_name = self.get_trait_name(trait_id);
                    Err(TraitError::NoImpl(trait_name, ty.to_string()))
                }
                1 => Ok(self.impls.get(&impl_ids[0]).unwrap()),
                _ => {
                    let trait_name = self.get_trait_name(trait_id);
                    Err(TraitError::Ambiguous(trait_name, ty.to_string()))
                }
            }
        } else {
            let trait_name = self.get_trait_name(trait_id);
            Err(TraitError::NoImpl(trait_name, ty.to_string()))
        }
    }

    /// Check if a type implements a trait
    pub fn has_impl(&self, trait_id: TraitId, ty: &Type) -> bool {
        self.find_impl(trait_id, ty).is_ok()
    }

    /// Check orphan rule: either trait or type must be local
    fn check_orphan_rule(&self, impl_def: &TraitImpl) -> TraitResult<()> {
        // If implementing in the current crate, we need to own either the trait or the type
        if impl_def.defining_crate != self.current_crate {
            return Ok(()); // Foreign impl, assume it's checked elsewhere
        }

        let trait_local = self
            .traits
            .get(&impl_def.trait_id)
            .map(|t| t.name.starts_with(&self.current_crate))
            .unwrap_or(false);

        let type_local = self.is_type_local(&impl_def.self_type);

        if !trait_local && !type_local {
            let trait_name = self.get_trait_name(impl_def.trait_id);
            return Err(TraitError::OrphanRule(
                trait_name,
                impl_def.self_type.to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a type is defined in the current crate
    fn is_type_local(&self, ty: &Type) -> bool {
        match ty {
            Type::Named { name, .. } => name.starts_with(&self.current_crate),
            _ => false, // Primitives and built-in types are not local
        }
    }

    /// Check coherence (no overlapping implementations)
    fn check_coherence(&self, impl_def: &TraitImpl) -> TraitResult<()> {
        let key = Self::make_key(impl_def.trait_id, &impl_def.self_type);

        if let Some(existing) = self.impl_index.get(&key) {
            if !existing.is_empty() {
                let trait_name = self.get_trait_name(impl_def.trait_id);
                return Err(TraitError::Coherence(trait_name));
            }
        }

        Ok(())
    }

    /// Get trait name
    fn get_trait_name(&self, trait_id: TraitId) -> String {
        self.traits
            .get(&trait_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| format!("Trait#{}", trait_id))
    }

    /// Get trait definition
    pub fn get_trait(&self, trait_id: TraitId) -> Option<&Trait> {
        self.traits.get(&trait_id)
    }

    /// Check if all supertraits are satisfied
    pub fn check_supertraits(&self, trait_id: TraitId, ty: &Type) -> TraitResult<()> {
        if let Some(trait_def) = self.traits.get(&trait_id) {
            for &supertrait_id in &trait_def.supertraits {
                if !self.has_impl(supertrait_id, ty) {
                    let supertrait_name = self.get_trait_name(supertrait_id);
                    return Err(TraitError::SupertraitNotSatisfied(
                        supertrait_name,
                        ty.to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ty::PrimitiveType;

    #[test]
    fn test_register_trait() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };

        let id = registry.register_trait(trait_def);
        assert_eq!(id, 0);
        assert!(registry.get_trait(id).is_some());
    }

    #[test]
    fn test_register_impl() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "my_crate::Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        let impl_def = TraitImpl {
            id: 0,
            trait_id,
            self_type: Type::Named {
                name: "my_crate::MyType".to_string(),
                args: vec![],
            },
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };

        let result = registry.register_impl(impl_def);
        assert!(result.is_ok());
    }

    #[test]
    fn test_coherence_check() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "my_crate::Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        let ty = Type::Named {
            name: "my_crate::MyType".to_string(),
            args: vec![],
        };

        let impl1 = TraitImpl {
            id: 0,
            trait_id,
            self_type: ty.clone(),
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };

        let impl2 = TraitImpl {
            id: 0,
            trait_id,
            self_type: ty.clone(),
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };

        // First impl should succeed
        assert!(registry.register_impl(impl1).is_ok());

        // Second impl should fail (coherence violation)
        assert!(matches!(
            registry.register_impl(impl2),
            Err(TraitError::Coherence(_))
        ));
    }

    #[test]
    fn test_orphan_rule() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        // Foreign trait
        let trait_def = Trait {
            id: 0,
            name: "std::Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        // Foreign type
        let foreign_type = Type::Primitive(PrimitiveType::I32);

        let impl_def = TraitImpl {
            id: 0,
            trait_id,
            self_type: foreign_type,
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };

        // Should fail orphan rule (neither trait nor type is local)
        assert!(matches!(
            registry.register_impl(impl_def),
            Err(TraitError::OrphanRule(_, _))
        ));
    }

    #[test]
    fn test_find_impl() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        let trait_def = Trait {
            id: 0,
            name: "my_crate::Display".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let trait_id = registry.register_trait(trait_def);

        let ty = Type::Named {
            name: "my_crate::MyType".to_string(),
            args: vec![],
        };

        let impl_def = TraitImpl {
            id: 0,
            trait_id,
            self_type: ty.clone(),
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };

        registry.register_impl(impl_def).unwrap();

        // Should find the impl
        assert!(registry.find_impl(trait_id, &ty).is_ok());
        assert!(registry.has_impl(trait_id, &ty));

        // Different type should not find impl
        let other_ty = Type::Primitive(PrimitiveType::I32);
        assert!(registry.find_impl(trait_id, &other_ty).is_err());
    }

    #[test]
    fn test_supertraits() {
        let mut registry = TraitRegistry::new("my_crate".to_string());

        // Base trait
        let base_trait = Trait {
            id: 0,
            name: "my_crate::Base".to_string(),
            type_params: vec![],
            supertraits: vec![],
            assoc_types: vec![],
            methods: vec![],
        };
        let base_id = registry.register_trait(base_trait);

        // Derived trait with supertrait
        let derived_trait = Trait {
            id: 0,
            name: "my_crate::Derived".to_string(),
            type_params: vec![],
            supertraits: vec![base_id],
            assoc_types: vec![],
            methods: vec![],
        };
        let derived_id = registry.register_trait(derived_trait);

        let ty = Type::Named {
            name: "my_crate::MyType".to_string(),
            args: vec![],
        };

        // Implement base trait
        let base_impl = TraitImpl {
            id: 0,
            trait_id: base_id,
            self_type: ty.clone(),
            type_args: vec![],
            assoc_type_defs: HashMap::new(),
            defining_crate: "my_crate".to_string(),
        };
        registry.register_impl(base_impl).unwrap();

        // Check supertraits
        assert!(registry.check_supertraits(derived_id, &ty).is_ok());
    }
}
