//! The `item:` section of an item file, which this pack owns (`ARC-31`, `ARC-36`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, ItemId, Rejection};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::category::Category;
use crate::event::ItemKindDeclared;
use crate::name::ItemName;
use crate::system::ItemSystem;

/// The section as authored: `item: { category: drink, name: Flat White }`.
///
/// An object rather than a bare category, so that a kind may say more about itself without changing
/// what an existing file means — as `name` now does (step-13 R-PK-2). Both keys are required; unknown
/// keys are refused, at their line.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredItem {
    category: Category,
    name: ItemName,
}

impl AuthoredItem {
    /// The authored category.
    pub const fn category(&self) -> &Category {
        &self.category
    }

    /// The authored display name.
    pub const fn name(&self) -> &ItemName {
        &self.name
    }
}

impl AuthoredSection for ItemSystem {
    /// `item: { category: drink, name: Flat White }`.
    const SECTION: SectionName = SectionName::from_static("item");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Item];

    /// Decoding it is [`Category`]'s and [`ItemName`]'s check.
    type Authored = AuthoredItem;

    /// One `item-kind-declared` about the item whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredItem,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // The loader refuses the section anywhere but an item file (`CARRIED_BY`); refused here too,
        // because the subject's type is this pack's to check, not the loader's to promise.
        let item = ItemId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            ItemKindDeclared::new(item, authored.category.clone(), authored.name.clone())
                .emission(),
        ])
    }
}
