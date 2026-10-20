//! Population profiles (`DECISIONS.md` `ARC-123`; `MODULE_SPEC.md` §4.1 "Population profiles";
//! `pr-s30d-sampler.md` §5.4): the `populations/` file format, decoded with its line and column, and
//! the frozen [`Profile`] the sampler reads, every static rule checked when it is built.

pub(crate) mod rules;

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::Tag;
use serde::{Deserialize, Serialize};
use serde_saphyr::Spanned;

use crate::declaration::FormatOne;
use crate::id::{QualifiedId, ValueSlug};
use crate::value::{AttributeValue, Loose};

pub use rules::{ProfileError, build};

/// Who a profile describes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Covers {
    /// Everyone.
    #[default]
    All,
    /// People of 18 or more at the epoch, by every `age`-derived attribute (F-S30d-8).
    Adults,
}

/// Whether a source's figures were read, still need verifying, or are an authored convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    /// Read from the cited source.
    Read,
    /// Still to verify.
    Verify,
    /// An authored convention, never presented as data.
    Authored,
}

/// One cited source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// The citation.
    pub cite: String,
    /// Where it is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// When it was read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read: Option<String>,
    /// Its status.
    pub status: SourceStatus,
}

/// `uniform { min, max }` as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawUniform {
    min: Loose,
    max: Loose,
}

/// `table { quantiles }` as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawTable {
    quantiles: BTreeMap<String, Loose>,
}

/// One `adjust` entry as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAdjust {
    when: BTreeMap<String, Vec<String>>,
    ratios: BTreeMap<String, Loose>,
}

/// One `mask` entry as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawMask {
    when: BTreeMap<String, Vec<String>>,
    only: Vec<String>,
}

/// One distribution as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawDistribution {
    #[serde(default)]
    given: Vec<String>,
    #[serde(default)]
    prior: Option<Loose>,
    #[serde(default)]
    normal: Option<Loose>,
    #[serde(default)]
    uniform: Option<RawUniform>,
    #[serde(default)]
    table: Option<RawTable>,
    #[serde(default)]
    uniform_age_within_band: Option<Loose>,
    #[serde(default)]
    clip_sd: Option<u8>,
    #[serde(default)]
    adjust: Vec<RawAdjust>,
    #[serde(default)]
    mask: Vec<RawMask>,
    #[serde(default)]
    stratify: Option<bool>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    verify: Option<String>,
}

/// One role as written: `{ when: { tags?: [...], <id>: [...] }, add?: {...}, set?: {...} }`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawRole {
    #[serde(default)]
    when: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    add: BTreeMap<String, Loose>,
    #[serde(default)]
    set: BTreeMap<String, Loose>,
}

/// One `populations/*.yaml` file, decoded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileFile {
    #[allow(dead_code)]
    format: FormatOne,
    /// The profile's id.
    pub profile: String,
    /// Its version.
    pub version: u32,
    #[serde(default)]
    covers: Covers,
    #[serde(default)]
    sources: BTreeMap<String, Source>,
    #[serde(default)]
    distributions: BTreeMap<String, Spanned<RawDistribution>>,
    #[serde(default)]
    roles: Vec<Spanned<RawRole>>,
}

/// Rows of a conditional table: a leaf, or one level per parent in `given` order with an `otherwise:`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Rows<T> {
    /// The row for every combination below here.
    Leaf(T),
    /// One parent's values.
    Node {
        /// Per value.
        by: BTreeMap<ValueSlug, Rows<T>>,
        /// Every other value.
        otherwise: Option<Box<Rows<T>>>,
    },
}

impl<T> Rows<T> {
    /// The row for `parents`, in `given` order.
    pub fn row(&self, parents: &[ValueSlug]) -> Option<&T> {
        match (self, parents.split_first()) {
            (Self::Leaf(leaf), _) => Some(leaf),
            (Self::Node { by, otherwise }, Some((first, rest))) => by
                .get(first)
                .or(otherwise.as_deref())
                .and_then(|rows| rows.row(rest)),
            (Self::Node { .. }, None) => None,
        }
    }

