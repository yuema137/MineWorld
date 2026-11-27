//! One newcomer drawn against the people already there (`pr-s30j-runtime-sampling.md` §5.4; `DECISIONS.md`
//! `ARC-128`): a pure, integer function of the catalogue, the profile, the population's words, the
//! newcomer's key, tags and fixed set, the residents' stored traits, and the instant of arrival.
//!
//! ```text
//! 1 fixed      authored, ruled and locked values are kept, never drawn (I-S30-7)
//! 2 group      the newcomer's parent values at `at`; the residents with the same values form the group
//! 3 category   stratified plain trait: deficit(v) = max(0, (n + 1)·w(v) − 10⁶·c(v)), drawn by word 0
//! 4 numeric    stratified plain trait: ten decile bins under the same rule, then a quantile in the bin
//!              by word 1 (D-j-3)
//! 5 otherwise  stratify: false, or an owner's attribute: one independent draw by word 0 (D-j-4)
//! 6 birth      uniform_age_within_band gives an age; the birth is `at − age` (D-j-5)
//! 7 roles      right after their target, from the newcomer's tags and earlier values
//! 8 defaults   what is still absent; derived values are never stored
//! 9 covers     adults: every age-derived attribute at least 18 at `at`
//! ```
//!
//! The words are the genesis words keyed by the person's key (R-S30j-d-1); the admission's cause is named in
//! a refusal and never mixed into a word (D-j-7).

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{EventId, Tags, WorldTime};
use serde::{Deserialize, Serialize};

use super::hash::ppm;
use super::remainder::largest_remainder;
use super::resolve::{Origin, Words, missing, unsatisfiable};
use super::values::{
    Values, apply_roles, current, index_of, numeric_value, slug_of, stored, traits_of, value_at,
};
use crate::ATTRIBUTES;
use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Derive, Mutability};
use crate::derive;
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::Kind;
use crate::lock::{LockEntry, Pinned};
use crate::profile::rules::{categorical, group_weights};
use crate::profile::{Covers, Distribution, Profile, ProfileError, Shape};
use crate::section::{Authored, Restriction};
use crate::traits::Traits;
use crate::value::{AttributeValue, ScaledInt};

/// One million: a probability of 1 in parts per million.
const PPM: u128 = 1_000_000;

/// A decile bin's share in parts per million.
const BIN: u128 = 100_000;

/// How a newcomer's value was fixed before arrival.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixedBy {
    /// The person's own file.
    Authored,
    /// A lock entry from a ruling.
    Ruled,
    /// A lock entry.
    Locked,
}

impl FixedBy {
    /// The origin a tool reports for it.
    pub const fn origin(self) -> Origin {
        match self {
            Self::Authored => Origin::Authored,
            Self::Ruled => Origin::Ruled,
            Self::Locked => Origin::Locked,
        }
    }
}

/// A deferred person's fixed values and restrictions: what genesis would have kept, kept for arrival.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixed {
    /// Per attribute, the value and how it was fixed.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub values: BTreeMap<QualifiedId, (AttributeValue, FixedBy)>,
    /// Per attribute, the restriction the draw respects.
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        with = "restrictions"
    )]
    pub restrictions: BTreeMap<QualifiedId, Restriction>,
}

impl Fixed {
    /// The lock's entries, then the person's authored values (authored wins, as genesis), and the
    /// authored restrictions.
    pub fn of(
        authored: Option<&Authored>,
        locked: Option<&BTreeMap<QualifiedId, LockEntry>>,
    ) -> Self {
        let mut fixed = Self::default();
        for (id, entry) in locked.into_iter().flatten() {
            let by = match entry.origin {
                Pinned::Ruled => FixedBy::Ruled,
                Pinned::Locked => FixedBy::Locked,
            };
            fixed.values.insert(id.clone(), (entry.value.clone(), by));
        }
        if let Some(authored) = authored {
            for (id, value) in &authored.values {
                fixed
                    .values
                    .insert(id.clone(), (value.clone(), FixedBy::Authored));
            }
            fixed.restrictions.clone_from(&authored.restrictions);
        }
        fixed
    }
}

