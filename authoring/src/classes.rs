//! A world's entity classes: named tag selectors over one entity type (`docs/DECISIONS.md` `ARC-64`).
//!
//! `configure/classes.yaml`, a framework key of `configure:` (`ARC-61` note), is a list of
//! `{ class, of, tag }` in priority order. An entity's class is the first entry whose `of` is its type
//! and whose tag it carries; otherwise its type's implicit class, named by the type. Every entity also
//! matches the selector of its type's implicit class. Tags are fixed after genesis, so a class never
//! changes while a world runs, and a class is not state: nothing here is seeded on its own.

use std::collections::BTreeSet;

use mineworld_contracts::{EntityType, SystemId, Tag, Tags};
use serde::{Deserialize, Serialize};

/// The name of an entity class: the system-id grammar (lowercase ASCII letters, digits, `-` and `_`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClassName(SystemId);

impl ClassName {
    /// A class name in code, checked while the declaring crate compiles.
    pub const fn from_static(value: &'static str) -> Self {
        Self(SystemId::from_static(value))
    }

    /// Validates a name.
    ///
    /// # Errors
    ///
    /// The identifier rule's refusal.
    pub fn new(value: impl Into<String>) -> Result<Self, mineworld_contracts::ContractError> {
        SystemId::new(value).map(Self)
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// The implicit class every entity of `entity_type` matches.
    pub const fn implicit(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Person => Self::from_static("person"),
            EntityType::Place => Self::from_static("place"),
            EntityType::Item => Self::from_static("item"),
            EntityType::Organization => Self::from_static("organization"),
        }
    }

    /// The type this name is the implicit class of, if it is one of the four reserved names.
    pub fn implicit_type(&self) -> Option<EntityType> {
        IMPLICIT
            .into_iter()
            .find(|entity_type| Self::implicit(*entity_type) == *self)
    }
}

impl core::fmt::Display for ClassName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The four entity types, each with its implicit class.
const IMPLICIT: [EntityType; 4] = [
    EntityType::Person,
    EntityType::Place,
    EntityType::Item,
    EntityType::Organization,
];

/// One entry of `configure/classes.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassDefinition {
    /// The class.
    pub class: ClassName,
    /// The one entity type it selects among.
    pub of: EntityType,
    /// The tag an entity of that type must carry.
    pub tag: Tag,
}

/// What an entity is, for class membership: its type and its tags.
#[derive(Debug, Clone, Copy)]
pub struct Classed<'a> {
    /// Its entity type.
    pub entity_type: EntityType,
    /// Its tags.
    pub tags: &'a Tags,
}

/// A world's entity classes, in priority order (`ARC-64`). Empty when the world lists no `classes`:
/// then every entity is in its implicit class only.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(into = "Vec<ClassDefinition>")]
pub struct EntityClasses(Vec<ClassDefinition>);

/// The refusal for `definition`, after the classes defined before it, if any.
fn refusal(seen: &BTreeSet<ClassName>, definition: &ClassDefinition) -> Option<String> {
    if let Some(entity_type) = definition.class.implicit_type() {
        return Some(format!(
            "'{}' is the implicit class of every {entity_type}; a class cannot take its name",
            definition.class
        ));
    }
    seen.contains(&definition.class).then(|| {
        format!(
            "the class '{}' is defined twice; a class is one tag over one type",
            definition.class
        )
    })
}

impl TryFrom<Vec<ClassDefinition>> for EntityClasses {
    type Error = String;

    fn try_from(definitions: Vec<ClassDefinition>) -> Result<Self, Self::Error> {
        let mut seen = BTreeSet::new();
        for definition in &definitions {
            if let Some(refused) = refusal(&seen, definition) {
                return Err(refused);
            }
            seen.insert(definition.class.clone());
        }
        Ok(Self(definitions))
    }
}

/// Decoded entry by entry, so a refusal is raised while the decoder still stands at the offending
/// entry and keeps its line and column.
impl<'de> Deserialize<'de> for EntityClasses {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = EntityClasses;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a list of { class, of, tag }")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut seen = BTreeSet::new();
                let mut definitions = Vec::new();
                while let Some(definition) = seq.next_element_seed(Checked(&seen))? {
                    seen.insert(definition.class.clone());
                    definitions.push(definition);
                }
                Ok(EntityClasses(definitions))
            }
        }

        deserializer.deserialize_seq(Visitor)
    }
}

/// One definition, refused inside its own decoding — so the decoder reports the entry's position —
/// when the classes before it make it invalid.
struct Checked<'s>(&'s BTreeSet<ClassName>);

impl<'de> serde::de::DeserializeSeed<'de> for Checked<'_> {
    type Value = ClassDefinition;

    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        let definition = ClassDefinition::deserialize(d)?;
        match refusal(self.0, &definition) {
            Some(refused) => Err(serde::de::Error::custom(refused)),
            None => Ok(definition),
        }
    }
}

impl From<EntityClasses> for Vec<ClassDefinition> {
    fn from(classes: EntityClasses) -> Self {
        classes.0
    }
}

impl EntityClasses {
    /// The definitions, in priority order.
    pub fn definitions(&self) -> &[ClassDefinition] {
        &self.0
    }

    /// The class an entity is in: the first definition of its type whose tag it carries, otherwise its
    /// implicit class.
    pub fn class_of(&self, entity: Classed<'_>) -> ClassName {
        self.0
            .iter()
            .find(|definition| {
                definition.of == entity.entity_type && entity.tags.contains(&definition.tag)
            })
            .map_or_else(
                || ClassName::implicit(entity.entity_type),
                |definition| definition.class.clone(),
            )
    }

