//! The `economy:` section of a person's or organization's file, which this pack owns (`ARC-31`,
//! `ARC-36`).

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, ItemId, OrganizationId, PlaceId, Rejection, Visibility,
};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::codec;
use crate::component::Price;
use crate::event::{Funded, ShopOpened};
use crate::money::is_holder;
use crate::system::EconomySystem;

/// The section as authored:
///
/// ```yaml
/// economy:
///   wallet: 20000                  # minor units, at genesis
///   shop:                          # an organization only: the shop it runs
///     at: cafe                     # a place key
///     prices: { coffee: 300 }      # item key → price in minor units, at least 1
/// ```
///
/// A pack owns one section, so a shop is authored on the organization that runs it (step-10 F-47,
/// QS-43). Negative or fractional amounts are refused as they are decoded, at their line.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredEconomy {
    wallet: u64,
    #[serde(default)]
    shop: Option<AuthoredShop>,
}

/// A shop, as its operator authors it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredShop {
    at: EntityKey,
    prices: BTreeMap<EntityKey, NonZeroU64>,
}

impl AuthoredEconomy {
    /// The opening balance.
    pub const fn wallet(&self) -> u64 {
        self.wallet
    }

    /// The shop, if this holder runs one.
    pub const fn shop(&self) -> Option<&AuthoredShop> {
        self.shop.as_ref()
    }
}

impl AuthoredSection for EconomySystem {
    /// `economy: { wallet, shop }`.
    const SECTION: SectionName = SectionName::from_static("economy");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person, ContentKind::Organization];

    type Authored = AuthoredEconomy;

    /// A shop's place is a Place and every priced key an Item; the loader refuses any other by name
    /// before a world exists.
    fn references(authored: &AuthoredEconomy) -> Vec<Reference<'_>> {
        let Some(shop) = &authored.shop else {
            return Vec::new();
        };
        let mut references = vec![Reference {
            key: &shop.at,
            entity_type: EntityType::Place,
        }];
        references.extend(shop.prices.keys().map(|key| Reference {
            key,
            entity_type: EntityType::Item,
        }));
        references
    }

    /// One `funded` about the holder whose file it is; for an organization with a shop, then one
    /// `shop-opened` about the place. A shop on a person's file is refused: only an organization runs
    /// one (the loader reports it naming the file).
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredEconomy,
    ) -> Result<Vec<Emission>, Rejection> {
        let world = seeding.world();
        if !is_holder(world, subject) {
            return Err(Rejection::PreconditionFailed);
        }
        let mut facts = vec![
            Emission::new::<Funded>(
                codec::encode(&Funded::new(subject, authored.wallet)),
                Visibility::Entities([subject].into_iter().collect()),
            )
            .about(vec![subject])
            .with_participants(vec![subject]),
        ];
        let Some(shop) = &authored.shop else {
            return Ok(facts);
        };
        let refused = |_| Rejection::PreconditionFailed;
        let entity_type = world
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        let operator = OrganizationId::new(subject, entity_type).map_err(refused)?;
        let place = seeding
            .resolve(&shop.at, EntityType::Place)
            .ok_or(Rejection::PreconditionFailed)?;
        let place = PlaceId::new(place, EntityType::Place).map_err(refused)?;
        let mut prices = shop
            .prices
            .iter()
            .map(|(key, price)| {
                let item = seeding
                    .resolve(key, EntityType::Item)
                    .ok_or(Rejection::PreconditionFailed)?;
                let item = ItemId::new(item, EntityType::Item).map_err(refused)?;
                Ok(Price::new(item, price.get()))
            })
            .collect::<Result<Vec<_>, Rejection>>()?;
        prices.sort_by_key(Price::item);
        facts.push(
            Emission::new::<ShopOpened>(
                codec::encode(&ShopOpened::new(place, operator, prices)),
                Visibility::Public,
            )
            .about(vec![place.entity_id(), subject])
            .at_place(place),
        );
        Ok(facts)
    }
}
