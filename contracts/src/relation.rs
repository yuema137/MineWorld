//! Relations: typed edges between any two entities.
//!
//! A relation is a first-class edge, not a field on a person (`docs/CORE_CONCEPTS.md` §9):
//! `Person ↔ Person`, `Person ↔ Organization`, `Organization ↔ Place`, `Person ↔ Item`,
//! `Place ↔ Place` are all the same machinery with different declarations.
//!
//! # What a relation is, and what it is not
//!
//! This module provides the *edge* and the rules for forming one. It provides no relationship
//! **values**: there is no friendship strength, no trust, no employment here, and there never
//! will be — those are components owned by the system that declares them. A relation is
//! identified by the triple `(relation type, from, to)`, so per-relation state is a component
//! keyed by that same triple and owned by the declaring system, which keeps the single-writer
//! rule (`INV-7`) working unchanged for state that hangs off an edge.
//!
//! # One edge per type per ordered pair
//!
//! The triple *is* the identity. There is no relation id, and a second edge of the same type
//! between the same pair is not representable — asking for one twice is asking for the same
//! edge. A relation type that needs to distinguish two connections between the same entities
//! (two roads between two towns, say) is modelling something else: the roads are entities, each
//! with its own edges.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::entity::Entity;
use crate::error::ContractError;
use crate::ids::{EntityId, EntityType, RelationTypeId, SystemId};

/// A non-empty set of entity types, as a relation type's declaration permits at an endpoint.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "BTreeSet<EntityType>", into = "BTreeSet<EntityType>")]
pub struct EntityTypeSet(BTreeSet<EntityType>);

impl EntityTypeSet {
    /// Collects the types, rejecting an empty set: a relation type that permits nothing at an
    /// endpoint cannot be satisfied, and a declaration a world can never use is a mistake worth
    /// catching where it is written.
    pub fn new(types: impl IntoIterator<Item = EntityType>) -> Result<Self, ContractError> {
        let types: BTreeSet<EntityType> = types.into_iter().collect();
        if types.is_empty() {
            return Err(ContractError::EmptyEntityTypeSet);
        }
        Ok(Self(types))
    }

    /// Whether the set permits this type.
    pub fn contains(&self, entity_type: EntityType) -> bool {
        self.0.contains(&entity_type)
    }

    /// The types, in their fixed order.
    pub fn iter(&self) -> impl Iterator<Item = EntityType> + '_ {
        self.0.iter().copied()
    }
}

impl TryFrom<BTreeSet<EntityType>> for EntityTypeSet {
    type Error = ContractError;

    fn try_from(value: BTreeSet<EntityType>) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<EntityTypeSet> for BTreeSet<EntityType> {
    fn from(value: EntityTypeSet) -> Self {
        value.0
    }
}

impl core::fmt::Display for EntityTypeSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for entity_type in self.iter() {
            if !first {
                f.write_str(", ")?;
            }
            write!(f, "{entity_type}")?;
            first = false;
        }
        Ok(())
    }
}

/// Whether the two ends of a relation type mean different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationDirection {
    /// `from` and `to` are different roles: `employee_of` does not mean the same thing read
    /// backwards, so `(a, b)` and `(b, a)` are two different edges.
    Directed,
    /// The ends are interchangeable: `connected_to` is the same fact read either way, so
    /// `(a, b)` and `(b, a)` are one edge and are stored identically.
    Undirected,
}

/// Whether an entity may relate to itself under a relation type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum SelfEdges {
    /// An entity cannot be both ends of this edge. The default: for most relation types a
    /// self-edge is a bug in whoever built it.
    #[default]
    Forbidden,
    /// An entity may be both ends — a place connected to itself, a person who is their own
    /// emergency contact.
    Permitted,
}

/// What a system declares about a kind of edge before any edge of that kind exists.
///
/// The declaration is owned by a system, exactly as a component type is: the declaring system is
/// the single writer of any state attached to edges of this type. A relation itself is formed by
/// checking against this declaration — [`Relation::between`] — so the rules live in one value
/// rather than in each call site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationTypeDeclaration {
    relation_type: RelationTypeId,
    owner: SystemId,
    direction: RelationDirection,
    from_types: EntityTypeSet,
    to_types: EntityTypeSet,
    self_edges: SelfEdges,
}

impl RelationTypeDeclaration {
    /// Declares an edge whose two ends mean different things, such as `employee_of` from a
    /// person to an organization.
    pub fn directed(
        relation_type: RelationTypeId,
        owner: SystemId,
        from_types: EntityTypeSet,
        to_types: EntityTypeSet,
    ) -> Self {
        Self {
            relation_type,
            owner,
            direction: RelationDirection::Directed,
            from_types,
            to_types,
            self_edges: SelfEdges::Forbidden,
        }
    }

