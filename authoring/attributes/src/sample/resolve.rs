//! The resolution (`MODULE_SPEC.md` §4.1 "Population profiles", "The lock file"; `pr-s30d-sampler.md`
//! §§5.5, 5.6): every person's values with their origins, from the person's file, the lock, the profile's
//! sample, roles and defaults — a pure function of the catalogue, the profile, the lock, the population
//! seed and salt, and each subject's key, tags and section (I-S30-6). Nothing reads a name.
//!
//! ```text
//! 1 fix        authored values, then lock entries (authored wins); never drawn (I-S30-7)
//! 2 order      the profile's attributes in topological order of given, ties by id
//! 3 per X      group by resolved parent values; restricted members drawn first within their
//!              restriction and counted fixed (Q-S30d-1); stratified by largest remainder, or independent;
//!              numeric by quantile; then roles targeting X (F-S30d-4)
//! 4 defaults   an attribute still absent takes its declaration's default
//! 5 covers     adults: every age-derived attribute at least 18 at the epoch (F-S30d-8)
//! ```

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{Tags, WorldTime};
use serde::Serialize;

use super::hash::{fnv1a64, mix, ppm, stream};
use super::micro::{MILLION, ceil_div, micro_text, round_half_away};
use super::normal::z;
use super::remainder::largest_remainder;
use crate::ATTRIBUTES;
use crate::catalogue::Catalogue;
use crate::declaration::{Declaration, Derive, Mutability};
use crate::derive;
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::Kind;
use crate::lock::{Lock, Pinned};
use crate::profile::rules::{categorical, group_weights, matches};
use crate::profile::{Covers, Distribution, Profile, ProfileError, RoleAction, Shape};
use crate::section::{Authored, Restriction};
use crate::traits::Traits;
use crate::value::{AttributeValue, LevelValue, ScaledInt};

/// `2 × z₀.₉₅` in micro-units, rounded: the spread `p95 − p5` is this many standard deviations.
pub const SPREAD: i128 = 3_289_707;

/// Where a resolved value came from: reported by tools, never saved.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Origin {
    /// The person's own file.
    Authored,
    /// A lock entry from a ruling.
    Ruled,
    /// A lock entry.
    Locked,
    /// A profile role.
    Role,
    /// The profile's sample.
    Sampled {
        /// The group key.
        stratum: String,
        /// The member's slot, or 0 for an independent draw.
        draw: u64,
    },
    /// The declaration's default.
    Default,
}

/// One subject, as the seam hands it: key, tags and section — never a name.
#[derive(Debug, Clone, Copy)]
pub struct Member<'a> {
    /// The authoring key.
    pub key: &'a str,
    /// The tags.
    pub tags: &'a Tags,
    /// The decoded `attributes:` section, if any.
    pub authored: Option<&'a Authored>,
}

/// One group's realised counts against its targets, for a categorical attribute.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Realised {
    /// Members in the group.
    pub n: u64,
    /// `n × p(v)`, as a decimal with 6 places.
    pub target: BTreeMap<ValueSlug, String>,
    /// How many hold each value.
    pub count: BTreeMap<ValueSlug, u64>,
}

/// The least and most a numeric draw can return in one stratum, at the attribute's scale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Support {
    /// The least.
    pub min: String,
    /// The most.
    pub max: String,
}

/// The resolution of every member.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Resolution {
    /// Per person key, per attribute, the value and its origin.
    pub people: BTreeMap<String, BTreeMap<QualifiedId, (AttributeValue, Origin)>>,
    /// Per categorical attribute, per group, the realised counts.
    pub realised: BTreeMap<QualifiedId, BTreeMap<String, Realised>>,
    /// Per numeric attribute, per stratum, the support.
    pub support: BTreeMap<QualifiedId, BTreeMap<String, Support>>,
    /// The attributes drawn independently (`stratify: false`).
    pub unstratified: BTreeSet<QualifiedId>,
}

/// The population's words of randomness (§5.3).
#[derive(Debug, Clone, Copy)]
pub struct Words {
    s: u64,
}

impl Words {
    /// From the population seed and salt.
    pub const fn new(seed: &str, salt: u32) -> Self {
        Self {
            s: stream(seed, salt),
        }
    }

    fn attribute(self, id: &QualifiedId) -> u64 {
        mix(self.s, fnv1a64(id.as_str().as_bytes()))
    }

