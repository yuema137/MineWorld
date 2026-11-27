//! The helpers genesis (`resolve`) and an arrival (`newcomer`) share (`pr-s30j-runtime-sampling.md` F-S30j-8):
//! a categorical value by index, a value's slug, a member's current value, a numeric value at a rational
//! quantile, and the roles. Moved from `resolve.rs` without a change of behaviour; `at` is the instant a
//! derived value is computed at — `WorldTime::EPOCH` at genesis, the instant of arrival for a newcomer.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{Tags, WorldTime};

use super::micro::{MILLION, ceil_div, round_half_away};
use super::normal::z;
use super::resolve::{Origin, SPREAD};
use crate::ATTRIBUTES;
use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Mutability};
use crate::derive;
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::Kind;
use crate::profile::rules::matches;
use crate::profile::{Distribution, Profile, RoleAction, Shape};
use crate::traits::Traits;
use crate::value::{AttributeValue, LevelValue, ScaledInt};

/// One person's values so far, with their origins.
pub(crate) type Values = BTreeMap<QualifiedId, (AttributeValue, Origin)>;

/// The value at `index` of a categorical attribute's domain.
pub(crate) fn value_at(kind: &Kind, values: &[ValueSlug], index: usize) -> Option<AttributeValue> {
    let slug = values.get(index)?.clone();
    Some(match kind {
        Kind::Category { .. } => AttributeValue::Category(slug),
        Kind::Level { .. } => AttributeValue::Level(LevelValue {
            slug,
            rank: u16::try_from(index).ok()?,
        }),
        Kind::Flag => AttributeValue::Flag(index == 1),
        Kind::Set { .. } => AttributeValue::Set(BTreeSet::from([slug])),
        Kind::Quantity(_) | Kind::Text { .. } | Kind::Instant => return None,
    })
}

/// A categorical value's slug, as a parent's value or a group key.
pub(crate) fn slug_of(value: &AttributeValue) -> Option<ValueSlug> {
    match value {
        AttributeValue::Category(slug) | AttributeValue::Level(LevelValue { slug, .. }) => {
            Some(slug.clone())
        }
        AttributeValue::Flag(flag) => ValueSlug::new(if *flag { "true" } else { "false" }).ok(),
        _ => None,
    }
}

/// The index of a categorical value in its domain.
pub(crate) fn index_of(values: &[ValueSlug], value: &AttributeValue) -> Option<usize> {
    let slug = slug_of(value)?;
    values.iter().position(|known| *known == slug)
}

/// A person's values so far, as the plain traits derived values are computed from.
pub(crate) fn traits_of(catalogue: &Catalogue, values: &Values) -> Traits {
    let plain = values.iter().filter(|(id, _)| {
        catalogue.get(id).is_some_and(|declaration| {
            declaration.owner == ATTRIBUTES && declaration.mutability != Mutability::Derived
        })
    });
    Traits::stated(
        catalogue,
        plain.map(|(id, (value, _))| (id.clone(), value.clone())),
    )
    .unwrap_or_default()
}

/// A member's value of `id` at `at`: resolved, or — for a derived attribute — derived from what is
/// resolved.
pub(crate) fn current(
    catalogue: &Catalogue,
    values: &Values,
    id: &QualifiedId,
    at: WorldTime,
) -> Option<AttributeValue> {
    if let Some((value, _)) = values.get(id) {
        return Some(value.clone());
    }
    let declaration = catalogue.get(id)?;
    if declaration.mutability != Mutability::Derived {
        return None;
    }
    derive::value(catalogue, declaration, &traits_of(catalogue, values), at)
}

/// A stored person's value of `id` at `at`: stored, or derived from the stored traits.
pub(crate) fn stored(
    catalogue: &Catalogue,
    traits: &Traits,
    id: &QualifiedId,
    at: WorldTime,
) -> Option<AttributeValue> {
    if let Some(value) = traits.value(id) {
        return Some(value.clone());
    }
    let declaration = catalogue.get(id)?;
    if declaration.mutability != Mutability::Derived {
        return None;
    }
    derive::value(catalogue, declaration, traits, at)
}

