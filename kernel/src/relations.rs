//! The relation graph: typed edges between any two entities, written only by the system that
//! declared the edge type.
//!
//! An edge is the triple `(relation type, from, to)` and nothing else (`DD-6`). There is no edge
//! identity and no per-edge payload here: state that hangs off a relationship is a component
//! owned by the declaring system and keyed by the same triple, which is how `INV-7` keeps working
//! for it unchanged.
//!
//! # The store forms the edge, so the declaration cannot be bypassed
//!
//! [`RelationStore::insert`] takes the two entities rather than a ready-made [`Relation`], and
//! builds the edge itself with [`Relation::between`] against the declaration this world holds. A
//! caller therefore cannot supply an edge built against a declaration it invented: the endpoint
//! types, the self-edge rule and the canonical ordering of an undirected edge are all checked
//! against the authoritative declaration, every time.
//!
//! Nothing here looks at where an entity *is*. The only fact an edge checks about an endpoint is
//! its [`EntityType`](mineworld_contracts::EntityType), so a person may be connected to an
//! organization that has no location at all, and two places may be connected without sharing any
//! spatial frame. A relation is not proximity.
//!
//! # Ownership of an edge type is checked at run time, and why
//!
//! Component writes are gated by the compiler, because a component type is a Rust type and its
//! owner is a constant on it. A relation type is not a Rust type: it is a
//! [`RelationTypeId`] value in a declaration, and unlike [`SystemId`](mineworld_contracts::SystemId)
//! and [`ComponentTypeId`](mineworld_contracts::ComponentTypeId) it has no
//! `from_static`, so it cannot appear in an associated constant and there is no type-level owner
//! to check. The gate is therefore the declaration the store holds: a write compares the declared
//! owner with the writing system and refuses by name, changing nothing. The token is still
//! required, so the writer still has to *be* a system; what the compiler cannot do here is say
//! *which* system, and pretending otherwise would be worse than saying so.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{
    Entity, EntityId, LifecycleState, Relation, RelationDirection, RelationTypeDeclaration,
    RelationTypeId,
};
use serde::{Deserialize, Serialize};

use crate::access::{SystemIdentity, WriteToken};
use crate::error::KernelError;

/// Every declared edge type in one world, and every edge of those types.
///
/// Iteration is in `(relation type, from, to)` order, because that is the order a
/// [`BTreeSet`] of [`Relation`] keeps them in — never the order they were created (`KD-5`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RelationStoreSnapshot")]
pub struct RelationStore {
    declarations: BTreeMap<RelationTypeId, RelationTypeDeclaration>,
    edges: BTreeSet<Relation>,
}

impl Default for RelationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationStore {
    /// A world with no edge types declared.
    pub fn new() -> Self {
        Self {
            declarations: BTreeMap::new(),
            edges: BTreeSet::new(),
        }
    }

    /// Declares an edge type, which is what has to happen before any edge of it can exist.
    ///
    /// The declaring system must be the one the declaration names as owner — a system cannot
    /// install somebody else's edge type — and two systems cannot claim one edge type name, for
    /// the same reason they cannot claim one component type. Re-declaring the identical
    /// declaration is not an error.
    pub fn declare<S: SystemIdentity>(
        &mut self,
        token: &WriteToken<S>,
        declaration: RelationTypeDeclaration,
    ) -> Result<(), KernelError> {
        if *declaration.owner() != token.system() {
            return Err(KernelError::RelationTypeNotOwned {
                relation_type: declaration.relation_type().clone(),
                owner: declaration.owner().clone(),
                writing_system: token.system(),
            });
        }

        match self.declarations.get(declaration.relation_type()) {
            None => {
                self.declarations
                    .insert(declaration.relation_type().clone(), declaration);
                Ok(())
            }
            Some(existing) if existing.owner() != declaration.owner() => {
                Err(KernelError::RelationTypeClaimedByAnotherSystem {
                    relation_type: declaration.relation_type().clone(),
                    declared_by: existing.owner().clone(),
                    claimed_by: declaration.owner().clone(),
                })
            }
            Some(existing) if *existing != declaration => {
                Err(KernelError::RelationTypeAlreadyDeclared {
                    relation_type: declaration.relation_type().clone(),
                })
            }
            Some(_) => Ok(()),
        }
    }

