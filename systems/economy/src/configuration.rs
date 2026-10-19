//! `configure/economy.yaml`, decoded by this pack's own types (`ARC-61`, `ARC-111`): deserializing is
//! validating, so a value out of bounds is refused at assembly with the file, its line and column, and a
//! message naming the key.
//!
//! ```yaml
//! currency:
//!   name: US dollar     # 1 … 64 bytes of text, no control characters; display only
//!   symbol: "$"         # 1 … 8 bytes, no control characters, no whitespace at either end; display only
//!   exponent: 2         # minor units per major unit = 10^exponent; 0 … 4 (ISO 4217's minor-unit column)
//!   symbol_at: before   # before | after
//! ```
//!
//! No currency name, symbol or exponent is a constant anywhere in code: a world states its own, and a
//! fantasy world may declare a coin no ISO table holds (`DEP-63`).

use serde::{Deserialize, Serialize};

/// What a world configures about its economy: its one currency.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({
    "currency": { "name": "US dollar", "symbol": "$", "exponent": 2, "symbol_at": "before" }
}))]
pub struct EconomyConfiguration {
    /// The world's one currency, in which every wallet, price and reference value is counted. Required:
    /// a world that enables economy and authors any amount must say what its amounts are.
    currency: Currency,
}

impl EconomyConfiguration {
    /// The world's currency.
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }
}

/// A currency: what it is called, its symbol, how many minor units make one major unit, and which side of
/// an amount the symbol is written on. Display data only: no amount changes with any of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Currency {
    /// Its name, such as "US dollar" or "crown": 1 to 64 bytes of text with no control characters.
    name: CurrencyName,
    /// Its symbol, such as "$" or "cr": 1 to 8 bytes, no control characters, no whitespace at either end.
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
