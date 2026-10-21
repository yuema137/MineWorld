//! The `economy:` section of a person's, organization's or item's file, which this pack owns (`ARC-31`,
//! `ARC-36`, `ARC-112`, `ARC-113`).

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, ItemId, OrganizationId, PersonId, PlaceId, Rejection,
    RejectionCode, Visibility,
};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::code::{
    self, ASKS_ON_ORGANIZATION, ITEM_REFERENCE_ONLY, KEEPER_DUPLICATE, KEEPERS_TOO_MANY,
    PRICE_OUT_OF_RANGE, REFERENCE_ON_HOLDER, SHOP_ON_PERSON, WALLET_UNRESOLVED,
};
use crate::codec;
use crate::component::MAX_KEEPERS;
use crate::event::{Funded, Listed, ReferenceDeclared, ShopOpened};
use crate::money::{is_holder, price_in_range};
use crate::system::EconomySystem;

/// Money and selling: what this person or organization has when the world begins and what it offers
/// for sale, or what this item kind is typically worth.
///
/// ```yaml
/// economy:                          # an organization
///   wallet: 300000                  # minor units, at genesis
///   shop:
///     at: store                     # a place key: the shop is there
///     prices: { apple: 100 }        # item key → asking price, 1 … 10^12
///     keepers: [felix]              # people who may change its prices while in the shop
/// economy: { wallet: 20000, asks: { flowers: 450 } }    # a person, selling for itself
/// economy: { reference: 300 }                           # an item kind: its typical worth
/// ```
///
/// A pack owns one section, so a shop is authored on the organization that runs it (step-10 F-47,
/// QS-43). Negative or fractional amounts are refused as they are decoded, at their line; which keys a
/// kind of file may carry is refused at genesis, by code.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    example =serde_json::json!({ "wallet": 2000, "shop": { "at": "shop", "prices": { "bread": 250 } } })
)]
pub struct AuthoredEconomy {
    /// The money held when the world begins, in minor units of the world's currency (with two decimal
    /// places, 2000 is 20.00). Required on a person or an organization; never on an item.
    #[serde(default)]
    wallet: Option<u64>,
    /// The shop this holder runs. Only an organization runs one; on a person's file it is refused.
    #[serde(default)]
    shop: Option<AuthoredShop>,
    /// What this person offers to sell, for itself: an item key, then its asking price, 1 to 10^12.
    /// Only a person; an organization sells through its shop's `prices`.
    #[serde(default)]
    #[schemars(with = "BTreeMap<mineworld_authoring::schema::Key, NonZeroU64>")]
    asks: BTreeMap<EntityKey, NonZeroU64>,
    /// This item kind's typical worth, 1 to 10^12: never charged, used for wealth and as a pricing
    /// default. Only on an item's file, and then nothing else.
    #[serde(default)]
    reference: Option<NonZeroU64>,
}

/// A shop, as its operator authors it: where it is, what it sells at what price, and who keeps it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredShop {
    /// The place the shop is in: a place key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    at: EntityKey,
    /// What it sells: an item key, then its asking price in minor units, 1 to 10^12. These are the
    /// organization's asks.
    #[schemars(with = "BTreeMap<mineworld_authoring::schema::Key, NonZeroU64>")]
    prices: BTreeMap<EntityKey, NonZeroU64>,
    /// The people who may list, re-price and withdraw the shop's goods while standing in it: person
    /// keys, each once, at most 8. Without keepers the shop keeps its opening prices.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    keepers: Vec<EntityKey>,
}

impl AuthoredEconomy {
    /// The opening balance, if authored.
    pub const fn wallet(&self) -> Option<u64> {
        self.wallet
    }

    /// The shop, if this holder runs one.
    pub const fn shop(&self) -> Option<&AuthoredShop> {
        self.shop.as_ref()
    }
}

/// A seeding refusal carrying economy's `code`.
const fn refused(code: RejectionCode) -> Rejection {
    code::refusal(code)
}

impl AuthoredSection for EconomySystem {
    /// `economy: { wallet, shop, asks, reference }`.
    const SECTION: SectionName = SectionName::from_static("economy");
    const CARRIED_BY: &'static [ContentKind] = &[
        ContentKind::Person,
        ContentKind::Organization,
        ContentKind::Item,
    ];

    type Authored = AuthoredEconomy;

    /// A shop's place is a Place, every priced or asked key an Item, every keeper a Person; the loader
    /// refuses any other by name before a world exists.
    fn references(authored: &AuthoredEconomy) -> Vec<Reference<'_>> {
        let mut references = Vec::new();
        if let Some(shop) = &authored.shop {
            references.push(Reference {
                key: &shop.at,
                entity_type: EntityType::Place,
            });
            references.extend(shop.prices.keys().map(|key| Reference {
                key,
                entity_type: EntityType::Item,
            }));
            references.extend(shop.keepers.iter().map(|key| Reference {
                key,
                entity_type: EntityType::Person,
            }));
        }
        references.extend(authored.asks.keys().map(|key| Reference {
            key,
            entity_type: EntityType::Item,
        }));
        references
    }

    /// Per kind of file, in this order (`pr-s31a-currency-asks.md` §5.4):
    ///
    /// - an item: `reference-declared`, SystemInternal (Q-S31a-2);
    /// - an organization: `funded`; for a shop, `shop-opened` (Public, at the place), then one `listed`
    ///   per price, heard in the shop;
    /// - a person: `funded`, then one `listed` per ask, heard by the person (its place is not reduced
    ///   while sections are seeded, SD-S31a-13).
    ///
    /// A key a kind of file may not carry, a missing wallet, a keeper twice or a ninth, and an amount
    /// above 10^12 are refused by code; the loader reports it naming the file.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredEconomy,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        match entity_type {
            EntityType::Item => seed_item(subject, authored),
            EntityType::Person | EntityType::Organization => {
                if !is_holder(seeding.world(), subject) {
                    return Err(Rejection::PreconditionFailed);
                }
                seed_holder(seeding, subject, entity_type, authored)
            }
            EntityType::Place => Err(Rejection::PreconditionFailed),
        }
    }
}