    /// Every leaf.
    pub fn leaves(&self) -> Vec<&T> {
        match self {
            Self::Leaf(leaf) => vec![leaf],
            Self::Node { by, otherwise } => by
                .values()
                .chain(otherwise.as_deref())
                .flat_map(Self::leaves)
                .collect(),
        }
    }
}

/// `normal { mean, p5, p95 }` at the attribute's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NormalParams {
    /// The mean.
    pub mean: i64,
    /// The 5th percentile.
    pub p5: i64,
    /// The 95th percentile.
    pub p95: i64,
}

/// A condition over parent values: each named parent's value is one of the set.
pub type When = BTreeMap<QualifiedId, BTreeSet<ValueSlug>>;

/// One `adjust` entry: ratios (micro) per value index, where `when` matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Adjust {
    /// The condition.
    pub when: When,
    /// Per value index, the ratio in micro.
    pub ratios: BTreeMap<usize, i64>,
}

/// One `mask` entry: only these value indices, where `when` matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mask {
    /// The condition.
    pub when: When,
    /// The value indices allowed.
    pub only: BTreeSet<usize>,
}

/// How one attribute is drawn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Shape {
    /// A categorical prior: per row, a weight (micro, normalised to exactly 10⁶) per value index.
    Prior {
        /// The rows.
        rows: Rows<Vec<i64>>,
        /// The adjustments.
        adjust: Vec<Adjust>,
        /// The masks.
        mask: Vec<Mask>,
    },
    /// A normal by stratum, clipped at `clip_sd` standard deviations.
    Normal {
        /// The rows.
        rows: Rows<NormalParams>,
        /// 1 … 3.
        clip_sd: u8,
    },
    /// Uniform in `[min, max]` at the attribute's scale.
    Uniform {
        /// The least.
        min: i64,
        /// The most.
        max: i64,
    },
    /// Piecewise linear between quantiles (micro) and values at the attribute's scale.
    Table {
        /// The points, by quantile.
        points: Vec<(i64, i64)>,
    },
    /// An instant: an age uniform in whole days within the parent band's years.
    AgeWithinBand {
        /// Per band value index, `[low, high)` in years.
        bands: Vec<(i64, i64)>,
    },
}

/// One attribute's distribution, checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Distribution {
    /// The parents, in order.
    pub given: Vec<QualifiedId>,
    /// How it is drawn.
    pub shape: Shape,
    /// Stratified (the default) or independent.
    pub stratify: bool,
    /// The source it cites, or `authored`.
    pub source: String,
    /// A `verify` mark, reported and never refused here.
    pub verify: Option<String>,
}

/// What a role does to its target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RoleAction {
    /// Replaces the value.
    Set(AttributeValue),
    /// Unions into a set.
    Add(BTreeSet<ValueSlug>),
}

/// One role, applied right after its target is resolved (F-S30d-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Role {
    /// Tags every matching member carries.
    pub tags: BTreeSet<Tag>,
    /// Attribute conditions, each earlier in the order than the target.
    pub when: When,
    /// The attribute it changes.
    pub target: QualifiedId,
    /// How.
    pub action: RoleAction,
}

/// One profile, frozen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Profile {
    /// Its id.
    pub id: String,
    /// Its version.
    pub version: u32,
    /// Who it describes.
    pub covers: Covers,
    /// Its sources.
    pub sources: BTreeMap<String, Source>,
    /// Its distributions.
    pub distributions: BTreeMap<QualifiedId, Distribution>,
    /// Its roles.
    pub roles: Vec<Role>,
    /// The distributions in topological order of `given`, ties by id.
    pub order: Vec<QualifiedId>,
}

impl Profile {
    /// FNV-1a 64 of the profile's canonical JSON: its identity in `attributes-composed`.
    pub fn digest(&self) -> u64 {
        let canonical = serde_json::to_vec(self).unwrap_or_default();
        crate::sample::hash::fnv1a64(&canonical)
    }
}