    /// Declares an edge whose ends are interchangeable, such as `connected_to` between places.
    ///
    /// One endpoint set, not two: the ends of an undirected edge are canonically ordered when
    /// the edge is formed, so two sets could otherwise reject the same edge depending on which
    /// way the caller happened to write it.
    pub fn undirected(
        relation_type: RelationTypeId,
        owner: SystemId,
        endpoint_types: EntityTypeSet,
    ) -> Self {
        Self {
            relation_type,
            owner,
            direction: RelationDirection::Undirected,
            from_types: endpoint_types.clone(),
            to_types: endpoint_types,
            self_edges: SelfEdges::Forbidden,
        }
    }

    /// Permits an entity to be both ends of this edge. A declaration says so explicitly or a
    /// self-edge is refused; nothing decides it per call site.
    #[must_use]
    pub fn permitting_self_edges(mut self) -> Self {
        self.self_edges = SelfEdges::Permitted;
        self
    }

    /// The name of the declared edge.
    pub const fn relation_type(&self) -> &RelationTypeId {
        &self.relation_type
    }

    /// The system that declared it, and therefore owns any state attached to its edges.
    pub const fn owner(&self) -> &SystemId {
        &self.owner
    }

    /// Whether the ends mean different things.
    pub const fn direction(&self) -> RelationDirection {
        self.direction
    }

    /// The types permitted at the `from` end — for an undirected type, at either end.
    pub const fn from_types(&self) -> &EntityTypeSet {
        &self.from_types
    }

    /// The types permitted at the `to` end — for an undirected type, at either end.
    pub const fn to_types(&self) -> &EntityTypeSet {
        &self.to_types
    }

    /// Whether an entity may relate to itself here.
    pub const fn self_edges(&self) -> SelfEdges {
        self.self_edges
    }
}

/// Which end of an edge a problem is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationEnd {
    /// The end the edge leads from.
    From,
    /// The end the edge leads to.
    To,
}

impl core::fmt::Display for RelationEnd {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::From => "from",
            Self::To => "to",
        })
    }
}

/// One edge: a relation type and its two endpoints, and nothing else.
///
/// The triple is the whole identity (`DD-6`), which is why this type is also what per-relation
/// state is keyed by. It is formed only through [`Relation::between`], so an edge that exists has
/// been checked against its declaration, and an undirected edge is already canonical: the fields
/// are private precisely so that ordering cannot be bypassed by building the value directly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Relation {
    relation_type: RelationTypeId,
    from: EntityId,
    to: EntityId,
}

impl Relation {
    /// Forms the edge `from` → `to` if `declaration` permits it.
    ///
    /// The endpoints are whole entities rather than identities, because the check is about their
    /// types and reading the type off the entity is the only way it cannot be misstated by the
    /// caller. For an undirected type the endpoints are stored in a canonical order, so the same
    /// edge written either way is one value that serializes identically.
    pub fn between(
        declaration: &RelationTypeDeclaration,
        from: &Entity,
        to: &Entity,
    ) -> Result<Self, ContractError> {
        if from.id() == to.id() && declaration.self_edges() == SelfEdges::Forbidden {
            return Err(ContractError::SelfEdgeNotPermitted {
                relation_type: declaration.relation_type().clone(),
                entity: from.id(),
            });
        }

        Self::check_end(declaration, RelationEnd::From, from)?;
        Self::check_end(declaration, RelationEnd::To, to)?;

        let (from, to) = match declaration.direction() {
            RelationDirection::Directed => (from.id(), to.id()),
            RelationDirection::Undirected if to.id() < from.id() => (to.id(), from.id()),
            RelationDirection::Undirected => (from.id(), to.id()),
        };

        Ok(Self {
            relation_type: declaration.relation_type().clone(),
            from,
            to,
        })
    }

    fn check_end(
        declaration: &RelationTypeDeclaration,
        end: RelationEnd,
        entity: &Entity,
    ) -> Result<(), ContractError> {
        let permitted = match end {
            RelationEnd::From => declaration.from_types(),
            RelationEnd::To => declaration.to_types(),
        };
        if permitted.contains(entity.entity_type()) {
            return Ok(());
        }
        Err(ContractError::RelationEndpointNotPermitted {
            relation_type: declaration.relation_type().clone(),
            end,
            entity: entity.id(),
            actual: entity.entity_type(),
            permitted: permitted.clone(),
        })
    }

    /// The kind of edge this is.
    pub const fn relation_type(&self) -> &RelationTypeId {
        &self.relation_type
    }

    /// The end the edge leads from — for an undirected edge, the lower of the two identities.
    pub const fn from(&self) -> EntityId {
        self.from
    }

    /// The end the edge leads to — for an undirected edge, the higher of the two identities.
    pub const fn to(&self) -> EntityId {
        self.to
    }
}
