//! `configure/economy.yaml` (`ARC-61`; WE-a design §6.2): economy's first world configuration, optional
//! and absent by default. It says which shops change their prices while which happenings run.
//!
//! ```yaml
//! price_modifiers:                                         # at most 32; (during, shop) unique
//!   - { during: market-day, shop: store, permille: 900 }   # 10 % off at the store on market days
//! ```
//!
//! `during` is a happening kind's id, matched against `happening-began` and `happening-ended` and
//! interpreted no further: economy does not learn what a market is. A kind the world's
//! `configure/happenings.yaml` does not declare cannot be cross-checked here (packs do not read each
//! other's configuration) and never matches. `shop` must be a Place; a place where no shop opens is
//! never adjusted (the shops open after the configuration is seeded, so neither can be refused at
//! load). A non-empty list requires `happenings` in the world, so removing one without the other is
//! refused by name.

use std::collections::BTreeSet;

use mineworld_contracts::{EntityKey, PlaceId};
use mineworld_happenings::HappeningKindId;
use serde::{Deserialize, Serialize};

/// The most price modifiers one world may configure.
pub const MODIFIERS_MAX: usize = 32;

/// A price multiplier per mille, 100 … 10 000: 900 is 10 % off, 1 500 half as much again.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "u16", into = "u16")]
pub struct Permille(u16);

impl Permille {
    /// The multiplier, per mille.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Permille {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if (100..=10_000).contains(&value) {
            Ok(Self(value))
        } else {
            Err(format!("permille is 100 to 10000, not {value}"))
        }
    }
}

impl From<Permille> for u16 {
    fn from(value: Permille) -> Self {
        value.0
    }
}

/// While a happening of kind `during` runs, the shop at `shop` multiplies its prices by `permille`.
/// `P` is how the shop's place is named: its key in the file, its id once seeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "PriceModifier", bound = "")]
pub struct PriceModifier<P> {
    /// The happening kind it applies during, by id; matched, never interpreted.
    during: HappeningKindId,
    /// The shop's place, by key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    shop: P,
    /// The price multiplier per mille, 100 to 10000: 900 is 10 % off.
    permille: Permille,
}

impl<P> PriceModifier<P> {
    /// The happening kind it applies during.
    pub const fn during(&self) -> &HappeningKindId {
        &self.during
    }

    /// The shop's place.
    pub const fn shop(&self) -> &P {
        &self.shop
    }

    /// The multiplier.
    pub const fn permille(&self) -> Permille {
        self.permille
    }

    /// The same modifier at a resolved place.
    pub(crate) fn at<Q>(&self, shop: Q) -> PriceModifier<Q> {
        PriceModifier {
            during: self.during.clone(),
            shop,
            permille: self.permille,
        }
    }
}

/// Economy's world configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawConfiguration")]
pub struct EconomyConfiguration {
    price_modifiers: Vec<PriceModifier<EntityKey>>,
}

/// The schema is the file's shape: deserializing goes through [`RawConfiguration`].
impl schemars::JsonSchema for EconomyConfiguration {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("EconomyConfiguration")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <RawConfiguration as schemars::JsonSchema>::json_schema(generator)
    }
}

/// The example the generated reference prints: 10 % off at the store while a market day runs.
fn example() -> serde_json::Value {
    serde_json::json!({
        "price_modifiers": [{ "during": "market-day", "shop": "store", "permille": 900 }],
    })
}

/// The file as written.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example())]
struct RawConfiguration {
    /// Which shops change their prices while which happenings run: at most 32, each (during, shop)
    /// once.
    #[serde(default)]
    price_modifiers: Vec<PriceModifier<EntityKey>>,
}

impl TryFrom<RawConfiguration> for EconomyConfiguration {
    type Error = String;

    fn try_from(raw: RawConfiguration) -> Result<Self, Self::Error> {
        let modifiers = raw.price_modifiers;
        if modifiers.len() > MODIFIERS_MAX {
            return Err(format!(
                "price_modifiers lists at most {MODIFIERS_MAX} modifiers, not {}",
                modifiers.len()
            ));
        }
        let mut seen = BTreeSet::new();
        for (index, modifier) in modifiers.iter().enumerate() {
            if !seen.insert((modifier.during(), modifier.shop())) {
                return Err(format!(
                    "price_modifiers[{index}]: '{}' at '{}' is configured twice",
                    modifier.during(),
                    modifier.shop().as_str()
                ));
            }
        }
        Ok(Self {
            price_modifiers: modifiers,
        })
    }
}

impl EconomyConfiguration {
    /// The modifiers, in file order.
    pub fn price_modifiers(&self) -> &[PriceModifier<EntityKey>] {
        &self.price_modifiers
    }
}

/// A modifier as the price book keeps it, its shop resolved.
pub type ResolvedModifier = PriceModifier<PlaceId>;
