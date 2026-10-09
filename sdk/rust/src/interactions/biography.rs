//! The selection the biography projection asks (`ARC-65` item 4, the `ARC-29` note): whether a fact
//! enters a person's biography, given the compiled set and the save's configured sections.
//!
//! Pure, and type-erased: the projection reads a save, not a pack's types, so each pack's configured
//! fact is read into a [`ConsequenceTable`] by the function its `SectionDecl` names.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_authoring::EntityClasses;
use mineworld_contracts::{EntityId, EntityKey, EntityType, EventEnvelope, EventTypeId, Tags};
use serde::{Deserialize, Serialize};

use super::InteractionSection;
use super::decl::{FactDecl, Position};
use super::resolve::Resolution;
use super::selector::{RoleSubjects, Selectors};

/// One consequence entry, as far as a biography needs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BiographyRow {
    fact: EventTypeId,
    selectors: Selectors,
    biography: Option<bool>,
}

/// A configured section's biographical answers: its classes, its base and each region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsequenceTable {
    classes: EntityClasses,
    base: Vec<BiographyRow>,
    regions: BTreeMap<EntityKey, Vec<BiographyRow>>,
}

fn rows<S: InteractionSection>(resolution: &Resolution<S>) -> Vec<BiographyRow> {
    resolution
        .consequences
        .iter()
        .filter(|entry| entry.biography.is_some())
        .map(|entry| BiographyRow {
            fact: entry.fact.clone(),
            selectors: entry.selectors.clone(),
            biography: entry.biography,
        })
        .collect()
}

/// Reads `S`'s configured fact into its [`ConsequenceTable`]: what `SectionDecl::consequences` names.
///
/// # Errors
///
/// The pack's codec refusing the payload.
pub fn consequences<S: InteractionSection>(payload: &[u8]) -> Result<ConsequenceTable, String> {
    let resolved = S::decode(payload)?;
    Ok(ConsequenceTable {
        classes: resolved.classes.clone(),
        base: rows(&resolved.base),
        regions: resolved
            .regions
            .iter()
            .map(|(place, resolution)| (place.clone(), rows(resolution)))
            .collect(),
    })
}

/// An entity as the selection sees it: its key, type and tags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Known {
    /// Its authoring key.
    pub key: EntityKey,
    /// Its type.
    pub entity_type: EntityType,
    /// Its tags.
    pub tags: Tags,
}

/// What a save says about its world's configured sections, for the projection.
#[derive(Debug, Clone, Default)]
pub struct Configured {
    facts: BTreeMap<EventTypeId, (FactDecl, ConsequenceTable)>,
    entities: BTreeMap<EntityId, Known>,
}

impl Configured {
    /// Nothing configured: the selection is `ARC-29` exactly.
    pub fn none() -> Self {
        Self::default()
    }

    /// The world's entities.
    #[must_use]
    pub fn with_entities(mut self, entities: BTreeMap<EntityId, Known>) -> Self {
        self.entities = entities;
        self
    }

    /// A configured section's table, for each fact its owner declares.
    #[must_use]
    pub fn with_section(mut self, facts: &[FactDecl], table: &ConsequenceTable) -> Self {
        for fact in facts {
            self.facts
                .insert(fact.fact.clone(), (fact.clone(), table.clone()));
        }
        self
    }
}

/// The entity filling `position` of `fact`'s envelope.
fn filled(fact: &EventEnvelope, position: Position) -> Option<EntityId> {
    match position {
        Position::Subject(index) => fact.subjects().get(index).copied(),
        Position::Participant(index) => fact.participants().get(index).copied(),
        Position::Place => fact.place().map(|place| place.entity_id()),
    }
}

/// Whether `fact` enters `person`'s biography (`ARC-29`, `ARC-65`): the person is among its subjects
/// or participants, and — when its owner's section is configured and says so for the fact's roles —
/// the section's flag, otherwise whether its type is in the compiled set.
pub fn selected(
    fact: &EventEnvelope,
    person: EntityId,
    compiled: &BTreeSet<EventTypeId>,
    configured: &Configured,
) -> bool {
    let named = fact.subjects().contains(&person) || fact.participants().contains(&person);
    if !named {
        return false;
    }
    let compiled = compiled.contains(fact.event_type());
    let Some((declared, table)) = configured.facts.get(fact.event_type()) else {
        return compiled;
    };
    let mut subjects = RoleSubjects::none();
    for (role, position) in declared.roles {
        if let Some(known) = filled(fact, *position).and_then(|id| configured.entities.get(&id)) {
            subjects = subjects.with(*role, known.entity_type, &known.tags);
        }
    }
    let place = fact
        .place()
        .and_then(|place| configured.entities.get(&place.entity_id()))
        .map(|known| &known.key);
    let rows = place
        .and_then(|place| table.regions.get(place))
        .unwrap_or(&table.base);
    let mut matching: Vec<&BiographyRow> = rows
        .iter()
        .filter(|row| row.fact == *fact.event_type())
        .filter(|row| row.selectors.matches(&table.classes, &subjects))
        .collect();
    matching.sort_by_key(|row| row.selectors.specificity());
    matching
        .into_iter()
        .rev()
        .find_map(|row| row.biography)
        .unwrap_or(compiled)
}