    /// What this world says about an edge type, if any system declared one.
    pub fn declaration(&self, relation_type: &RelationTypeId) -> Option<&RelationTypeDeclaration> {
        self.declarations.get(relation_type)
    }

    /// Every declared edge type, in name order.
    pub fn declarations(&self) -> impl Iterator<Item = &RelationTypeDeclaration> {
        self.declarations.values()
    }

    /// Forms the edge `from` → `to` and stores it, returning the edge as the store holds it.
    ///
    /// The edge is built here, against this world's declaration, so the endpoint types, the
    /// self-edge rule and — for an undirected type — the canonical ordering of the two ends are
    /// checked against the declaration rather than against whatever the caller had in mind. An
    /// undirected edge given the other way round is therefore the same edge, not a second one.
    ///
    /// Storing an edge that is already there changes nothing and is not an error: the triple is
    /// the identity, so asking twice is asking for the same edge (`DD-6`).
    pub fn insert<S: SystemIdentity>(
        &mut self,
        token: &WriteToken<S>,
        relation_type: &RelationTypeId,
        from: &Entity,
        to: &Entity,
    ) -> Result<Relation, KernelError> {
        let declaration = self.owned_declaration(relation_type, token)?;
        let relation = Relation::between(declaration, from, to)?;
        self.edges.insert(relation.clone());
        Ok(relation)
    }

    /// Removes an edge, reporting whether it was there.
    ///
    /// Only the declaring system may remove one. The edge is passed as a value because every
    /// [`Relation`] that exists was formed through [`Relation::between`] and is therefore already
    /// canonical: there is no way to hold a non-canonical edge to pass in.
    pub fn remove<S: SystemIdentity>(
        &mut self,
        token: &WriteToken<S>,
        relation: &Relation,
    ) -> Result<bool, KernelError> {
        self.owned_declaration(relation.relation_type(), token)?;
        Ok(self.edges.remove(relation))
    }

    /// Whether this exact edge exists.
    pub fn contains(&self, relation: &Relation) -> bool {
        self.edges.contains(relation)
    }

    /// Every edge, in `(relation type, from, to)` order.
    pub fn iter(&self) -> impl Iterator<Item = &Relation> {
        self.edges.iter()
    }