/// A numeric value at the rational quantile `a / b`, and the stratum's support. An instant drawn by
/// `uniform_age_within_band` is a birth relative to `WorldTime::EPOCH`.
pub(crate) fn numeric_value(
    declaration: &Declaration,
    distribution: &Distribution,
    parents: &[ValueSlug],
    catalogue: &Catalogue,
    a: u128,
    b: u128,
) -> Option<(AttributeValue, (i64, i64))> {
    let clamp_to = |value: i128| -> i64 {
        let (min, max) = match &declaration.kind {
            Kind::Quantity(quantity) => (quantity.min, quantity.max),
            _ => (i64::MIN, i64::MAX),
        };
        i64::try_from(value).unwrap_or(0).clamp(min, max)
    };
    let scale = match &declaration.kind {
        Kind::Quantity(quantity) => quantity.scale,
        _ => 0,
    };
    let quantity = |value: i64| AttributeValue::Quantity(ScaledInt::new(value, scale));
    match &distribution.shape {
        Shape::Normal { rows, clip_sd } => {
            let params = rows.row(parents)?;
            let spread = i128::from(params.p95 - params.p5);
            let value = i128::from(params.mean)
                + round_half_away(i128::from(z(a, b, *clip_sd)) * spread, SPREAD);
            let reach =
                round_half_away(i128::from(*clip_sd) * i128::from(MILLION) * spread, SPREAD);
            let support = (
                clamp_to(i128::from(params.mean) - reach),
                clamp_to(i128::from(params.mean) + reach),
            );
            Some((quantity(clamp_to(value)), support))
        }
        Shape::Uniform { min, max } => {
            let span = u128::try_from(max - min + 1).ok()?;
            let value =
                (i128::from(*min) + i128::try_from(a * span / b).ok()?).min(i128::from(*max));
            Some((quantity(clamp_to(value)), (*min, *max)))
        }
        Shape::Table { points } => {
            let q = i128::try_from(a * 1_000_000 / b).ok()?;
            let (first, last) = (points.first()?, points.last()?);
            let value = if q <= i128::from(first.0) {
                i128::from(first.1)
            } else if q >= i128::from(last.0) {
                i128::from(last.1)
            } else {
                let at = points.windows(2).find(|pair| q < i128::from(pair[1].0))?;
                let ((q0, v0), (q1, v1)) = (at[0], at[1]);
                i128::from(v0)
                    + round_half_away(
                        i128::from(v1 - v0) * (q - i128::from(q0)),
                        i128::from(q1 - q0),
                    )
            };
            Some((quantity(clamp_to(value)), (first.1, last.1)))
        }
        Shape::AgeWithinBand { bands } => {
            let band = distribution
                .given
                .first()
                .and_then(|band| catalogue.get(band))?;
            let values = band.kind.values()?;
            let index = values.iter().position(|value| *value == parents[0])?;
            let (low, high) = *bands.get(index)?;
            let days = |years: i64| ceil_div(i128::from(years) * 146_097, 400);
            let (d_low, d_high) = (days(low), days(high));
            let span = u128::try_from(d_high - d_low).ok()?;
            let age = d_low + i128::try_from(a * span / b).ok()?;
            let birth = i64::try_from(-age * 86_400).ok()?;
            Some((
                AttributeValue::Instant(WorldTime::from_seconds(birth)),
                (low, high),
            ))
        }
        Shape::Prior { .. } => None,
    }
}

/// Every role targeting `id`, for members whose `id` is neither authored nor locked and whose tags and
/// earlier values (at `at`) match (F-S30d-4): `set` replaces, `add` unions into a set.
pub(crate) fn apply_roles(
    catalogue: &Catalogue,
    profile: &Profile,
    id: &QualifiedId,
    members: &[&Tags],
    values: &mut [Values],
    pinned: &[BTreeSet<QualifiedId>],
    at: WorldTime,
) {
    for role in profile.roles.iter().filter(|role| role.target == *id) {
        for (index, tags) in members.iter().enumerate() {
            if pinned[index].contains(id) || !role.tags.iter().all(|tag| tags.contains(tag)) {
                continue;
            }
            let parents: Vec<QualifiedId> = role.when.keys().cloned().collect();
            let parent_values: Vec<ValueSlug> = parents
                .iter()
                .filter_map(|parent| {
                    current(catalogue, &values[index], parent, at).and_then(|value| slug_of(&value))
                })
                .collect();
            if parent_values.len() != parents.len()
                || !matches(&role.when, &parents, &parent_values)
            {
                continue;
            }
            let next = match (&role.action, values[index].get(id)) {
                (RoleAction::Set(value), _) => value.clone(),
                (RoleAction::Add(added), Some((AttributeValue::Set(had), _))) => {
                    AttributeValue::Set(had.union(added).cloned().collect())
                }
                (RoleAction::Add(added), _) => AttributeValue::Set(added.clone()),
            };
            values[index].insert(id.clone(), (next, Origin::Role));
        }
    }
}
