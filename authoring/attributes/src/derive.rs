//! Derived values (`MODULE_SPEC.md` §4.1 "Derived attributes"; F-S30c-8): computed at a read and at
//! disclosure from stored traits and the instant, never stored and never stated.
//!
//! ```text
//! age    whole years since an instant, by the mean Gregorian year (146 097 days per 400 years):
//!        floor((at − birth) × 400 / (146 097 × 86 400)) — integers only; a birthday-exact age needs a
//!        calendar and is not MVP-0's
//! band   the level value of the first threshold the source is below, else the last
//! ```

use mineworld_contracts::WorldTime;

use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Derive, Mutability, Threshold};
use crate::kind::Kind;
use crate::traits::Traits;
use crate::value::{AttributeValue, LevelValue, ScaledInt};

/// Seconds in 400 mean Gregorian years.
const FOUR_HUNDRED_YEARS: i128 = 146_097 * 86_400;

/// Whole years from `birth` to `at`, floored (negative before `birth`).
pub fn age(birth: WorldTime, at: WorldTime) -> i64 {
    let since = i128::from(at.seconds()) - i128::from(birth.seconds());
    let years = (since * 400).div_euclid(FOUR_HUNDRED_YEARS);
    i64::try_from(years).unwrap_or(if years < 0 { i64::MIN } else { i64::MAX })
}

/// The index of the band `value` falls in: the first threshold it is below, else the last band.
pub fn band_index(value: &AttributeValue, below: &[Threshold]) -> Option<usize> {
    let below_it = |threshold: &Threshold| match (value, threshold) {
        (AttributeValue::Quantity(value), Threshold::Quantity(bound)) => {
            Some(rescaled(*value, bound.scale()) < bound.value())
        }
        (AttributeValue::Level(value), Threshold::Level(bound)) => Some(value.rank < bound.rank),
        _ => None,
    };
    for (index, threshold) in below.iter().enumerate() {
        if below_it(threshold)? {
            return Some(index);
        }
    }
    Some(below.len())
}

/// `value` at `scale`, by integer arithmetic; saturating.
fn rescaled(value: ScaledInt, scale: u8) -> i64 {
    let (from, to) = (u32::from(value.scale()), u32::from(scale));
    if to >= from {
        value
            .value()
            .saturating_mul(10_i64.saturating_pow(to - from))
    } else {
        value.value().div_euclid(10_i64.pow(from - to))
    }
}

/// `declaration`'s derived value for a person whose stored traits are `traits`, at `at`; `None` when a
/// source is absent, or for a projection (the owner's).
pub fn value(
    catalogue: &Catalogue,
    declaration: &Declaration,
    traits: &Traits,
    at: WorldTime,
) -> Option<AttributeValue> {
    value_bounded(catalogue, declaration, traits, at, 8)
}

fn value_bounded(
    catalogue: &Catalogue,
    declaration: &Declaration,
    traits: &Traits,
    at: WorldTime,
    depth: u8,
) -> Option<AttributeValue> {
    let source = |id| {
        let source = catalogue.get(id)?;
        match source.mutability {
            Mutability::Derived if depth > 0 => {
                value_bounded(catalogue, source, traits, at, depth - 1)
            }
            Mutability::Derived => None,
            Mutability::Fixed | Mutability::Evolving => traits.value(id).cloned(),
        }
    };
    match declaration.derive.as_ref()? {
        Derive::Age { from } => {
            let AttributeValue::Instant(birth) = source(from)? else {
                return None;
            };
            Some(AttributeValue::Quantity(ScaledInt::new(age(birth, at), 0)))
        }
        Derive::Band { from, below } => {
            let index = band_index(&source(from)?, below)?;
            let Kind::Level { values } = &declaration.kind else {
                return None;
            };
            let slug = values.get(index)?.clone();
            Some(AttributeValue::Level(LevelValue {
                slug,
                rank: u16::try_from(index).ok()?,
            }))
        }
        Derive::Projection => None,
    }
}

#[cfg(test)]
mod tests;
