//! What a notice carries — the body-kind catalog — and the one check every body passes.
//!
//! `notices` does not know what a price means (step-31 §8.3). A price line is an item and an amount the
//! publisher claims, with `basis` the publisher's own count of the observations behind it.

use mineworld_contracts::{EntityId, EntityType, LifecycleState, Rejection, RejectionCode};
use mineworld_kernel::WorldRead;
use serde::{Deserialize, Serialize};

use crate::topic::BodyKind;

/// The most price lines one notice may carry.
pub const PRICE_LINES_MAX: usize = 32;

/// The longest text body, in bytes.
pub const TEXT_MAX_BYTES: usize = 280;

/// The highest price a line may state: 10^12 minor units, equal to economy's `PRICE_MAX` by design
/// (`ARC-112`, F-S31b-6). `notices` depends on no system, so it states the bound as its own.
pub const NOTICE_PRICE_MAX: u64 = 1_000_000_000_000;

/// One line of a `prices` body: an item kind, an amount in minor units, and how many observations
/// the publisher says back it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceLine {
    item: EntityId,
    price: u64,
    basis: u16,
}

impl PriceLine {
    /// A line.
    pub const fn new(item: EntityId, price: u64, basis: u16) -> Self {
        Self { item, price, basis }
    }

    /// The item kind.
    pub const fn item(&self) -> EntityId {
        self.item
    }

    /// The amount, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// The publisher's count of observations.
    pub const fn basis(&self) -> u16 {
        self.basis
    }
}

/// One line of text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NoticeText(String);

impl NoticeText {
    /// The text, unchecked: [`judge_body`] is the check.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a notice carries. JSON, externally tagged: `{"prices": [{"item": "41", "price": 300,
/// "basis": 6}]}` or `{"text": "The market opens late tomorrow."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeBody {
    /// Price lines, strictly ascending by item, 1 to 32.
    Prices(Vec<PriceLine>),
    /// One line of text, 1 to 280 bytes, no control character.
    Text(NoticeText),
}

impl NoticeBody {
    /// Which kind this body is.
    pub const fn kind(&self) -> BodyKind {
        match self {
            Self::Prices(_) => BodyKind::Prices,
            Self::Text(_) => BodyKind::Text,
        }
    }
}

fn code(code: &'static str, detail: Option<String>) -> Rejection {
    Rejection::System {
        code: RejectionCode::from_static(code),
        detail,
    }
}

fn invalid(detail: String) -> Rejection {
    code("body-invalid", Some(detail))
}

/// Whether `body` may be posted on a topic whose notices carry `kind`, in this order:
///
/// 1. the body's kind is the topic's, else `body-mismatch`;
/// 2. at most 32 price lines or 280 bytes of text, else `body-too-large`;
/// 3. a `prices` body has at least one line; each line names a living Item, a price from 1 to
///    [`NOTICE_PRICE_MAX`] and a `basis` of at least 1; the items strictly ascend — else
///    `body-invalid`, naming the line and the reason. A `text` body is not empty and has no control
///    character (a newline included: a notice is one line) — else `body-invalid`.
///
/// # Errors
///
/// The first rule `body` breaks.
pub fn judge_body(
    world: &WorldRead<'_>,
    kind: BodyKind,
    body: &NoticeBody,
) -> Result<(), Rejection> {
    if body.kind() != kind {
        return Err(code("body-mismatch", None));
    }
    match body {
        NoticeBody::Prices(lines) => {
            if lines.len() > PRICE_LINES_MAX {
                return Err(code("body-too-large", None));
            }
            if lines.is_empty() {
                return Err(invalid("a prices body has no line".to_owned()));
            }
            let mut previous: Option<EntityId> = None;
            for (index, line) in lines.iter().enumerate() {
                let is_item = world.entity(line.item).is_some_and(|record| {
                    record.entity_type() == EntityType::Item
                        && record.lifecycle() != LifecycleState::Destroyed
                });
                if !is_item {
                    return Err(invalid(format!("line {index}: the item is not an item")));
                }
                if line.price == 0 || line.price > NOTICE_PRICE_MAX {
                    return Err(invalid(format!(
                        "line {index}: the price must be 1 to {NOTICE_PRICE_MAX}"
                    )));
                }
                if line.basis == 0 {
                    return Err(invalid(format!(
                        "line {index}: the basis must be at least 1"
                    )));
                }
                if previous.is_some_and(|before| before >= line.item) {
                    return Err(invalid(format!(
                        "line {index}: items must strictly ascend, with none twice"
                    )));
                }
                previous = Some(line.item);
            }
            Ok(())
        }
        NoticeBody::Text(text) => {
            if text.0.len() > TEXT_MAX_BYTES {
                return Err(code("body-too-large", None));
            }
            if text.0.is_empty() {
                return Err(invalid("the text is empty".to_owned()));
            }
            if text.0.chars().any(char::is_control) {
                return Err(invalid(
                    "the text holds a control character; a notice is one line".to_owned(),
                ));
            }
            Ok(())
        }
    }
}