    /// A member's place in X's assignment order.
    pub fn order(self, id: &QualifiedId, key: &str) -> u64 {
        mix(self.attribute(id), fnv1a64(key.as_bytes()))
    }

    /// A value's tie-break within a group.
    pub fn tie(self, id: &QualifiedId, group: &str, value: usize) -> u64 {
        mix(
            mix(self.attribute(id), fnv1a64(group.as_bytes())),
            value as u64,
        )
    }

    /// The n-th independent word for a member and X.
    pub fn draw(self, id: &QualifiedId, key: &str, n: u64) -> u64 {
        mix(self.order(id, key), n)
    }
}

/// The value at `index` of a categorical attribute's domain.
fn value_at(kind: &Kind, values: &[ValueSlug], index: usize) -> Option<AttributeValue> {
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
fn slug_of(value: &AttributeValue) -> Option<ValueSlug> {
    match value {
        AttributeValue::Category(slug) | AttributeValue::Level(LevelValue { slug, .. }) => {
            Some(slug.clone())
        }
        AttributeValue::Flag(flag) => ValueSlug::new(if *flag { "true" } else { "false" }).ok(),
        _ => None,
    }
}

/// The index of a categorical value in its domain.
fn index_of(values: &[ValueSlug], value: &AttributeValue) -> Option<usize> {
    let slug = slug_of(value)?;
    values.iter().position(|known| *known == slug)
}

/// A person's values so far, as the plain traits derived values are computed from.
fn traits_of(
    catalogue: &Catalogue,
    values: &BTreeMap<QualifiedId, (AttributeValue, Origin)>,
) -> Traits {
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

/// A member's value of `id`: resolved, or — for a derived attribute — derived from what is resolved.
fn current(
    catalogue: &Catalogue,
    values: &BTreeMap<QualifiedId, (AttributeValue, Origin)>,
    id: &QualifiedId,
) -> Option<AttributeValue> {
    if let Some((value, _)) = values.get(id) {
        return Some(value.clone());
    }
    let declaration = catalogue.get(id)?;
    if declaration.mutability != Mutability::Derived {
        return None;
    }
    derive::value(
        catalogue,
        declaration,
        &traits_of(catalogue, values),
        WorldTime::EPOCH,
    )
}

/// The resolution.
///
/// # Errors
///
/// `Unsatisfiable`, `EmptySupport` or `NotCovered`, naming the person and the attribute.
pub fn resolve(
    catalogue: &Catalogue,
    profile: Option<&Profile>,
    lock: Option<&Lock>,
    words: Words,
    members: &[Member<'_>],
) -> Result<Resolution, ProfileError> {
    let mut resolution = Resolution::default();
    if let Some(lock) = lock
        && let Some(stranger) = lock
            .people
            .keys()
            .find(|key| !members.iter().any(|member| member.key == key.as_str()))
    {
        return Err(ProfileError {
            name: "UnknownPerson",
            at: None,
            detail: format!("the lock pins '{stranger}', whom the world does not declare"),
        });
    }
    let mut values: Vec<BTreeMap<QualifiedId, (AttributeValue, Origin)>> =
        Vec::with_capacity(members.len());
    let mut pinned: Vec<BTreeSet<QualifiedId>> = Vec::with_capacity(members.len());
    for member in members {
        let mut own = BTreeMap::new();
        if let Some(entries) = lock.and_then(|lock| lock.people.get(member.key)) {
            for (id, entry) in entries {
                let origin = match entry.origin {
                    Pinned::Ruled => Origin::Ruled,
                    Pinned::Locked => Origin::Locked,
                };
                own.insert(id.clone(), (entry.value.clone(), origin));
            }
        }
        if let Some(authored) = member.authored {
            for (id, value) in &authored.values {
                own.insert(id.clone(), (value.clone(), Origin::Authored));
            }
        }
        pinned.push(own.keys().cloned().collect());
        values.push(own);
    }
    if let Some(profile) = profile {
        for id in &profile.order {
            let distribution = &profile.distributions[id];
            let declaration = catalogue.get(id).ok_or_else(|| missing(id))?;
            if !distribution.stratify {
                resolution.unstratified.insert(id.clone());
            }
            sample(
                catalogue,
                declaration,
                distribution,
                words,
                members,
                &mut values,
                &mut resolution,
            )?;
            apply_roles(catalogue, profile, id, members, &mut values, &pinned);
        }
        // Roles whose target the profile does not draw apply after every drawn attribute.
        let unprofiled: BTreeSet<QualifiedId> = profile
            .roles
            .iter()
            .map(|role| role.target.clone())
            .filter(|target| !profile.distributions.contains_key(target))
            .collect();
        for id in &unprofiled {
            apply_roles(catalogue, profile, id, members, &mut values, &pinned);
        }
    }
    for own in &mut values {
        for declaration in catalogue.attributes() {
            if declaration.mutability == Mutability::Derived || own.contains_key(&declaration.id) {
                continue;
            }
            if let Some(default) = &declaration.default {
                own.insert(declaration.id.clone(), (default.clone(), Origin::Default));
            }
        }
        own.retain(|id, _| {
            catalogue
                .get(id)
                .is_some_and(|declaration| declaration.mutability != Mutability::Derived)
        });
    }
    if profile.is_some_and(|profile| profile.covers == Covers::Adults) {
        for (member, own) in members.iter().zip(&values) {
            let traits = traits_of(catalogue, own);
            for declaration in catalogue.attributes() {
                if !matches!(declaration.derive, Some(Derive::Age { .. })) {
                    continue;
                }
                if let Some(AttributeValue::Quantity(age)) =
                    derive::value(catalogue, declaration, &traits, WorldTime::EPOCH)
                    && age.value() < 18
                {
                    return Err(ProfileError {
                        name: "NotCovered",
                        at: None,
                        detail: format!(
                            "'{}' is {} by '{}', and the profile covers adults",
                            member.key,
                            age.value(),
                            declaration.id
                        ),
                    });
                }
            }
        }
    }
    for (member, own) in members.iter().zip(values) {
        resolution.people.insert(member.key.to_owned(), own);
    }
    Ok(resolution)
}

fn missing(id: &QualifiedId) -> ProfileError {
    ProfileError {
        name: "UnknownAttribute",
        at: None,
        detail: format!("'{id}' is declared by no catalogue"),
    }
}

/// One attribute X, for every member: grouped, drawn, recorded.
fn sample(
    catalogue: &Catalogue,
    declaration: &Declaration,
    distribution: &Distribution,
    words: Words,
    members: &[Member<'_>],
    values: &mut [BTreeMap<QualifiedId, (AttributeValue, Origin)>],
    resolution: &mut Resolution,
) -> Result<(), ProfileError> {
    let id = &declaration.id;
    let mut groups: BTreeMap<String, (Vec<ValueSlug>, Vec<usize>)> = BTreeMap::new();
    for (index, own) in values.iter().enumerate() {
        let parents: Option<Vec<ValueSlug>> = distribution
            .given
            .iter()
            .map(|parent| current(catalogue, own, parent).and_then(|value| slug_of(&value)))
            .collect();
        let Some(parents) = parents else {
            continue;
        };
        let key = distribution
            .given
            .iter()
            .zip(&parents)
            .map(|(parent, value)| format!("{parent}={value}"))
            .collect::<Vec<_>>()
            .join(";");
        groups
            .entry(key)
            .or_insert_with(|| (parents, Vec::new()))
            .1
            .push(index);
    }
    for (key, (parents, indices)) in groups {
        match &distribution.shape {
            Shape::Prior { .. } => categorical_group(
                catalogue,
                declaration,
                distribution,
                words,
                members,
                values,
                resolution,
                &key,
                &parents,
                &indices,
            )?,
            _ => numeric_group(
                catalogue,
                declaration,
                distribution,
                words,
                members,
                values,
                resolution,
                &key,
                &parents,
                &indices,
            )?,
        }
    }
    let _ = id;
    Ok(())
}

fn unsatisfiable(key: &str, id: &QualifiedId) -> ProfileError {
    ProfileError {
        name: "Unsatisfiable",
        at: None,
        detail: format!("no value of '{id}' respects '{key}''s restriction"),
    }
}

#[allow(clippy::too_many_arguments)]
fn categorical_group(
    catalogue: &Catalogue,
    declaration: &Declaration,
    distribution: &Distribution,
    words: Words,
    members: &[Member<'_>],
    values: &mut [BTreeMap<QualifiedId, (AttributeValue, Origin)>],
    resolution: &mut Resolution,
    group: &str,
    parents: &[ValueSlug],
    indices: &[usize],
) -> Result<(), ProfileError> {
    let id = &declaration.id;
    let domain = categorical(&declaration.kind).unwrap_or_default();
    let weights = group_weights(distribution, parents).unwrap_or_default();
    if weights.iter().all(|weight| *weight == 0) {
        return Err(ProfileError {
            name: "EmptySupport",
            at: None,
            detail: format!("'{id}' has no value for {group}"),
        });
    }
    let ppm_of = largest_remainder(1_000_000, &weights, |_| 0);
    let n = indices.len();
    let mut fixed = vec![0_u128; domain.len()];
    let mut unfixed = Vec::new();
    let mut restricted = Vec::new();
    for index in indices {
        match current(catalogue, &values[*index], id) {
            Some(value) => {
                if let Some(at) = index_of(&domain, &value) {
                    fixed[at] += 1;
                }
            }
            None => {
                let restriction = members[*index]
                    .authored
                    .and_then(|authored| authored.restrictions.get(id));
                match restriction {
                    Some(Restriction::Allowed(allowed)) => {
                        restricted.push((*index, allowed.clone()))
                    }
                    _ => unfixed.push(*index),
                }
            }
        }
    }
    restricted.sort_by_key(|(index, _)| words.order(id, members[*index].key));
    for (index, allowed) in restricted {
        let within: Vec<u128> = domain
            .iter()
            .zip(&ppm_of)
            .map(|(value, weight)| if allowed.contains(value) { *weight } else { 0 })
            .collect();
        let total: u128 = within.iter().sum();
        if total == 0 {
            return Err(unsatisfiable(members[index].key, id));
        }
        let point = u128::from(ppm(words.draw(id, members[index].key, 0))) * total / 1_000_000;
        let mut cumulative = 0;
        let chosen = within
            .iter()
            .position(|weight| {
                cumulative += weight;
                cumulative > point
            })
            .unwrap_or(0);
        fixed[chosen] += 1;
        let value = value_at(&declaration.kind, &domain, chosen).ok_or_else(|| missing(id))?;
        values[index].insert(
            id.clone(),
            (
                value,
                Origin::Sampled {
                    stratum: group.to_owned(),
                    draw: 0,
                },
            ),
        );
    }
    unfixed.sort_by_key(|index| words.order(id, members[*index].key));
    let u = unfixed.len() as u128;
    if distribution.stratify {
        let n128 = n as u128;
        let mut quota: Vec<u128> = ppm_of
            .iter()
            .zip(&fixed)
            .map(|(share, held)| (n128 * share).saturating_sub(held * 1_000_000))
            .collect();
        if quota.iter().sum::<u128>() == 0 {
            quota.clone_from(&ppm_of);
        }
        let counts = largest_remainder(u, &quota, |value| words.tie(id, group, value));
        let slots: Vec<usize> = counts
            .iter()
            .enumerate()
            .flat_map(|(value, count)| {
                std::iter::repeat_n(value, usize::try_from(*count).unwrap_or(0))
            })
            .collect();
        for (slot, (index, value)) in unfixed.iter().zip(slots).enumerate() {
            let drawn = value_at(&declaration.kind, &domain, value).ok_or_else(|| missing(id))?;
            values[*index].insert(
                id.clone(),
                (
                    drawn,
                    Origin::Sampled {
                        stratum: group.to_owned(),
                        draw: slot as u64,
                    },
                ),
            );
        }
    } else {
        for index in &unfixed {
            let point = u128::from(ppm(words.draw(id, members[*index].key, 0)));
            let mut cumulative = 0;
            let chosen = ppm_of
                .iter()
                .position(|weight| {
                    cumulative += weight;
                    cumulative > point
                })
                .unwrap_or(0);
            let drawn = value_at(&declaration.kind, &domain, chosen).ok_or_else(|| missing(id))?;
            values[*index].insert(
                id.clone(),
                (
                    drawn,
                    Origin::Sampled {
                        stratum: group.to_owned(),
                        draw: 0,
                    },
                ),
            );
        }
    }
    let mut count: BTreeMap<ValueSlug, u64> =
        domain.iter().map(|value| (value.clone(), 0)).collect();
    for index in indices {
        if let Some(slug) = current(catalogue, &values[*index], id)
            .and_then(|value| index_of(&domain, &value))
            .map(|at| domain[at].clone())
        {
            *count.entry(slug).or_insert(0) += 1;
        }
    }
    let target = domain
        .iter()
        .zip(&ppm_of)
        .map(|(value, share)| {
            (
                value.clone(),
                micro_text(i64::try_from(n as u128 * share).unwrap_or(i64::MAX)),
            )
        })
        .collect();
    resolution.realised.entry(id.clone()).or_default().insert(
        group.to_owned(),
        Realised {
            n: n as u64,
            target,
            count,
        },
    );
    let _ = MILLION;
    Ok(())
}

/// A numeric value at the rational quantile `a / b`, and the stratum's support.
fn numeric_value(
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

#[allow(clippy::too_many_arguments)]
fn numeric_group(
    catalogue: &Catalogue,
    declaration: &Declaration,
    distribution: &Distribution,
    words: Words,
    members: &[Member<'_>],
    values: &mut [BTreeMap<QualifiedId, (AttributeValue, Origin)>],
    resolution: &mut Resolution,
    group: &str,
    parents: &[ValueSlug],
    indices: &[usize],
) -> Result<(), ProfileError> {
    let id = &declaration.id;
    let mut unfixed: Vec<usize> = indices
        .iter()
        .copied()
        .filter(|index| current(catalogue, &values[*index], id).is_none())
        .collect();
    unfixed.sort_by_key(|index| words.order(id, members[*index].key));
    let u = unfixed.len() as u128;
    let Some((_, support)) = numeric_value(declaration, distribution, parents, catalogue, 1, 2)
    else {
        return Err(ProfileError {
            name: "MissingCondition",
            at: None,
            detail: format!("'{id}' has no row for {group}"),
        });
    };
    let scale = match &declaration.kind {
        Kind::Quantity(quantity) => quantity.scale,
        _ => 0,
    };
    resolution.support.entry(id.clone()).or_default().insert(
        group.to_owned(),
        Support {
            min: ScaledInt::new(support.0, scale).to_string(),
            max: ScaledInt::new(support.1, scale).to_string(),
        },
    );
    for (slot, index) in unfixed.iter().enumerate() {
        let key = members[*index].key;
        let (a, b) = if distribution.stratify {
            (2 * slot as u128 + 1, 2 * u)
        } else {
            (2 * u128::from(ppm(words.draw(id, key, 0))) + 1, 2_000_000)
        };
        let restriction = members[*index]
            .authored
            .and_then(|authored| authored.restrictions.get(id));
        let (value, draw) = match restriction {
            Some(Restriction::Between(low, high)) => {
                let (low, high) = (low.value().max(support.0), high.value().min(support.1));
                if low > high {
                    return Err(unsatisfiable(key, id));
                }
                let span = u128::try_from(high - low + 1).unwrap_or(1);
                let offset = u128::from(ppm(words.draw(id, key, 0))) * span / 1_000_000;
                let value = low + i64::try_from(offset).unwrap_or(0);
                (
                    AttributeValue::Quantity(ScaledInt::new(value.min(high), scale)),
                    0,
                )
            }
            _ => {
                let (value, _) = numeric_value(declaration, distribution, parents, catalogue, a, b)
                    .ok_or_else(|| missing(id))?;
                (
                    value,
                    if distribution.stratify {
                        slot as u64
                    } else {
                        0
                    },
                )
            }
        };
        values[*index].insert(
            id.clone(),
            (
                value,
                Origin::Sampled {
                    stratum: group.to_owned(),
                    draw,
                },
            ),
        );
    }
    Ok(())
}

/// Every role targeting `id`, for members whose `id` is neither authored nor locked and whose tags and
/// earlier values match (F-S30d-4): `set` replaces, `add` unions into a set.
fn apply_roles(
    catalogue: &Catalogue,
    profile: &Profile,
    id: &QualifiedId,
    members: &[Member<'_>],
    values: &mut [BTreeMap<QualifiedId, (AttributeValue, Origin)>],
    pinned: &[BTreeSet<QualifiedId>],
) {
    for role in profile.roles.iter().filter(|role| role.target == *id) {
        for (index, member) in members.iter().enumerate() {
            if pinned[index].contains(id) || !role.tags.iter().all(|tag| member.tags.contains(tag))
            {
                continue;
            }
            let parents: Vec<QualifiedId> = role.when.keys().cloned().collect();
            let parent_values: Vec<ValueSlug> = parents
                .iter()
                .filter_map(|parent| {
                    current(catalogue, &values[index], parent).and_then(|value| slug_of(&value))
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
