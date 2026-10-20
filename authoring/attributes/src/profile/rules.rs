//! The static rules of a profile (`MODULE_SPEC.md` §4.1 "Population profiles"; `pr-s30d-sampler.md`
//! §5.4), each refused by its name before anything is sampled (C-6).

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use mineworld_contracts::Tag;

use super::{
    Adjust, Covers, Distribution, Mask, NormalParams, Profile, ProfileFile, RawDistribution,
    RawRole, Role, RoleAction, Rows, Shape, When,
};
use crate::catalogue::Catalogue;
use crate::config::AttributesConfiguration;
use crate::declaration::{Declaration, Derive, Mutability, Threshold};
use crate::id::{QualifiedId, ValueSlug};
use crate::kind::Kind;
use crate::sample::micro::{MILLION, micro};
use crate::sample::remainder::largest_remainder;
use crate::value::{AttributeValue, Loose, ScaledInt, decode};

/// A profile's refusal: its name in `MODULE_SPEC.md` §4.1's table, where, and what.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}{name}: {detail}", .at.as_ref().map(|at| format!("{at}: ")).unwrap_or_default())]
pub struct ProfileError {
    /// The refusal's name (`DependencyCycle`, `BadDistribution` …).
    pub name: &'static str,
    /// The file and line, when known.
    pub at: Option<String>,
    /// What, naming the attribute.
    pub detail: String,
}

fn refused(name: &'static str, at: Option<&str>, detail: impl Into<String>) -> ProfileError {
    ProfileError {
        name,
        at: at.map(str::to_owned),
        detail: detail.into(),
    }
}

/// A distribution's position, for its refusals.
fn position(file: &std::path::Path, line: u64) -> String {
    let file = file.display().to_string().replace('\\', "/");
    if line > 0 {
        format!("{file} line {line}")
    } else {
        file
    }
}

/// The values a categorical attribute draws among: a domain's, or `false`/`true` for a flag.
pub(crate) fn categorical(kind: &Kind) -> Option<Vec<ValueSlug>> {
    match kind {
        Kind::Category { values, .. } | Kind::Level { values } | Kind::Set { values, .. } => {
            Some(values.clone())
        }
        Kind::Flag => Some(vec![slug("false"), slug("true")]),
        Kind::Quantity(_) | Kind::Text { .. } | Kind::Instant => None,
    }
}

fn slug(text: &str) -> ValueSlug {
    ValueSlug::new(text).unwrap_or_else(|_| unreachable!("a constant slug"))
}

/// The world's profile, frozen: `None` when the configuration names none.
///
/// # Errors
///
/// `NoSeed`, `UnknownProfile`, `DuplicateProfile`, and every rule of [`build_one`].
pub fn build(
    catalogue: &Catalogue,
    files: &[(PathBuf, ProfileFile)],
    configuration: Option<&AttributesConfiguration>,
) -> Result<Option<Profile>, ProfileError> {
    let mut seen: BTreeMap<&str, &PathBuf> = BTreeMap::new();
    for (path, file) in files {
        if let Some(first) = seen.insert(&file.profile, path) {
            return Err(refused(
                "DuplicateProfile",
                None,
                format!(
                    "'{}' is declared by {} and {}",
                    file.profile,
                    first.display(),
                    path.display()
                ),
            ));
        }
    }
    let Some(configuration) = configuration else {
        return Ok(None);
    };
    let Some(wanted) = &configuration.profile else {
        return Ok(None);
    };
    if configuration.seed.as_deref().is_none_or(str::is_empty) {
        return Err(refused(
            "NoSeed",
            None,
            format!("configure/attributes.yaml names the profile '{wanted}' and no seed"),
        ));
    }
    let Some((path, file)) = files.iter().find(|(_, file)| file.profile == *wanted) else {
        let known: Vec<&str> = seen.keys().copied().collect();
        return Err(refused(
            "UnknownProfile",
            None,
            format!(
                "'{wanted}' is no profile of this world (known: {})",
                known.join(", ")
            ),
        ));
    };
    build_one(catalogue, path, file).map(Some)
}