    /// Whether a selector naming `class` matches the entity: it is the entity's class, or the implicit
    /// class of the entity's type.
    pub fn matches(&self, class: &ClassName, entity: Classed<'_>) -> bool {
        class.implicit_type() == Some(entity.entity_type) || self.class_of(entity) == *class
    }

    /// The type a class selects among: a defined class's `of`, an implicit class's type, or [`None`] for
    /// a name that is neither.
    pub fn type_of(&self, class: &ClassName) -> Option<EntityType> {
        class.implicit_type().or_else(|| {
            self.0
                .iter()
                .find(|definition| definition.class == *class)
                .map(|definition| definition.of)
        })
    }

    /// Only the definitions that can decide whether one of `referenced` applies: each referenced
    /// definition, and every definition of the same type listed before it, which could shadow it. In
    /// priority order. What a configuration copies into its own fact, so editing a definition it cannot
    /// see is not drift (`ARC-64` item 3).
    pub fn restricted_to(&self, referenced: &BTreeSet<ClassName>) -> Self {
        let last = |entity_type: EntityType| {
            self.0.iter().rposition(|definition| {
                definition.of == entity_type && referenced.contains(&definition.class)
            })
        };
        let kept = self
            .0
            .iter()
            .enumerate()
            .filter(|(index, definition)| last(definition.of).is_some_and(|last| *index <= last))
            .map(|(_, definition)| definition.clone())
            .collect();
        Self(kept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition(class: &'static str, of: EntityType, tag: &str) -> ClassDefinition {
        ClassDefinition {
            class: ClassName::from_static(class),
            of,
            tag: Tag::new(tag).expect("a tag"),
        }
    }

    fn tags(values: &[&str]) -> Tags {
        Tags::new(values.iter().map(|value| Tag::new(*value).expect("a tag")))
    }

    /// A class defined twice and an implicit name reused are each refused with the owner's message; a
    /// valid list keeps its order. (An `of` that is no entity type is `EntityType`'s own refusal; its
    /// line and column are proven through the loader.)
    #[test]
    fn a_classes_list_refuses_a_duplicate_and_an_implicit_name() {
        let twice = EntityClasses::try_from(vec![
            definition("noble", EntityType::Person, "noble"),
            definition("noble", EntityType::Item, "gold"),
        ])
        .expect_err("defined twice");
        assert!(twice.contains("'noble' is defined twice"), "{twice}");

        let implicit = EntityClasses::try_from(vec![definition("person", EntityType::Person, "x")])
            .expect_err("an implicit name");
        assert!(
            implicit.contains("'person' is the implicit class"),
            "{implicit}"
        );

        let fine = EntityClasses::try_from(vec![
            definition("noble", EntityType::Person, "noble"),
            definition("heir", EntityType::Item, "heirloom"),
        ])
        .expect("a valid list");
        assert_eq!(
            fine.definitions()
                .iter()
                .map(|definition| definition.class.as_str())
                .collect::<Vec<_>>(),
            ["noble", "heir"]
        );
    }

    /// One class per entity, by priority, within its type; the implicit class otherwise; and the
    /// implicit selector matches every entity of its type, whatever its class.
    #[test]
    fn an_entity_is_in_the_first_matching_class_of_its_type_and_always_matches_its_implicit_class()
    {
        let classes = EntityClasses(vec![
            definition("noble", EntityType::Person, "noble"),
            definition("servant", EntityType::Person, "servant"),
            definition("staff-only", EntityType::Place, "noble"),
        ]);
        let both = tags(&["servant", "noble"]);
        let person = Classed {
            entity_type: EntityType::Person,
            tags: &both,
        };
        assert_eq!(classes.class_of(person).as_str(), "noble", "priority");
        assert!(classes.matches(&ClassName::from_static("person"), person));
        assert!(!classes.matches(&ClassName::from_static("servant"), person));

        let place = Classed {
            entity_type: EntityType::Place,
            tags: &both,
        };
        assert_eq!(
            classes.class_of(place).as_str(),
            "staff-only",
            "a person's class never applies to a place carrying the same tag"
        );
        let none = tags(&[]);
        let plain = Classed {
            entity_type: EntityType::Person,
            tags: &none,
        };
        assert_eq!(classes.class_of(plain).as_str(), "person");
        assert_eq!(
            classes.type_of(&ClassName::from_static("staff-only")),
            Some(EntityType::Place)
        );
        assert_eq!(classes.type_of(&ClassName::from_static("nobody")), None);
    }

    /// What a section copies: the classes it references and those of the same type that could shadow
    /// them, in order — and nothing listed after them.
    #[test]
    fn a_restriction_keeps_the_referenced_classes_and_whatever_could_shadow_them() {
        let classes = EntityClasses(vec![
            definition("noble", EntityType::Person, "noble"),
            definition("heir", EntityType::Item, "heirloom"),
            definition("servant", EntityType::Person, "servant"),
            definition("guard", EntityType::Person, "guard"),
        ]);
        let referenced = [ClassName::from_static("servant")].into_iter().collect();
        let kept: Vec<_> = classes
            .restricted_to(&referenced)
            .definitions()
            .iter()
            .map(|definition| definition.class.as_str().to_owned())
            .collect();
        assert_eq!(kept, ["noble", "servant"]);
        assert!(
            classes
                .restricted_to(&BTreeSet::new())
                .definitions()
                .is_empty()
        );
    }
}
