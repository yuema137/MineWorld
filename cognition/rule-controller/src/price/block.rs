//! The `pricing:` block of `rule` and `motivated` (`ARC-117`; `pr-s31d-reporter-policies.md` §5.4).
//!
//! Configured once per model (a decision list's parameters are per model, `ARC-84`), so each half names
//! the seats it applies to by their own tags. Decoded by hand, as `motivated`'s block: an unknown key is
//! refused listing the known ones, a bound is refused at its value, a key written twice is refused. Every
//! key may be omitted; the defaults decide exactly as a model without the block (`hold`, `any`).

use core::fmt::{self, Display};

use mineworld_contracts::Tag;
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

/// The most tags one half may list.
pub const TAGS_AT_MOST: usize = 8;

/// The most item categories `sell.list.categories` may list.
pub const CATEGORIES_AT_MOST: usize = 16;

/// The longest item category, in bytes: item's own rule, restated (a controller may not name `item`,
/// `ARC-35` check 2; `systems/economy/tests/controller_wire.rs` holds the two equal).
pub const CATEGORY_MAX_BYTES: usize = 32;

/// The whole block: how a seat prices what it sells, and how it chooses what it buys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Pricing {
    /// How a seat prices its own asks, and a shop's it keeps.
    pub sell: Sell,
    /// How a seat chooses among offers to buy one kind.
    pub buy: Buy,
}

/// What a seller does with its asks.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SellPolicy {
    /// Never move a price.
    #[default]
    Hold,
    /// Move a bounded step toward the current suggestion on the notice board.
    TowardSuggestion,
    /// Raise when stock is low, lower when it is full.
    Scarcity,
}

/// Which offer to buy a kind from, once a model has chosen to buy it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum BuyChoice {
    /// The offer the model chose: today's behaviour.
    #[default]
    Any,
    /// The cheapest available offer of the same kind; the first in observation order on a tie.
    Cheapest,
}

/// Which held kinds a seller lists at their reference value.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct ListRule {
    /// Item categories, 0 … 16: a held kind of one of these with no ask of mine is listed. Empty: none.
    #[schemars(length(max = 16))]
    pub categories: Vec<String>,
}

/// The seller half.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Sell {
    /// `hold`, `toward-suggestion` or `scarcity`.
    pub policy: SellPolicy,
    /// The pricing period in minutes, 60 … 10 080: each priceable ask is considered once per period.
    #[schemars(range(min = 60, max = 10_080))]
    pub every_minutes: u32,
    /// Per mille, 1 … 500: how far one move goes.
    #[schemars(range(min = 1, max = 500))]
    pub step_permille: u32,
    /// Per mille of the ask, 0 … 500: a gap no larger than this is ignored.
    #[schemars(range(min = 0, max = 500))]
    pub deadband_permille: u32,
    /// Per mille of the reference value, 1 … 1 000: the lowest ask a policy proposes.
    #[schemars(range(min = 1, max = 1_000))]
    pub floor_permille: u32,
    /// Per mille of the reference value, 1 000 … 10 000: the highest ask a policy proposes.
    #[schemars(range(min = 1_000, max = 10_000))]
    pub ceiling_permille: u32,
    /// `scarcity`: fewer than this in stock raises the ask, 0 … 1 000.
    #[schemars(range(min = 0, max = 1_000))]
    pub low_below: u32,
    /// `scarcity`: more than this in stock lowers the ask, 1 … 100 000.
    #[schemars(range(min = 1, max = 100_000))]
    pub full_above: u32,
    /// Which held kinds are listed at their reference value.
    pub list: ListRule,
    /// Only seats carrying one of these tags, 0 … 8; empty: every seat on this model.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub tags: Vec<Tag>,
    // S31-h: `accept_at_least_permille` attaches here (`pr-s31d-reporter-policies.md` §5.6).
}

impl Pricing {
    /// The defaults: hold every price, buy what the model chose.
    pub const DEFAULT: Self = Self {
        sell: Sell::DEFAULT,
        buy: Buy::DEFAULT,
    };
}

impl Default for Pricing {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Sell {
    /// The defaults, fixed before any run (`ARC-23`).
    pub const DEFAULT: Self = Self {
        policy: SellPolicy::Hold,
        every_minutes: 240,
        step_permille: 100,
        deadband_permille: 30,
        floor_permille: 500,
        ceiling_permille: 2_000,
        low_below: 2,
        full_above: 10,
        list: ListRule {
            categories: Vec::new(),
        },
        tags: Vec::new(),
    };
}

impl Default for Sell {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Buy {
    /// The defaults: buy what the model chose, every seat.
    pub const DEFAULT: Self = Self {
        choose: BuyChoice::Any,
        tags: Vec::new(),
    };
}

impl Default for Buy {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// The buyer half.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Buy {
    /// `any` or `cheapest`.
    pub choose: BuyChoice,
    /// Only seats carrying one of these tags, 0 … 8; empty: every seat on this model.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub tags: Vec<Tag>,
    // S31-h: `haggle: { offer_permille, deadband_permille }` attaches here (§5.6).
}

impl Sell {
    /// Whether this half can ever submit anything: a moving policy, or a list rule.
    pub fn is_active(&self) -> bool {
        self.policy != SellPolicy::Hold || !self.list.categories.is_empty()
    }
}

const PRICING_KEYS: &[&str] = &["sell", "buy"];
const SELL_KEYS: &[&str] = &[
    "policy",
    "every_minutes",
    "step_permille",
    "deadband_permille",
    "floor_permille",
    "ceiling_permille",
    "low_below",
    "full_above",
    "list",
    "tags",
];
const BUY_KEYS: &[&str] = &["choose", "tags"];
const LIST_KEYS: &[&str] = &["categories"];

/// A value inside `low ..= high`, or a refusal naming the bound.
fn within<E: DeError>(key: &str, value: u32, low: u32, high: u32) -> Result<u32, E> {
    if value < low || value > high {
        Err(E::custom(format!("'{key}' is {low} … {high}, not {value}")))
    } else {
        Ok(value)
    }
}

/// A key not written before, or a refusal.
fn first_time<E: DeError>(seen: &mut Vec<String>, key: &str) -> Result<(), E> {
    if seen.iter().any(|written| written == key) {
        return Err(E::custom(format!("'{key}' is written twice")));
    }
    seen.push(key.to_owned());
    Ok(())
}

/// At most [`TAGS_AT_MOST`] tags, or a refusal naming the key.
fn tags<E: DeError>(key: &str, tags: Vec<Tag>) -> Result<Vec<Tag>, E> {
    if tags.len() > TAGS_AT_MOST {
        return Err(E::custom(format!(
            "'{key}' lists at most {TAGS_AT_MOST} tags, not {}",
            tags.len()
        )));
    }
    Ok(tags)
}

/// Whether `category` keeps item's category rule: 1 … 32 bytes of a-z, 0-9 and '-', not beginning or
/// ending with '-'.
pub fn is_category(category: &str) -> bool {
    !category.is_empty()
        && category.len() <= CATEGORY_MAX_BYTES
        && category
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !category.starts_with('-')
        && !category.ends_with('-')
}

struct Named<'a>(&'a str, &'a [&'a str]);