/// One profile against the catalogue.
fn build_one(
    catalogue: &Catalogue,
    path: &std::path::Path,
    file: &ProfileFile,
) -> Result<Profile, ProfileError> {
    let mut distributions = BTreeMap::new();
    for (written, spanned) in &file.distributions {
        let at = position(path, spanned.referenced.line());
        let declaration = catalogue
            .resolve(written)
            .map_err(|error| refused("UnknownAttribute", Some(&at), error.to_string()))?;
        let distribution = distribution(catalogue, declaration, &spanned.value, &at, file)?;
        distributions.insert(declaration.id.clone(), distribution);
    }
    let order = order(&distributions)?;
    let mut roles = Vec::new();
    for spanned in &file.roles {
        let at = position(path, spanned.referenced.line());
        roles.extend(role(catalogue, &spanned.value, &order, &at)?);
    }
    let profile = Profile {
        id: file.profile.clone(),
        version: file.version,
        covers: file.covers,
        sources: file.sources.clone(),
        distributions,
        roles,
        order,
    };
    check_conditions(catalogue, &profile)?;
    Ok(profile)
}

/// Every parent named, every rule of one distribution.
fn distribution(
    catalogue: &Catalogue,
    declaration: &Declaration,
    raw: &RawDistribution,
    at: &str,
    file: &ProfileFile,
) -> Result<Distribution, ProfileError> {
    let id = &declaration.id;
    let bad = |detail: String| refused("BadDistribution", Some(at), format!("'{id}': {detail}"));
    if declaration.mutability == Mutability::Derived
        && !matches!(declaration.derive, Some(Derive::Band { .. }))
    {
        return Err(bad(
            "a derived attribute other than a band has no distribution (OutOfDomain)".to_owned(),
        ));
    }
    let source = raw.source.clone().ok_or_else(|| {
        refused(
            "UncitedDistribution",
            Some(at),
            format!("'{id}' cites no source"),
        )
    })?;
    if source != "authored" && !file.sources.contains_key(&source) {
        return Err(refused(
            "UncitedDistribution",
            Some(at),
            format!("'{id}' cites '{source}', which the file does not declare"),
        ));
    }
    let mut given = Vec::new();
    for parent in &raw.given {
        let parent = catalogue
            .resolve(parent)
            .map_err(|error| refused("UnknownAttribute", Some(at), error.to_string()))?;
        if categorical(&parent.kind).is_none() {
            return Err(bad(format!(
                "the parent '{}' is not categorical",
                parent.id
            )));
        }
        given.push(parent.id.clone());
    }
    let parents: Vec<Vec<ValueSlug>> = given
        .iter()
        .map(|parent| {
            categorical(
                &catalogue
                    .get(parent)
                    .map(|d| d.kind.clone())
                    .unwrap_or(Kind::Flag),
            )
            .unwrap_or_default()
        })
        .collect();
    let shapes = [
        raw.prior.is_some(),
        raw.normal.is_some(),
        raw.uniform.is_some(),
        raw.table.is_some(),
        raw.uniform_age_within_band.is_some(),
    ];
    if shapes.iter().filter(|shape| **shape).count() != 1 {
        return Err(bad(
            "exactly one of prior, normal, uniform, table, uniform_age_within_band".to_owned(),
        ));
    }
    let shape = if let Some(prior) = &raw.prior {
        let values = categorical(&declaration.kind)
            .ok_or_else(|| bad(format!("a {} has no prior", declaration.kind.name())))?;
        let rows = rows(
            prior,
            &parents,
            0,
            &|leaf: &Loose| weights(leaf, &values),
            at,
            id,
        )?;
        let adjust = raw
            .adjust
            .iter()
            .map(|entry| {
                adjustment(
                    catalogue,
                    &given,
                    &values,
                    &entry.when,
                    &entry.ratios,
                    at,
                    id,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mask = raw
            .mask
            .iter()
            .map(|entry| masking(catalogue, &given, &values, &entry.when, &entry.only, at, id))
            .collect::<Result<Vec<_>, _>>()?;
        Shape::Prior { rows, adjust, mask }
    } else if let Some(normal) = &raw.normal {
        let Kind::Quantity(quantity) = &declaration.kind else {
            return Err(bad("a normal is for a quantity".to_owned()));
        };
        let scale = quantity.scale;
        let rows = match normal {
            Loose::Map(map) if map.contains_key("by") => rows(
                &map["by"],
                &parents,
                0,
                &|leaf: &Loose| normal_params(leaf, scale),
                at,
                id,
            )?,
            leaf => Rows::Leaf(normal_params(leaf, scale).map_err(&bad)?),
        };
        let clip_sd = raw.clip_sd.unwrap_or(3);
        if !(1..=3).contains(&clip_sd) {
            return Err(bad(format!("clip_sd is 1 … 3, not {clip_sd}")));
        }
        Shape::Normal { rows, clip_sd }
    } else if let Some(uniform) = &raw.uniform {
        let Kind::Quantity(quantity) = &declaration.kind else {
            return Err(bad("a uniform is for a quantity".to_owned()));
        };
        let at_scale = |loose: &Loose| scaled(loose, quantity.scale).map_err(&bad);
        let (min, max) = (at_scale(&uniform.min)?, at_scale(&uniform.max)?);
        if min > max || min < quantity.min || max > quantity.max {
            return Err(refused(
                "MissingSupport",
                Some(at),
                format!("'{id}': uniform outside the declaration's bounds"),
            ));
        }
        Shape::Uniform { min, max }
    } else if let Some(table) = &raw.table {
        let Kind::Quantity(quantity) = &declaration.kind else {
            return Err(bad("a table is for a quantity".to_owned()));
        };
        let mut points = Vec::new();
        for (q, value) in &table.quantiles {
            points.push((
                micro(q).map_err(|error| bad(error.to_string()))?,
                scaled(value, quantity.scale).map_err(&bad)?,
            ));
        }
        points.sort_unstable();
        if points.len() < 2
            || points
                .windows(2)
                .any(|pair| pair[0].0 >= pair[1].0 || pair[0].1 > pair[1].1)
        {
            return Err(refused(
                "MissingSupport",
                Some(at),
                format!("'{id}': a table needs two or more increasing quantiles"),
            ));
        }
        Shape::Table { points }
    } else {
        age_within_band(catalogue, declaration, &given, raw, at)?
    };
    Ok(Distribution {
        given,
        shape,
        stratify: raw.stratify.unwrap_or(true),
        source,
        verify: raw.verify.clone(),
    })
}

/// A decimal at `scale`.
fn scaled(loose: &Loose, scale: u8) -> Result<i64, String> {
    match loose {
        Loose::Number(text) | Loose::Text(text) => ScaledInt::parse(text, scale)
            .map(ScaledInt::value)
            .map_err(|error| error.to_string()),
        other => Err(format!("{} is not a number", other.describe())),
    }
}

fn normal_params(leaf: &Loose, scale: u8) -> Result<NormalParams, String> {
    let Loose::Map(map) = leaf else {
        return Err("a normal row is { mean, p5, p95 }".to_owned());
    };
    let field = |name: &str| {
        map.get(name)
            .ok_or_else(|| format!("a normal row states {name}"))
            .and_then(|loose| scaled(loose, scale))
    };
    let params = NormalParams {
        mean: field("mean")?,
        p5: field("p5")?,
        p95: field("p95")?,
    };
    if params.p5 >= params.p95 || params.mean < params.p5 || params.mean > params.p95 {
        return Err("a normal row has p5 < mean < p95".to_owned());
    }
    Ok(params)
}

/// One prior row: a weight per value, each ≥ 0 with at most 6 decimals, summing to 1 ± 10⁻⁶, then
/// normalised to exactly 10⁶ by largest remainder (ties by lower index).
fn weights(leaf: &Loose, values: &[ValueSlug]) -> Result<Vec<i64>, String> {
    let Loose::Map(map) = leaf else {
        return Err("a prior row maps values to probabilities".to_owned());
    };
    let mut weights = vec![0_i64; values.len()];
    for (value, weight) in map {
        let index = values
            .iter()
            .position(|known| known.as_str() == value)
            .ok_or_else(|| format!("'{value}' is not a value of the attribute"))?;
        let text = match weight {
            Loose::Number(text) | Loose::Text(text) => text,
            other => return Err(format!("{} is not a probability", other.describe())),
        };
        let weight = micro(text).map_err(|error| error.to_string())?;
        if weight < 0 {
            return Err(format!("{value}: a probability is not below 0"));
        }
        weights[index] = weight;
    }
    let sum: i64 = weights.iter().sum();
    if (sum - MILLION).abs() > 1 {
        return Err(format!(
            "the row sums to {}",
            crate::sample::micro::micro_text(sum)
        ));
    }
    let shares = largest_remainder(
        1_000_000,
        &weights
            .iter()
            .map(|weight| u128::try_from(*weight).unwrap_or(0))
            .collect::<Vec<_>>(),
        |_| 0,
    );
    Ok(shares
        .into_iter()
        .map(|share| i64::try_from(share).unwrap_or(0))
        .collect())
}

/// Rows nested in `given` order, leaves decoded by `leaf`.
fn rows<T>(
    loose: &Loose,
    parents: &[Vec<ValueSlug>],
    depth: usize,
    leaf: &dyn Fn(&Loose) -> Result<T, String>,
    at: &str,
    id: &QualifiedId,
) -> Result<Rows<T>, ProfileError> {
    let bad = |detail: String| refused("BadDistribution", Some(at), format!("'{id}': {detail}"));
    let Some(values) = parents.get(depth) else {
        return leaf(loose).map(Rows::Leaf).map_err(bad);
    };
    let Loose::Map(map) = loose else {
        return Err(bad("a conditional row is a map of parent values".to_owned()));
    };
    let mut by = BTreeMap::new();
    let mut otherwise = None;
    for (key, value) in map {
        let next = rows(value, parents, depth + 1, leaf, at, id)?;
        if key == "otherwise" {
            otherwise = Some(Box::new(next));
        } else {
            let slug = values
                .iter()
                .find(|known| known.as_str() == key)
                .ok_or_else(|| bad(format!("'{key}' is not a value of the parent")))?;
            by.insert(slug.clone(), next);
        }
    }
    Ok(Rows::Node { by, otherwise })
}

/// A `when:` over parents: each a parent of the distribution (or, for a role, earlier), each value in
/// its domain.
fn condition(
    catalogue: &Catalogue,
    allowed: &[QualifiedId],
    when: &BTreeMap<String, Vec<String>>,
) -> Result<When, String> {
    let mut resolved = When::new();
    for (written, values) in when {
        let declaration = catalogue
            .resolve(written)
            .map_err(|error| error.to_string())?;
        if !allowed.contains(&declaration.id) {
            return Err(format!(
                "'{}' is not one of the attributes this may be conditioned on",
                declaration.id
            ));
        }
        let domain = categorical(&declaration.kind).unwrap_or_default();
        let mut set = BTreeSet::new();
        for value in values {
            let slug = domain
                .iter()
                .find(|known| known.as_str() == value)
                .ok_or_else(|| format!("'{value}' is not a value of '{}'", declaration.id))?;
            set.insert(slug.clone());
        }
        resolved.insert(declaration.id.clone(), set);
    }
    Ok(resolved)
}

fn adjustment(
    catalogue: &Catalogue,
    given: &[QualifiedId],
    values: &[ValueSlug],
    when: &BTreeMap<String, Vec<String>>,
    ratios: &BTreeMap<String, Loose>,
    at: &str,
    id: &QualifiedId,
) -> Result<Adjust, ProfileError> {
    let bad = |detail: String| refused("BadAdjustment", Some(at), format!("'{id}': {detail}"));
    let when = condition(catalogue, given, when).map_err(&bad)?;
    let mut resolved = BTreeMap::new();
    for (value, ratio) in ratios {
        let index = values
            .iter()
            .position(|known| known.as_str() == value)
            .ok_or_else(|| bad(format!("'{value}' is not a value")))?;
        let text = match ratio {
            Loose::Number(text) | Loose::Text(text) => text,
            other => return Err(bad(format!("{} is not a ratio", other.describe()))),
        };
        let ratio = micro(text).map_err(|error| bad(error.to_string()))?;
        if ratio <= 0 {
            return Err(bad(format!("a ratio is above 0, not {text}")));
        }
        resolved.insert(index, ratio);
    }
    Ok(Adjust {
        when,
        ratios: resolved,
    })
}

fn masking(
    catalogue: &Catalogue,
    given: &[QualifiedId],
    values: &[ValueSlug],
    when: &BTreeMap<String, Vec<String>>,
    only: &[String],
    at: &str,
    id: &QualifiedId,
) -> Result<Mask, ProfileError> {
    let when = condition(catalogue, given, when)
        .map_err(|detail| refused("BadMask", Some(at), format!("'{id}': {detail}")))?;
    let mut indices = BTreeSet::new();
    for value in only {
        let index = values
            .iter()
            .position(|known| known.as_str() == value)
            .ok_or_else(|| {
                refused(
                    "BadMask",
                    Some(at),
                    format!("'{id}': '{value}' is not a value"),
                )
            })?;
        indices.insert(index);
    }
    Ok(Mask {
        when,
        only: indices,
    })
}

/// `uniform_age_within_band`: an instant given exactly one band-derived parent, whose source is an
/// `age` at scale 0; the bands' years from the source's minimum, the thresholds and `upper`.
fn age_within_band(
    catalogue: &Catalogue,
    declaration: &Declaration,
    given: &[QualifiedId],
    raw: &RawDistribution,
    at: &str,
) -> Result<Shape, ProfileError> {
    let id = &declaration.id;
    let missing = |detail: &str| refused("MissingSupport", Some(at), format!("'{id}': {detail}"));
    if declaration.kind != Kind::Instant {
        return Err(missing("uniform_age_within_band is for an instant"));
    }
    let [band] = given else {
        return Err(missing("uniform_age_within_band is given exactly one band"));
    };
    let band = catalogue
        .get(band)
        .ok_or_else(|| missing("its band is undeclared"))?;
    let Some(Derive::Band { from, below }) = &band.derive else {
        return Err(missing("its parent is not a band"));
    };
    let lowest = match catalogue.get(from).map(|source| &source.kind) {
        Some(Kind::Quantity(quantity)) if quantity.scale == 0 => quantity.min,
        _ => return Err(missing("its band's source is not whole years")),
    };
    let upper = match &raw.uniform_age_within_band {
        Some(Loose::Map(map)) => map
            .get("upper")
            .map(|loose| scaled(loose, 0))
            .transpose()
            .map_err(|detail| missing(&detail))?,
        Some(_) | None => None,
    };
    let mut edges = vec![lowest];
    for threshold in below {
        let Threshold::Quantity(value) = threshold else {
            return Err(missing("its band's thresholds are not years"));
        };
        edges.push(value.value());
    }
    edges.push(upper.ok_or_else(|| missing("the open last band states no upper"))?);
    if edges.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(missing("the bands' years are not increasing"));
    }
    Ok(Shape::AgeWithinBand {
        bands: edges.windows(2).map(|pair| (pair[0], pair[1])).collect(),
    })
}

/// The distributions in topological order of `given` (Kahn's algorithm, ties by id); a cycle is
/// `DependencyCycle`, printed from the smallest remaining id.
fn order(
    distributions: &BTreeMap<QualifiedId, Distribution>,
) -> Result<Vec<QualifiedId>, ProfileError> {
    let parents = |id: &QualifiedId| -> Vec<QualifiedId> {
        distributions[id]
            .given
            .iter()
            .filter(|parent| distributions.contains_key(*parent))
            .cloned()
            .collect()
    };
    let mut done: BTreeSet<QualifiedId> = BTreeSet::new();
    let mut order = Vec::new();
    loop {
        let ready = distributions.keys().find(|id| {
            !done.contains(*id) && parents(id).iter().all(|parent| done.contains(parent))
        });
        match ready {
            Some(id) => {
                done.insert(id.clone());
                order.push(id.clone());
            }
            None => break,
        }
    }
    if order.len() == distributions.len() {
        return Ok(order);
    }
    let start = distributions
        .keys()
        .find(|id| !done.contains(*id))
        .cloned()
        .unwrap_or_else(|| unreachable!("some id remains"));
    let mut path = vec![start.clone()];
    let mut at = start;
    loop {
        let next = parents(&at)
            .into_iter()
            .find(|parent| !done.contains(parent))
            .unwrap_or_else(|| unreachable!("a remaining id has a remaining parent"));
        if let Some(first) = path.iter().position(|id| *id == next) {
            let mut cycle: Vec<String> = path[first..].iter().map(ToString::to_string).collect();
            cycle.push(next.to_string());
            return Err(refused(
                "DependencyCycle",
                None,
                format!("[{}]", cycle.join(", ")),
            ));
        }
        path.push(next.clone());
        at = next;
    }
}

/// One role, split per target: each target in the order, each condition earlier than the target.
fn role(
    catalogue: &Catalogue,
    raw: &RawRole,
    order: &[QualifiedId],
    at: &str,
) -> Result<Vec<Role>, ProfileError> {
    let bad = |detail: String| refused("BadRole", Some(at), detail);
    let mut tags = BTreeSet::new();
    let mut conditions = BTreeMap::new();
    for (key, values) in &raw.when {
        if key == "tags" {
            for value in values {
                tags.insert(Tag::new(value).map_err(|error| bad(error.to_string()))?);
            }
        } else {
            conditions.insert(key.clone(), values.clone());
        }
    }
    let mut roles = Vec::new();
    let targets = raw
        .add
        .iter()
        .map(|entry| (entry, true))
        .chain(raw.set.iter().map(|entry| (entry, false)));
    for ((written, value), adds) in targets {
        let declaration = catalogue
            .resolve(written)
            .map_err(|error| bad(error.to_string()))?;
        let target = declaration.id.clone();
        let earlier: Vec<QualifiedId> = match order.iter().position(|id| *id == target) {
            Some(index) => order[..index].to_vec(),
            None => order.to_vec(),
        };
        let when = condition(catalogue, &earlier, &conditions).map_err(&bad)?;
        let action = if adds {
            let Kind::Set { .. } = declaration.kind else {
                return Err(bad(format!("add: is for a set, and '{target}' is not one")));
            };
            match decode(&declaration.kind, value).map_err(|error| bad(error.to_string()))? {
                AttributeValue::Set(members) => RoleAction::Add(members),
                _ => return Err(bad("add: takes a list".to_owned())),
            }
        } else {
            RoleAction::Set(
                decode(&declaration.kind, value).map_err(|error| bad(error.to_string()))?,
            )
        };
        roles.push(Role {
            tags: tags.clone(),
            when,
            target,
            action,
        });
    }
    Ok(roles)
}

/// Whether `when` holds for these parent values.
pub(crate) fn matches(when: &When, given: &[QualifiedId], values: &[ValueSlug]) -> bool {
    when.iter().all(|(parent, allowed)| {
        given
            .iter()
            .position(|id| id == parent)
            .and_then(|index| values.get(index))
            .is_some_and(|value| allowed.contains(value))
    })
}

/// The weights of `shape`'s values in the group `parents` (in `given` order): prior × matching ratios,
/// zeroed by a matching mask; `None` when no row covers the group.
pub(crate) fn group_weights(
    distribution: &Distribution,
    parents: &[ValueSlug],
) -> Option<Vec<u128>> {
    let Shape::Prior { rows, adjust, mask } = &distribution.shape else {
        return None;
    };
    let row = rows.row(parents)?;
    let mut weights: Vec<u128> = row
        .iter()
        .map(|weight| u128::try_from(*weight).unwrap_or(0))
        .collect();
    for entry in adjust
        .iter()
        .filter(|entry| matches(&entry.when, &distribution.given, parents))
    {
        for (index, ratio) in &entry.ratios {
            if let Some(weight) = weights.get_mut(*index) {
                *weight = weight.saturating_mul(u128::try_from(*ratio).unwrap_or(0)) / 1_000_000;
            }
        }
    }
    for entry in mask
        .iter()
        .filter(|entry| matches(&entry.when, &distribution.given, parents))
    {
        for (index, weight) in weights.iter_mut().enumerate() {
            if !entry.only.contains(&index) {
                *weight = 0;
            }
        }
    }
    Some(weights)
}

/// F-S30d-5: every combination of a distribution's possible parent values has a row (`MissingCondition`)
/// and, for a prior, a value of positive weight (`EmptySupport`). A parent's possible values are those
/// with positive weight in some row of its own distribution, or its whole domain.
fn check_conditions(catalogue: &Catalogue, profile: &Profile) -> Result<(), ProfileError> {
    let possible = |parent: &QualifiedId| -> Vec<ValueSlug> {
        let domain = catalogue
            .get(parent)
            .and_then(|declaration| categorical(&declaration.kind))
            .unwrap_or_default();
        match profile
            .distributions
            .get(parent)
            .map(|distribution| &distribution.shape)
        {
            Some(Shape::Prior { rows, .. }) => domain
                .iter()
                .enumerate()
                .filter(|(index, _)| {
                    rows.leaves()
                        .iter()
                        .any(|row| row.get(*index).is_some_and(|weight| *weight > 0))
                })
                .map(|(_, value)| value.clone())
                .collect(),
            _ => domain,
        }
    };
    for (id, distribution) in &profile.distributions {
        let options: Vec<Vec<ValueSlug>> = distribution.given.iter().map(possible).collect();
        let mut combination = vec![0_usize; options.len()];
        if options.iter().any(Vec::is_empty) {
            continue;
        }
        loop {
            let parents: Vec<ValueSlug> = combination
                .iter()
                .zip(&options)
                .map(|(index, values)| values[*index].clone())
                .collect();
            let shown = || {
                distribution
                    .given
                    .iter()
                    .zip(&parents)
                    .map(|(parent, value)| format!("{parent}={value}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let covered = match &distribution.shape {
                Shape::Prior { rows, .. } => rows.row(&parents).is_some(),
                Shape::Normal { rows, .. } => rows.row(&parents).is_some(),
                _ => true,
            };
            if !covered {
                return Err(refused(
                    "MissingCondition",
                    None,
                    format!("'{id}' has no row for {}", shown()),
                ));
            }
            if let Some(weights) = group_weights(distribution, &parents)
                && weights.iter().all(|weight| *weight == 0)
            {
                return Err(refused(
                    "EmptySupport",
                    None,
                    format!("'{id}' has no value for {}", shown()),
                ));
            }
            let mut digit = 0;
            loop {
                if digit == combination.len() {
                    break;
                }
                combination[digit] += 1;
                if combination[digit] < options[digit].len() {
                    break;
                }
                combination[digit] = 0;
                digit += 1;
            }
            if digit == combination.len() {
                break;
            }
        }
    }
    let _ = Covers::All;
    Ok(())
}