/// `Restriction` as stored: `{ allowed: [...] }` or `{ between: [low, high] }`, the bounds as quantities.
mod restrictions {
    use std::collections::{BTreeMap, BTreeSet};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::id::{QualifiedId, ValueSlug};
    use crate::section::Restriction;
    use crate::value::AttributeValue;

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "lowercase", deny_unknown_fields)]
    enum Stored {
        Allowed(BTreeSet<ValueSlug>),
        Between(AttributeValue, AttributeValue),
    }

    pub(super) fn serialize<S: Serializer>(
        restrictions: &BTreeMap<QualifiedId, Restriction>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let stored: BTreeMap<&QualifiedId, Stored> = restrictions
            .iter()
            .map(|(id, restriction)| {
                let stored = match restriction {
                    Restriction::Allowed(allowed) => Stored::Allowed(allowed.clone()),
                    Restriction::Between(low, high) => Stored::Between(
                        AttributeValue::Quantity(*low),
                        AttributeValue::Quantity(*high),
                    ),
                };
                (id, stored)
            })
            .collect();
        stored.serialize(serializer)
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<QualifiedId, Restriction>, D::Error> {
        let stored = BTreeMap::<QualifiedId, Stored>::deserialize(deserializer)?;
        stored
            .into_iter()
            .map(|(id, stored)| {
                let restriction = match stored {
                    Stored::Allowed(allowed) => Restriction::Allowed(allowed),
                    Stored::Between(
                        AttributeValue::Quantity(low),
                        AttributeValue::Quantity(high),
                    ) => Restriction::Between(low, high),
                    Stored::Between(..) => {
                        return Err(serde::de::Error::custom(format!(
                            "'{id}': a between restriction holds two quantities"
                        )));
                    }
                };
                Ok((id, restriction))
            })
            .collect()
    }
}

/// The newcomer, as the hook hands it: key, tags and fixed set — never a name.
#[derive(Debug, Clone, Copy)]
pub struct Newcomer<'a> {
    /// The authoring key.
    pub key: &'a str,
    /// The entity's registry tags.
    pub tags: &'a Tags,
    /// Fixed values and restrictions.
    pub fixed: &'a Fixed,
}

/// One person already in the world: key and stored plain traits.
#[derive(Debug, Clone, Copy)]
pub struct Resident<'a> {
    /// The authoring key.
    pub key: &'a str,
    /// The stored plain traits.
    pub traits: &'a Traits,
}

/// A newcomer's values with their origins (tool output; facts carry values only).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Arrival {
    /// Per attribute, the value and its origin.
    pub values: BTreeMap<QualifiedId, (AttributeValue, Origin)>,
}

/// What every attribute's draw reads.
struct Draw<'a> {
    catalogue: &'a Catalogue,
    words: Words,
    key: &'a str,
    fixed: &'a Fixed,
    population: &'a [Resident<'a>],
    at: WorldTime,
    because: EventId,
}