impl Display for Named<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a {} block ({})", self.0, self.1.join(", "))
    }
}

impl<'de> Deserialize<'de> for Pricing {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(PricingFields)
    }
}

struct PricingFields;

impl<'de> Visitor<'de> for PricingFields {
    type Value = Pricing;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", Named("pricing", PRICING_KEYS))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Pricing, A::Error> {
        let mut pricing = Pricing::default();
        let mut seen = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            first_time::<A::Error>(&mut seen, &key)?;
            match key.as_str() {
                "sell" => pricing.sell = map.next_value()?,
                "buy" => pricing.buy = map.next_value()?,
                _ => return Err(A::Error::unknown_field(&key, PRICING_KEYS)),
            }
        }
        Ok(pricing)
    }
}

impl<'de> Deserialize<'de> for Sell {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(SellFields)
    }
}

struct SellFields;

impl<'de> Visitor<'de> for SellFields {
    type Value = Sell;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", Named("sell", SELL_KEYS))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Sell, A::Error> {
        let mut sell = Sell::default();
        let mut seen = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            first_time::<A::Error>(&mut seen, &key)?;
            let bounded = |low, high, value| within::<A::Error>(&key, value, low, high);
            match key.as_str() {
                "policy" => sell.policy = map.next_value()?,
                "every_minutes" => sell.every_minutes = bounded(60, 10_080, map.next_value()?)?,
                "step_permille" => sell.step_permille = bounded(1, 500, map.next_value()?)?,
                "deadband_permille" => {
                    sell.deadband_permille = bounded(0, 500, map.next_value()?)?;
                }
                "floor_permille" => sell.floor_permille = bounded(1, 1_000, map.next_value()?)?,
                "ceiling_permille" => {
                    sell.ceiling_permille = bounded(1_000, 10_000, map.next_value()?)?;
                }
                "low_below" => sell.low_below = bounded(0, 1_000, map.next_value()?)?,
                "full_above" => sell.full_above = bounded(1, 100_000, map.next_value()?)?,
                "list" => sell.list = map.next_value()?,
                "tags" => sell.tags = tags::<A::Error>("sell.tags", map.next_value()?)?,
                _ => return Err(A::Error::unknown_field(&key, SELL_KEYS)),
            }
        }
        Ok(sell)
    }
}

impl<'de> Deserialize<'de> for Buy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(BuyFields)
    }
}

struct BuyFields;

impl<'de> Visitor<'de> for BuyFields {
    type Value = Buy;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", Named("buy", BUY_KEYS))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Buy, A::Error> {
        let mut buy = Buy::default();
        let mut seen = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            first_time::<A::Error>(&mut seen, &key)?;
            match key.as_str() {
                "choose" => buy.choose = map.next_value()?,
                "tags" => buy.tags = tags::<A::Error>("buy.tags", map.next_value()?)?,
                _ => return Err(A::Error::unknown_field(&key, BUY_KEYS)),
            }
        }
        Ok(buy)
    }
}

impl<'de> Deserialize<'de> for ListRule {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ListFields)
    }
}

struct ListFields;

impl<'de> Visitor<'de> for ListFields {
    type Value = ListRule;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", Named("list", LIST_KEYS))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<ListRule, A::Error> {
        let mut list = ListRule::default();
        let mut seen = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            first_time::<A::Error>(&mut seen, &key)?;
            match key.as_str() {
                "categories" => {
                    let categories: Vec<String> = map.next_value()?;
                    if categories.len() > CATEGORIES_AT_MOST {
                        return Err(A::Error::custom(format!(
                            "'categories' lists at most {CATEGORIES_AT_MOST} categories, not {}",
                            categories.len()
                        )));
                    }
                    if let Some(bad) = categories.iter().find(|category| !is_category(category)) {
                        return Err(A::Error::custom(format!(
                            "'{bad}' is not an item category: 1 to {CATEGORY_MAX_BYTES} bytes of \
                             a-z, 0-9 and '-', not beginning or ending with '-'"
                        )));
                    }
                    list.categories = categories;
                }
                _ => return Err(A::Error::unknown_field(&key, LIST_KEYS)),
            }
        }
        Ok(list)
    }
}