/// Whether an authored amount is one this pack takes.
fn in_range(amount: NonZeroU64) -> Result<u64, Rejection> {
    if price_in_range(amount.get()) {
        Ok(amount.get())
    } else {
        Err(refused(PRICE_OUT_OF_RANGE))
    }
}

/// An item's `economy:`: its reference value and nothing else.
fn seed_item(subject: EntityId, authored: &AuthoredEconomy) -> Result<Vec<Emission>, Rejection> {
    let only_reference =
        authored.wallet.is_none() && authored.shop.is_none() && authored.asks.is_empty();
    let (true, Some(reference)) = (only_reference, authored.reference) else {
        return Err(refused(ITEM_REFERENCE_ONLY));
    };
    let value = in_range(reference)?;
    let item = ItemId::new(subject, EntityType::Item).map_err(|_| Rejection::PreconditionFailed)?;
    Ok(vec![Emission::new::<ReferenceDeclared>(
        codec::encode(&ReferenceDeclared::new(item, value)),
        Visibility::SystemInternal,
    )])
}

/// A person's or organization's `economy:`: its wallet, and what it sells.
fn seed_holder(
    seeding: &Seeding<'_, '_>,
    subject: EntityId,
    entity_type: EntityType,
    authored: &AuthoredEconomy,
) -> Result<Vec<Emission>, Rejection> {
    let person = entity_type == EntityType::Person;
    if authored.reference.is_some() {
        return Err(refused(REFERENCE_ON_HOLDER));
    }
    if person && authored.shop.is_some() {
        return Err(refused(SHOP_ON_PERSON));
    }
    if !person && !authored.asks.is_empty() {
        return Err(refused(ASKS_ON_ORGANIZATION));
    }
    let Some(wallet) = authored.wallet else {
        return Err(refused(WALLET_UNRESOLVED));
    };
    let mut facts = vec![
        Emission::new::<Funded>(
            codec::encode(&Funded::new(subject, wallet)),
            Visibility::Entities([subject].into_iter().collect()),
        )
        .about(vec![subject])
        .with_participants(vec![subject]),
    ];
    if person {
        let heard = Visibility::Entities([subject].into_iter().collect());
        for (key, price) in &authored.asks {
            let item = item(seeding, key)?;
            facts.push(
                Emission::new::<Listed>(
                    codec::encode(&Listed::new(subject, item, in_range(*price)?)),
                    heard.clone(),
                )
                .about(vec![subject]),
            );
        }
        return Ok(facts);
    }
    if let Some(shop) = &authored.shop {
        facts.extend(seed_shop(seeding, subject, entity_type, shop)?);
    }
    Ok(facts)
}

/// A shop's `shop-opened`, then its prices as the operator's `listed` facts, heard in the shop.
fn seed_shop(
    seeding: &Seeding<'_, '_>,
    subject: EntityId,
    entity_type: EntityType,
    shop: &AuthoredShop,
) -> Result<Vec<Emission>, Rejection> {
    let refused_value = |_| Rejection::PreconditionFailed;
    if shop.keepers.len() > MAX_KEEPERS {
        return Err(refused(KEEPERS_TOO_MANY));
    }
    if shop.keepers.iter().collect::<BTreeSet<_>>().len() != shop.keepers.len() {
        return Err(refused(KEEPER_DUPLICATE));
    }
    let operator = OrganizationId::new(subject, entity_type).map_err(refused_value)?;
    let place = seeding
        .resolve(&shop.at, EntityType::Place)
        .ok_or(Rejection::PreconditionFailed)?;
    let place = PlaceId::new(place, EntityType::Place).map_err(refused_value)?;
    let keepers = shop
        .keepers
        .iter()
        .map(|key| {
            let person = seeding
                .resolve(key, EntityType::Person)
                .ok_or(Rejection::PreconditionFailed)?;
            PersonId::new(person, EntityType::Person).map_err(refused_value)
        })
        .collect::<Result<Vec<_>, Rejection>>()?;
    let mut facts = vec![
        Emission::new::<ShopOpened>(
            codec::encode(&ShopOpened::new(place, operator, keepers)),
            Visibility::Public,
        )
        .about(vec![place.entity_id(), subject])
        .at_place(place),
    ];
    let mut listed = shop
        .prices
        .iter()
        .map(|(key, price)| Ok((item(seeding, key)?, in_range(*price)?)))
        .collect::<Result<Vec<_>, Rejection>>()?;
    listed.sort_by_key(|(item, _)| *item);
    for (item, price) in listed {
        facts.push(
            Emission::new::<Listed>(
                codec::encode(&Listed::new(subject, item, price)),
                Visibility::Place(place),
            )
            .about(vec![subject])
            .at_place(place),
        );
    }
    Ok(facts)
}

/// An item key as the kind it names.
fn item(seeding: &Seeding<'_, '_>, key: &EntityKey) -> Result<ItemId, Rejection> {
    let item = seeding
        .resolve(key, EntityType::Item)
        .ok_or(Rejection::PreconditionFailed)?;
    ItemId::new(item, EntityType::Item).map_err(|_| Rejection::PreconditionFailed)
}