impl Draw<'_> {
    fn refused(&self, mut error: ProfileError) -> ProfileError {
        error.detail = format!(
            "{} (the arrival of '{}', caused by event {})",
            error.detail, self.key, self.because
        );
        error
    }

    /// The residents whose parent values at `at` are `parents`.
    fn group<'g>(
        &'g self,
        distribution: &'g Distribution,
        parents: &'g [ValueSlug],
    ) -> impl Iterator<Item = &'g Resident<'g>> + 'g {
        self.population.iter().filter(move |resident| {
            distribution
                .given
                .iter()
                .zip(parents)
                .all(|(parent, wanted)| {
                    stored(self.catalogue, resident.traits, parent, self.at)
                        .and_then(|value| slug_of(&value))
                        .is_some_and(|value| value == *wanted)
                })
        })
    }

    /// `weights` with every value outside `allowed` zeroed.
    fn within(
        domain: &[ValueSlug],
        weights: &[u128],
        allowed: Option<&BTreeSet<ValueSlug>>,
    ) -> Vec<u128> {
        domain
            .iter()
            .zip(weights)
            .map(|(value, weight)| match allowed {
                Some(allowed) if !allowed.contains(value) => 0,
                _ => *weight,
            })
            .collect()
    }

    /// The index whose cumulative chance first passes `word`'s point.
    fn pick(chances: &[u128], word: u64) -> Option<usize> {
        let total: u128 = chances.iter().sum();
        if total == 0 {
            return None;
        }
        let point = u128::from(ppm(word)) * total / PPM;
        let mut cumulative = 0;
        chances.iter().position(|chance| {
            cumulative += chance;
            cumulative > point
        })
    }

    /// The deficit of each slot: `max(0, (n + 1)·share − 10⁶·held)`.
    fn deficits(shares: &[u128], held: &[u128], n: u128) -> Vec<u128> {
        shares
            .iter()
            .zip(held)
            .map(|(share, held)| ((n + 1) * share).saturating_sub(held * PPM))
            .collect()
    }

    fn categorical(
        &self,
        declaration: &Declaration,
        distribution: &Distribution,
        parents: &[ValueSlug],
        group: &str,
    ) -> Result<(AttributeValue, Origin), ProfileError> {
        let id = &declaration.id;
        let domain = categorical(&declaration.kind).unwrap_or_default();
        let weights = group_weights(distribution, parents).unwrap_or_default();
        if weights.iter().all(|weight| *weight == 0) {
            return Err(self.refused(ProfileError {
                name: "EmptySupport",
                at: None,
                detail: format!("'{id}' has no value for {group}"),
            }));
        }
        let shares = largest_remainder(PPM, &weights, |_| 0);
        let allowed = match self.fixed.restrictions.get(id) {
            Some(Restriction::Allowed(allowed)) => Some(allowed),
            _ => None,
        };
        let mut n = 0_u128;
        let chances = if distribution.stratify && declaration.owner == ATTRIBUTES {
            let mut held = vec![0_u128; domain.len()];
            for resident in self.group(distribution, parents) {
                if let Some(at) = stored(self.catalogue, resident.traits, id, self.at)
                    .and_then(|value| index_of(&domain, &value))
                {
                    held[at] += 1;
                    n += 1;
                }
            }
            Self::within(&domain, &Self::deficits(&shares, &held, n), allowed)
        } else {
            Self::within(&domain, &shares, allowed)
        };
        let word = self.words.draw(id, self.key, 0);
        let chosen = Self::pick(&chances, word)
            .or_else(|| Self::pick(&Self::within(&domain, &shares, allowed), word))
            .ok_or_else(|| self.refused(unsatisfiable(self.key, id)))?;
        let value = value_at(&declaration.kind, &domain, chosen).ok_or_else(|| missing(id))?;
        Ok((
            value,
            Origin::Sampled {
                stratum: group.to_owned(),
                draw: u64::try_from(n).unwrap_or(u64::MAX),
            },
        ))
    }

    /// A numeric value's place on its stratum's line: a quantity's units; an age in seconds for a birth
    /// (`edge`: a birth relative to the epoch, as the shape gives it; otherwise a resident's, at `at`).
    fn position(&self, value: &AttributeValue, edge: bool) -> Option<i128> {
        match value {
            AttributeValue::Quantity(quantity) => Some(i128::from(quantity.value())),
            AttributeValue::Instant(birth) if edge => Some(-i128::from(birth.seconds())),
            AttributeValue::Instant(birth) => {
                Some(i128::from(self.at.seconds()) - i128::from(birth.seconds()))
            }
            _ => None,
        }
    }

    fn numeric(
        &self,
        declaration: &Declaration,
        distribution: &Distribution,
        parents: &[ValueSlug],
        group: &str,
    ) -> Result<(AttributeValue, Origin), ProfileError> {
        let id = &declaration.id;
        let catalogue = self.catalogue;
        let Some((_, support)) = numeric_value(declaration, distribution, parents, catalogue, 1, 2)
        else {
            return Err(self.refused(ProfileError {
                name: "MissingCondition",
                at: None,
                detail: format!("'{id}' has no row for {group}"),
            }));
        };
        let scale = match &declaration.kind {
            Kind::Quantity(quantity) => quantity.scale,
            _ => 0,
        };
        if let Some(Restriction::Between(low, high)) = self.fixed.restrictions.get(id) {
            let (low, high) = (low.value().max(support.0), high.value().min(support.1));
            if low > high {
                return Err(self.refused(unsatisfiable(self.key, id)));
            }
            let span = u128::try_from(high - low + 1).unwrap_or(1);
            let offset = u128::from(ppm(self.words.draw(id, self.key, 0))) * span / PPM;
            let value = low + i64::try_from(offset).unwrap_or(0);
            return Ok((
                AttributeValue::Quantity(ScaledInt::new(value.min(high), scale)),
                Origin::Sampled {
                    stratum: group.to_owned(),
                    draw: 0,
                },
            ));
        }
        let mut n = 0_u128;
        let quantile = if distribution.stratify && declaration.owner == ATTRIBUTES {
            let edges: Vec<i128> = (1..10)
                .filter_map(|k| numeric_value(declaration, distribution, parents, catalogue, k, 10))
                .filter_map(|(value, _)| self.position(&value, true))
                .collect();
            let mut held = vec![0_u128; 10];
            for resident in self.group(distribution, parents) {
                if let Some(at) = stored(catalogue, resident.traits, id, self.at)
                    .and_then(|value| self.position(&value, false))
                {
                    held[edges.iter().filter(|edge| at >= **edge).count()] += 1;
                    n += 1;
                }
            }
            let chances = Self::deficits(&[BIN; 10], &held, n);
            let bin = Self::pick(&chances, self.words.draw(id, self.key, 0))
                .or_else(|| Self::pick(&[BIN; 10], self.words.draw(id, self.key, 0)))
                .unwrap_or(0);
            bin as u128 * BIN + u128::from(ppm(self.words.draw(id, self.key, 1))) / 10
        } else {
            u128::from(ppm(self.words.draw(id, self.key, 0)))
        };
        let (value, _) = numeric_value(
            declaration,
            distribution,
            parents,
            catalogue,
            2 * quantile + 1,
            2 * PPM,
        )
        .ok_or_else(|| missing(id))?;
        let value = match (&distribution.shape, value) {
            // D-j-5: the shape's birth is relative to the epoch; a newcomer's is relative to arrival.
            (Shape::AgeWithinBand { .. }, AttributeValue::Instant(birth)) => {
                AttributeValue::Instant(WorldTime::from_seconds(
                    self.at.seconds().saturating_add(birth.seconds()),
                ))
            }
            (_, value) => value,
        };
        Ok((
            value,
            Origin::Sampled {
                stratum: group.to_owned(),
                draw: u64::try_from(n).unwrap_or(u64::MAX),
            },
        ))
    }
}

