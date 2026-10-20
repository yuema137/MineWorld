//! The values a `motivated` consult is decided with: the calibrated and authored curves, the base weights,
//! the role tags and the scalar knobs (`pr-dl-b-motivated.md` §5.4, §6, SD-DLb-9).
//!
//! Built once and held by the model; a consult only reads it. The ATUS curves are derived here from the
//! committed table, and the authored curves are the defaults stated in §5.4 (fixed before any run, R-S24-2).

use std::collections::BTreeMap;

use mineworld_attributes_model::{QualifiedId, ValueSlug};
use mineworld_contracts::Tag;
use mineworld_rule_controller::commute::Commute;
use schemars::JsonSchema;
use serde::Serialize;

use crate::curve::Curve;
use crate::curves::{CurveSet, TABLE, Table, from_a3};

/// The base weight of each kind of choice, per mille (§5.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
// Each option may be omitted from `base`; an omitted option keeps its default.
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Base {
    /// Head for my agenda's place.
    #[schemars(range(min = 0, max = 1_000))]
    pub agenda: u32,
    /// Do nothing.
    #[schemars(range(min = 0, max = 1_000))]
    pub stay: u32,
    /// Greet somebody.
    #[schemars(range(min = 0, max = 1_000))]
    pub greet: u32,
    /// Walk to somebody.
    #[schemars(range(min = 0, max = 1_000))]
    pub approach: u32,
    /// Invite somebody to an activity.
    #[schemars(range(min = 0, max = 1_000))]
    pub invite: u32,
    /// Join somebody's activity.
    #[schemars(range(min = 0, max = 1_000))]
    pub join: u32,
    /// Leave my activity.
    #[schemars(range(min = 0, max = 1_000))]
    pub leave: u32,
    /// Walk through a doorway.
    #[schemars(range(min = 0, max = 1_000))]
    pub door: u32,
    /// Attempt a complete affordance the world offers.
    #[schemars(range(min = 0, max = 1_000))]
    pub offered: u32,
    /// Wander to a point nearby.
    #[schemars(range(min = 0, max = 1_000))]
    pub wander: u32,
    /// Set out for a punctual next part of my day (`ARC-282`).
    #[schemars(range(min = 0, max = 1_000))]
    pub commute: u32,
    /// Penalize a late member of staff, as a manager (`ARC-287`).
    #[schemars(range(min = 0, max = 1_000))]
    pub penalize: u32,
    /// Go to sleep where the world allows it, in a world with `needs` (`ARC-286`).
    #[schemars(range(min = 0, max = 1_000))]
    pub sleep: u32,
}

/// The tags that make a place a role (§5.4): the first role whose tags intersect a place's tags names it.
/// Each role lists at most eight tags (SD-DLb-7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
// Each role may be omitted from `roles`; an omitted role keeps its default.
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Roles {
    /// Places a person sleeps in.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub home: Vec<Tag>,
    /// Places a person eats in.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub eat: Vec<Tag>,
    /// Places a person works in.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub work: Vec<Tag>,
    /// Places a person shops in.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub shop: Vec<Tag>,
    /// Places a person is outdoors in.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub outdoor: Vec<Tag>,
}

/// Every curve a consideration reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Curves {
    /// The ATUS-derived set, `atus-a3-2025`.
    pub atus: CurveSet,
    /// Hours since I last had contact with anybody, to how much I want company.
    pub loneliness: Curve,
    /// My familiarity with somebody (0 … 1 000), to a score.
    pub familiarity: Curve,
    /// My regard for somebody (−1 000 … 1 000), to a score.
    pub regard: Curve,
    /// How far through my agenda slot I am (‰), to a score.
    pub lateness: Curve,
    /// Minutes since I joined my activity, to a score.
    pub activity_age: Curve,
    /// The urgency of setting out for a punctual next part (‰), to the commute's score (`ARC-282`).
    pub departure: Curve,
    /// The same urgency, to how readily I take up small talk: the inverse of `departure`.
    pub unhurried: Curve,
    /// How many times within the window a member of staff was late, to how readily a manager
    /// penalizes them (`ARC-287`).
    pub strictness: Curve,
    /// My hunger (0 … 1 000), to how much eating appeals (`ARC-286`).
    pub hungry: Curve,
    /// The inverse: my hunger, to how little eating appeals after a meal.
    pub sated: Curve,
    /// My tiredness (0 … 1 000), to how much sleep appeals.
    pub tired: Curve,
    /// The inverse: my tiredness, to how readily I take up small talk and wandering.
    pub wakeful: Curve,
}

