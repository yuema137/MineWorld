//! The `holdings:` section of a person's or organization's file, which this pack owns (`ARC-31`,
//! `ARC-36`).

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{EntityId, EntityKey, EntityType, ItemId, Rejection};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::admit::{PERSON_CAPACITY, is_holder, stocked};
use crate::system::InventorySystem;

/// The section as authored: `holdings: { coffee: 2, apple: 1 }` — item key to a count of at least
/// one.
///
/// A count of zero is refused as it is decoded, at its line and column: holding none of a kind is
/// saying nothing, and two ways to say it are two chances to disagree.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct AuthoredHoldings(BTreeMap<EntityKey, NonZeroU32>);

impl AuthoredHoldings {
    /// Every kind authored, in key order, with its count.
    pub fn counts(&self) -> impl Iterator<Item = (&EntityKey, u32)> {
        self.0.iter().map(|(key, count)| (key, count.get()))
    }

    /// How many items in all.
    pub fn total(&self) -> u64 {
        self.0.values().map(|count| u64::from(count.get())).sum()
    }
}

impl AuthoredSection for InventorySystem {
    /// `holdings: { coffee: 2 }`.
    const SECTION: SectionName = SectionName::from_static("holdings");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person, ContentKind::Organization];

    type Authored = AuthoredHoldings;

    /// Every key names an Item; the loader refuses one that does not, by name, before any world
    /// exists.
    fn references(authored: &AuthoredHoldings) -> Vec<Reference<'_>> {
        authored
            .0
            .keys()
            .map(|key| Reference {
                key,
                entity_type: EntityType::Item,
            })
            .collect()
    }

    /// One `stocked` per kind, in key order, about the holder whose file it is.
    ///
    /// What can be checked here is: the subject is a holder, every key an Item, and a person's total
    /// within [`PERSON_CAPACITY`] (refused [`Rejection::TargetUnavailable`], which the loader reports
    /// naming the file). Whether each kind is *declared* cannot be: sections are seeded against a
    /// world with no state yet (step-10 F-37). The reduction asks it, after `item` has reduced its
    /// declarations, which precede this section in genesis order (`ARC-36` item 7).
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredHoldings,
    ) -> Result<Vec<Emission>, Rejection> {
        let world = seeding.world();
        if !is_holder(world, subject) {
            return Err(Rejection::PreconditionFailed);
        }
        let is_person = world
            .entity(subject)
            .is_some_and(|record| record.entity_type() == EntityType::Person);
        if is_person && authored.total() > u64::from(PERSON_CAPACITY) {
            return Err(Rejection::TargetUnavailable);
        }
        authored
            .counts()
            .map(|(key, count)| {
                let item = seeding
                    .resolve(key, EntityType::Item)
                    .ok_or(Rejection::PreconditionFailed)?;
                let item = ItemId::new(item, EntityType::Item)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(stocked(subject, item, count))
            })
            .collect()
    }
}