/// One newcomer's values, drawn against the people already resolved (`pr-s30j-runtime-sampling.md` §5.4).
///
/// # Errors
///
/// `Unsatisfiable`, `EmptySupport`, `MissingCondition` or `NotCovered`, naming the person, the attribute
/// and the admission's cause.
pub fn sample_newcomer(
    catalogue: &Catalogue,
    profile: Option<&Profile>,
    words: Words,
    newcomer: Newcomer<'_>,
    population: &[Resident<'_>],
    at: WorldTime,
    because: EventId,
) -> Result<Arrival, ProfileError> {
    let draw = Draw {
        catalogue,
        words,
        key: newcomer.key,
        fixed: newcomer.fixed,
        population,
        at,
        because,
    };
    let mut values: Values = newcomer
        .fixed
        .values
        .iter()
        .map(|(id, (value, by))| (id.clone(), (value.clone(), by.origin())))
        .collect();
    let pinned: BTreeSet<QualifiedId> = values.keys().cloned().collect();
    if let Some(profile) = profile {
        for id in &profile.order {
            let distribution = &profile.distributions[id];
            let declaration = catalogue.get(id).ok_or_else(|| missing(id))?;
            let parents: Option<Vec<ValueSlug>> = distribution
                .given
                .iter()
                .map(|parent| {
                    current(catalogue, &values, parent, at).and_then(|value| slug_of(&value))
                })
                .collect();
            if let Some(parents) = parents
                && current(catalogue, &values, id, at).is_none()
            {
                let group = distribution
                    .given
                    .iter()
                    .zip(&parents)
                    .map(|(parent, value)| format!("{parent}={value}"))
                    .collect::<Vec<_>>()
                    .join(";");
                let drawn = match &distribution.shape {
                    Shape::Prior { .. } => {
                        draw.categorical(declaration, distribution, &parents, &group)?
                    }
                    _ => draw.numeric(declaration, distribution, &parents, &group)?,
                };
                values.insert(id.clone(), drawn);
            }
            apply_roles(
                catalogue,
                profile,
                id,
                &[newcomer.tags],
                std::slice::from_mut(&mut values),
                std::slice::from_ref(&pinned),
                at,
            );
        }
        let unprofiled: BTreeSet<QualifiedId> = profile
            .roles
            .iter()
            .map(|role| role.target.clone())
            .filter(|target| !profile.distributions.contains_key(target))
            .collect();
        for id in &unprofiled {
            apply_roles(
                catalogue,
                profile,
                id,
                &[newcomer.tags],
                std::slice::from_mut(&mut values),
                std::slice::from_ref(&pinned),
                at,
            );
        }
    }
    for declaration in catalogue.attributes() {
        if declaration.mutability == Mutability::Derived || values.contains_key(&declaration.id) {
            continue;
        }
        if let Some(default) = &declaration.default {
            values.insert(declaration.id.clone(), (default.clone(), Origin::Default));
        }
    }
    values.retain(|id, _| {
        catalogue
            .get(id)
            .is_some_and(|declaration| declaration.mutability != Mutability::Derived)
    });
    if profile.is_some_and(|profile| profile.covers == Covers::Adults) {
        let traits = traits_of(catalogue, &values);
        for declaration in catalogue.attributes() {
            if !matches!(declaration.derive, Some(Derive::Age { .. })) {
                continue;
            }
            if let Some(AttributeValue::Quantity(age)) =
                derive::value(catalogue, declaration, &traits, at)
                && age.value() < 18
            {
                return Err(draw.refused(ProfileError {
                    name: "NotCovered",
                    at: None,
                    detail: format!(
                        "'{}' is {} by '{}' at arrival, and the profile covers adults",
                        newcomer.key,
                        age.value(),
                        declaration.id
                    ),
                }));
            }
        }
    }
    Ok(Arrival { values })
}