/// One `attributes` entry (S30-f §5.7): a choice of `option` carries the consideration `by` gives the
/// observer's own disclosed value of `attribute` — only when that value is disclosed and named in `by`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AttributeRule {
    /// A qualified attribute id, `<namespace>.<slug>`, of a `level` or `category` attribute. Its shape is
    /// checked; its existence is not (the decision list is decoded without a catalogue).
    pub attribute: String,
    /// The option whose choices it adjusts: one of `base`'s options.
    pub option: String,
    /// Per value slug, the consideration's value, per mille 0 … 1 000.
    pub by: BTreeMap<String, u32>,
    /// `attribute`, typed.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) id: QualifiedId,
    /// `by`, typed.
    #[serde(skip)]
    #[schemars(skip)]
    pub(crate) values: BTreeMap<ValueSlug, u32>,
}

impl AttributeRule {
    /// The rule, its id and slugs checked.
    ///
    /// # Errors
    ///
    /// What is wrong with `attribute` or a slug of `by`.
    pub fn new(
        attribute: String,
        option: String,
        by: BTreeMap<String, u32>,
    ) -> Result<Self, String> {
        let id = QualifiedId::new(&attribute)
            .map_err(|error| format!("'{attribute}' is not a qualified attribute id: {error}"))?;
        let values = by
            .iter()
            .map(|(slug, value)| {
                ValueSlug::new(slug)
                    .map(|typed| (typed, *value))
                    .map_err(|error| format!("'{slug}' is not a value slug: {error}"))
            })
            .collect::<Result<_, String>>()?;
        Ok(Self {
            attribute,
            option,
            by,
            id,
            values,
        })
    }
}

/// The decision settings of one `motivated` model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// Out of 1 000: choices within this share of the best score form the band a draw chooses from.
    pub top_band: u32,
    /// How long, in minutes, a person keeps heading for the same door (the commitment window).
    pub commit_minutes: i64,
    /// Hours without contact at which loneliness reaches its top.
    pub social_need_hours: i64,
    /// The least chance, per mille, that an addressed person replies.
    pub reply_floor: u32,
    /// The base weight of each choice.
    pub base: Base,
    /// The role tags.
    pub roles: Roles,
    /// The curves.
    pub curves: Curves,
    /// When to set out for a punctual part of the day: `rule`'s policy block, shared (`ARC-282`).
    pub commute: Commute,
    /// The attribute considerations, in the order written (S30-f §5.7); none by default.
    pub attributes: Vec<AttributeRule>,
}

impl Settings {
    /// The settings with every default of §5.4 and §6.
    ///
    /// # Panics
    ///
    /// Never for the committed table: the table is parsed and checked by this crate's tests.
    pub fn defaults() -> Self {
        let table = Table::parse(TABLE).expect("the committed ATUS table parses");
        Self::with_table(&table, 3)
    }

    /// The settings with `table` as the ATUS source and `social_need_hours` as loneliness's top.
    pub fn with_table(table: &Table, social_need_hours: i64) -> Self {
        Self {
            top_band: 100,
            commit_minutes: 180,
            social_need_hours,
            reply_floor: 250,
            base: Base {
                agenda: 900,
                stay: 300,
                greet: 300,
                approach: 350,
                invite: 120,
                join: 100,
                leave: 150,
                door: 250,
                offered: 200,
                wander: 250,
                commute: 1_000,
                penalize: 600,
                sleep: 1_000,
            },
            roles: Roles {
                home: tags(&["residential"]),
                eat: tags(&["cafe"]),
                work: tags(&["office"]),
                shop: tags(&["shop"]),
                outdoor: tags(&["outdoor"]),
            },
            curves: Curves {
                atus: from_a3(table),
                loneliness: curve(&[(0, 200), (social_need_hours, 1_000)]),
                familiarity: curve(&[(0, 400), (1_000, 1_000)]),
                regard: curve(&[(-1_000, 0), (-200, 300), (0, 800), (1_000, 1_000)]),
                lateness: curve(&[(0, 600), (500, 900), (1_000, 1_000)]),
                activity_age: curve(&[(0, 50), (60, 300), (180, 1_000)]),
                departure: curve(&[(0, 0), (500, 700), (1_000, 1_000)]),
                unhurried: curve(&[(0, 1_000), (500, 300), (1_000, 0)]),
                strictness: curve(&[(1, 100), (2, 500), (3, 900)]),
                // Step-35 §6.3, fixed before any run (I-S35-7); `sated` and `wakeful` are the inverses.
                hungry: curve(&[(0, 100), (600, 800), (900, 1_000)]),
                sated: curve(&[(0, 900), (600, 200), (900, 0)]),
                tired: curve(&[(0, 0), (600, 700), (900, 1_000)]),
                wakeful: curve(&[(0, 1_000), (600, 300), (900, 0)]),
            },
            commute: Commute::default(),
            attributes: Vec::new(),
        }
    }
}

fn tags(names: &[&str]) -> Vec<Tag> {
    names
        .iter()
        .map(|name| Tag::new(*name).expect("a shipped role tag is a valid tag"))
        .collect()
}

fn curve(points: &[(i64, u32)]) -> Curve {
    Curve::new(points.to_vec()).expect("an authored curve is a valid curve")
}
