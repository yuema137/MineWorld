//! Selectors: which entities an entry applies to, one per role (`ARC-63` items 1 and 5, `ARC-64`).

use mineworld_authoring::{ClassName, Classed, EntityClasses};
use mineworld_contracts::{EntityId, EntityType, Tags};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::decl::Role;

/// One role's selector: `*`, or a class (declared or implicit).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selector {
    /// Any entity, or none.
    Any,
    /// The entities of this class, or of this implicit class's type.
    Class(ClassName),
}

impl Selector {
    /// The selector as a section writes it.
    pub fn parse(text: &str) -> Result<Self, String> {
        if text == "*" {
            return Ok(Self::Any);
        }
        ClassName::new(text)
            .map(Self::Class)
            .map_err(|_| format!("'{text}' is neither '*' nor a class name"))
    }
}

impl core::fmt::Display for Selector {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Any => f.write_str("*"),
            Self::Class(class) => f.write_str(class.as_str()),
        }
    }
}

impl Serialize for Selector {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

/// An entry's selectors, one per role, in [`Role::ALL`]'s order; a role the entry does not name is `*`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Selectors([Selector; 4]);

impl Default for Selectors {
    fn default() -> Self {
        Self([Selector::Any, Selector::Any, Selector::Any, Selector::Any])
    }
}

impl Selectors {
    /// Every role `*`.
    pub fn any() -> Self {
        Self::default()
    }

    /// These selectors with `role` set to `selector`.
    #[must_use]
    pub fn with(mut self, role: Role, selector: Selector) -> Self {
        self.0[role.index()] = selector;
        self
    }

    /// The selector for `role`.
    pub fn of(&self, role: Role) -> &Selector {
        &self.0[role.index()]
    }

    /// How many roles are named (not `*`): the specificity precedence compares.
    pub fn specificity(&self) -> usize {
        self.0
            .iter()
            .filter(|selector| **selector != Selector::Any)
            .count()
    }

    /// The classes named, for the classes a resolved section copies and for `ClassUndefined`.
    pub fn classes(&self) -> impl Iterator<Item = &ClassName> {
        self.0.iter().filter_map(|selector| match selector {
            Selector::Any => None,
            Selector::Class(class) => Some(class),
        })
    }

    /// Whether some entity tuple could match both (`ARC-63` item 5): for every role the selectors are
    /// equal, or one is `*`, or one is an implicit class whose type the other's class selects.
    pub fn overlaps(&self, other: &Self, classes: &EntityClasses) -> bool {
        self.0.iter().zip(&other.0).all(|pair| match pair {
            (Selector::Any, _) | (_, Selector::Any) => true,
            (Selector::Class(a), Selector::Class(b)) => {
                a == b
                    || a.implicit_type()
                        .is_some_and(|t| classes.type_of(b) == Some(t))
                    || b.implicit_type()
                        .is_some_and(|t| classes.type_of(a) == Some(t))
            }
        })
    }

    /// Whether these selectors apply to the entities filling each role.
    pub fn matches(&self, classes: &EntityClasses, subjects: &RoleSubjects<'_>) -> bool {
        self.0
            .iter()
            .zip(&subjects.0)
            .all(|(selector, subject)| match (selector, subject) {
                (Selector::Any, _) => true,
                (Selector::Class(_), None) => false,
                (Selector::Class(class), Some(subject)) => classes.matches(class, *subject),
            })
    }
}

/// The entities filling each role of one request or fact, as the class rule sees them: a type and
/// tags. A role nobody fills is matched only by `*`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RoleSubjects<'a>([Option<Classed<'a>>; 4]);

impl<'a> RoleSubjects<'a> {
    /// Nobody in any role.
    pub const fn none() -> Self {
        Self([None, None, None, None])
    }

    /// These subjects with `role` filled by an entity of `entity_type` carrying `tags`.
    #[must_use]
    pub const fn with(mut self, role: Role, entity_type: EntityType, tags: &'a Tags) -> Self {
        self.0[role.index()] = Some(Classed { entity_type, tags });
        self
    }

    /// Whether `role` is filled.
    pub const fn fills(&self, role: Role) -> bool {
        self.0[role.index()].is_some()
    }
}

/// Which entity fills each role of a request or a fact, by id: what a pack hands a lookup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Roles([Option<EntityId>; 4]);

impl Roles {
    /// Nobody in any role.
    pub const fn new() -> Self {
        Self([None, None, None, None])
    }

    /// These roles with `role` filled by `entity`.
    #[must_use]
    pub const fn with(mut self, role: Role, entity: EntityId) -> Self {
        self.0[role.index()] = Some(entity);
        self
    }

    /// The entity filling `role`.
    pub const fn get(&self, role: Role) -> Option<EntityId> {
        self.0[role.index()]
    }
}
