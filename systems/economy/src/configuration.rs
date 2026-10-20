//! `configure/economy.yaml`, decoded by this pack's own types (`ARC-61`, `ARC-111`; WE-a design §6.2):
//! deserializing is validating, so a value out of bounds is refused at assembly with the file, its line
//! and column, and a message naming the key.
//!
//! ```yaml
//! currency:                                                # required (ARC-111)
//!   name: mark          # 1 … 64 bytes of text, no control characters; display only
//!   symbol: "M"         # 1 … 8 bytes, no control characters, no whitespace at either end; display only
//!   exponent: 2         # minor units per major unit = 10^exponent; 0 … 4 (ISO 4217's minor-unit column)
//!   symbol_at: before   # before | after
//! price_modifiers:                                         # optional; at most 32; (during, shop) unique
//!   - { during: market-day, shop: store, permille: 900 }   # 10 % off at the store on market days
//! ```
//!
//! No currency name, symbol or exponent is a constant anywhere in code: a world states its own, and a
//! fantasy world may declare a coin no ISO table holds (`DEP-63`).
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

/// Economy's world configuration: its one currency, and the price modifiers it answers happenings with.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawConfiguration")]
pub struct EconomyConfiguration {
    currency: Currency,
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

/// The example the generated reference prints: a currency, and 10 % off at the store while a market
/// day runs.
fn example() -> serde_json::Value {
    serde_json::json!({
        "currency": { "name": "mark", "symbol": "M", "exponent": 2, "symbol_at": "before" },
        "price_modifiers": [{ "during": "market-day", "shop": "store", "permille": 900 }],
    })
}

/// The file as written.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example())]
struct RawConfiguration {
    /// The world's one currency, in which every wallet, price and reference value is counted. Required:
    /// a world that enables economy and authors any amount must say what its amounts are.
    currency: Currency,
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
            currency: raw.currency,
            price_modifiers: modifiers,
        })
    }
}

impl EconomyConfiguration {
    /// The world's currency.
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    /// The modifiers, in file order.
    pub fn price_modifiers(&self) -> &[PriceModifier<EntityKey>] {
        &self.price_modifiers
    }
}

/// A modifier as the price book keeps it, its shop resolved.
pub type ResolvedModifier = PriceModifier<PlaceId>;

/// A currency: what it is called, its symbol, how many minor units make one major unit, and which side of
/// an amount the symbol is written on. Display data only: no amount changes with any of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Currency {
    /// Its name, such as "mark" or "shell": 1 to 64 bytes of text with no control characters.
    name: CurrencyName,
    /// Its symbol, such as "M" or "sh": 1 to 8 bytes, no control characters, no whitespace at either end.
    symbol: CurrencySymbol,
    /// How many digits of an amount are minor units, from 0 to 4: 2 means 250 is shown as 2.50 (ISO
    /// 4217's minor-unit column: JPY 0, USD 2, KWD 3, CLF 4).
    exponent: Exponent,
    /// Where the symbol is written: `before` the amount ($2.50) or `after` it (3 cr).
    symbol_at: SymbolAt,
}

impl Currency {
    /// Its name.
    pub fn name(&self) -> &str {
        &self.name.0
    }

    /// Its symbol.
    pub fn symbol(&self) -> &str {
        &self.symbol.0
    }

    /// How many digits of an amount are minor units.
    pub const fn exponent(&self) -> u8 {
        self.exponent.0
    }

    /// Which side of an amount the symbol is written on.
    pub const fn symbol_at(&self) -> SymbolAt {
        self.symbol_at
    }
}

/// Whether text has a control character in it.
fn has_control(text: &str) -> bool {
    text.chars().any(char::is_control)
}

/// A currency's name: 1 to 64 bytes of text, no control characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String", extend("minLength" = 1, "maxLength" = 64))]
struct CurrencyName(String);

impl TryFrom<String> for CurrencyName {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.is_empty() || text.len() > 64 || has_control(&text) {
            return Err(format!(
                "name is 1 to 64 bytes of text with no control characters, not {text:?} ({} bytes)",
                text.len()
            ));
        }
        Ok(Self(text))
    }
}

impl From<CurrencyName> for String {
    fn from(name: CurrencyName) -> Self {
        name.0
    }
}

/// A currency's symbol: 1 to 8 bytes, no control characters, no whitespace at either end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
#[schemars(with = "String", extend("minLength" = 1, "maxLength" = 8))]
struct CurrencySymbol(String);

impl TryFrom<String> for CurrencySymbol {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.is_empty() || text.len() > 8 || has_control(&text) || text.trim() != text {
            return Err(format!(
                "symbol is 1 to 8 bytes with no control characters and no whitespace at either end, \
                 not {text:?} ({} bytes)",
                text.len()
            ));
        }
        Ok(Self(text))
    }
}

impl From<CurrencySymbol> for String {
    fn from(symbol: CurrencySymbol) -> Self {
        symbol.0
    }
}

/// How many digits of an amount are minor units: 0 to 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "u64", into = "u8")]
#[schemars(with = "u8", extend("minimum" = 0, "maximum" = 4))]
struct Exponent(u8);

impl TryFrom<u64> for Exponent {
    type Error = String;

    fn try_from(digits: u64) -> Result<Self, Self::Error> {
        u8::try_from(digits)
            .ok()
            .filter(|digits| *digits <= 4)
            .map(Self)
            .ok_or_else(|| format!("exponent is 0 to 4 (ISO 4217's minor units), not {digits}"))
    }
}

impl From<Exponent> for u8 {
    fn from(exponent: Exponent) -> Self {
        exponent.0
    }
}

/// Which side of an amount a currency's symbol is written on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", rename_all = "snake_case")]
#[schemars(extend("enum" = ["before", "after"]))]
pub enum SymbolAt {
    /// Before the amount: $2.50.
    Before,
    /// After the amount: 3 cr.
    After,
}

impl TryFrom<String> for SymbolAt {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        match text.as_str() {
            "before" => Ok(Self::Before),
            "after" => Ok(Self::After),
            _ => Err(format!("symbol_at is `before` or `after`, not '{text}'")),
        }
    }
}