    /// Every edge of one type, in endpoint order.
    pub fn of_type<'a>(
        &'a self,
        relation_type: &'a RelationTypeId,
    ) -> impl Iterator<Item = &'a Relation> {
        self.edges
            .iter()
            .filter(move |relation| relation.relation_type() == relation_type)
    }

    /// Every edge with `entity` at either end.
    ///
    /// Either end, always: an edge is a fact about both of its endpoints, so a directed
    /// `employee_of` is found from the organization as readily as from the person. A reader that
    /// only wants one direction compares [`Relation::from`] or [`Relation::to`] itself.
    ///
    /// This is a scan of the edge set rather than an index lookup. Deliberate: MineWorld's worlds
    /// hold hundreds of entities, `DEP-1` accepted exactly this trade, and an index is two copies
    /// of one fact — addable behind this method on the day a measurement asks for it.
    pub fn touching(&self, entity: EntityId) -> impl Iterator<Item = &Relation> {
        self.edges
            .iter()
            .filter(move |relation| relation.from() == entity || relation.to() == entity)
    }

    /// How many edges this world holds.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether this world holds no edges.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Removes every edge touching a destroyed entity, returning them in edge order.
    ///
    /// This is the one write here that no system's token authorizes, and it is deliberately
    /// narrow: it requires the entity record and refuses unless its lifecycle is
    /// [`Destroyed`](LifecycleState::Destroyed). An edge to a destroyed entity is a reference to
    /// something that has stopped participating in the world, and since `Destroyed` is terminal it
    /// can never start again — so no declaring system could ever want the edge kept. Every other
    /// removal goes through [`RelationStore::remove`] and its owner.
    ///
    /// The removed edges are returned rather than dropped, because their declaring systems have to
    /// be told what happened to state they own.
    pub fn remove_edges_of_destroyed_entity(
        &mut self,
        entity: &Entity,
    ) -> Result<Vec<Relation>, KernelError> {
        if entity.lifecycle() != LifecycleState::Destroyed {
            return Err(KernelError::EntityNotDestroyed {
                entity: entity.id(),
                lifecycle: entity.lifecycle(),
            });
        }

        let removed: Vec<Relation> = self.touching(entity.id()).cloned().collect();
        for relation in &removed {
            self.edges.remove(relation);
        }
        Ok(removed)
    }

    /// The declaration for `relation_type`, if this world has one and `token`'s system owns it.
    fn owned_declaration<S: SystemIdentity>(
        &self,
        relation_type: &RelationTypeId,
        token: &WriteToken<S>,
    ) -> Result<&RelationTypeDeclaration, KernelError> {
        let declaration = self.declarations.get(relation_type).ok_or_else(|| {
            KernelError::RelationTypeNotDeclared {
                relation_type: relation_type.clone(),
            }
        })?;
        if *declaration.owner() != token.system() {
            return Err(KernelError::RelationTypeNotOwned {
                relation_type: relation_type.clone(),
                owner: declaration.owner().clone(),
                writing_system: token.system(),
            });
        }
        Ok(declaration)
    }
}

/// A relation graph as it is written down: the declarations and the edges.
///
/// Public for the same reason [`EntityRegistrySnapshot`](crate::EntityRegistrySnapshot) is: it is
/// what persistence reads and writes, and the refusals below are part of the contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationStoreSnapshot {
    /// Every declared edge type, filed under its own name.
    pub declarations: BTreeMap<RelationTypeId, RelationTypeDeclaration>,
    /// Every edge.
    pub edges: BTreeSet<Relation>,
}

impl From<&RelationStore> for RelationStoreSnapshot {
    fn from(store: &RelationStore) -> Self {
        Self {
            declarations: store.declarations.clone(),
            edges: store.edges.clone(),
        }
    }
}

impl TryFrom<RelationStoreSnapshot> for RelationStore {
    type Error = KernelError;

    /// Rebuilds a graph from its persisted form, refusing one that has lost a property the store
    /// itself would never have let go.
    ///
    /// [`Relation`] deserializes field by field, so a file can hold an edge no
    /// [`Relation::between`] would ever have produced. Three things are therefore checked: a
    /// declaration filed under a name other than its own, an edge of a type nobody declared, and
    /// an undirected edge whose ends are the wrong way round — which would let the same fact be
    /// stored twice and read as two edges.
    fn try_from(snapshot: RelationStoreSnapshot) -> Result<Self, Self::Error> {
        for (name, declaration) in &snapshot.declarations {
            if declaration.relation_type() != name {
                return Err(KernelError::PersistedRelationTypeMismatch {
                    at: name.clone(),
                    found: declaration.relation_type().clone(),
                });
            }
        }

        for edge in &snapshot.edges {
            let declaration = snapshot
                .declarations
                .get(edge.relation_type())
                .ok_or_else(|| KernelError::RelationTypeNotDeclared {
                    relation_type: edge.relation_type().clone(),
                })?;
            if declaration.direction() == RelationDirection::Undirected && edge.to() < edge.from() {
                return Err(KernelError::PersistedRelationNotCanonical {
                    relation_type: edge.relation_type().clone(),
                    from: edge.from(),
                    to: edge.to(),
                });
            }
        }

        Ok(Self {
            declarations: snapshot.declarations,
            edges: snapshot.edges,
        })
    }
}
