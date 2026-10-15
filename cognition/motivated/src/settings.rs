//! The values a `motivated` consult is decided with: the calibrated and authored curves, the base weights,
//! the role tags and the scalar knobs (`pr-dl-b-motivated.md` §5.4, §6, SD-DLb-9).
//!
//! Built once and held by the model; a consult only reads it. The ATUS curves are derived here from the
//! committed table, and the authored curves are the defaults stated in §5.4 (fixed before any run, R-S24-2).

use mineworld_contracts::Tag;

use crate::curve::Curve;
use crate::curves::{CurveSet, TABLE, Table, from_a3};

/// The base weight of each kind of choice, per mille (§5.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base {
    /// Head for my agenda's place.
    pub agenda: u32,
    /// Do nothing.
    pub stay: u32,
    /// Greet somebody.
    pub greet: u32,
    /// Walk to somebody.
    pub approach: u32,
    /// Invite somebody to an activity.
    pub invite: u32,
    /// Join somebody's activity.
    pub join: u32,
    /// Leave my activity.
    pub leave: u32,
    /// Walk through a doorway.
    pub door: u32,
    /// Attempt a complete affordance the world offers.
    pub offered: u32,
    /// Wander to a point nearby.
    pub wander: u32,
}

/// The tags that make a place a role (§5.4): the first role whose tags intersect a place's tags names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roles {
    /// Places a person sleeps in.
    pub home: Vec<Tag>,
    /// Places a person eats in.
    pub eat: Vec<Tag>,
    /// Places a person works in.
    pub work: Vec<Tag>,
    /// Places a person shops in.
    pub shop: Vec<Tag>,
    /// Places a person is outdoors in.
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
            },
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
