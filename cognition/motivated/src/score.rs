//! Scoring and selection in integers (`pr-dl-b-motivated.md` SD-DLb-2 step 3–5, §5.3, §5.5).
//!
//! Every value is per mille. A choice's score is its base weight multiplied by its considerations through
//! Mark's compensation factor (the Infinite Axis Utility System's modification, in integers), and the choice
//! is a seeded draw weighted by score among the best. Each product and quotient is floor division in `u64`,
//! so the result is defined bit for bit.

use mineworld_decision::Draw;

use crate::considerations::{self, Consideration};
use crate::options::{self, Choice, Context};
use crate::settings::Settings;

/// Out of 1 000: the scale of every value.
const SCALE: u64 = 1_000;

/// The score of `base` multiplied by `values` through the compensation factor, in the order given.
///
/// With `k` values the modification is `1 000 − 1 000 / k`, and each value `c` is adjusted to
/// `c + ((1 000 − c) × modification / 1 000) × c / 1 000` before it multiplies the score. An empty list scores
/// the base. A single value passes through unchanged (`k = 1`).
pub fn combine(base: u32, values: &[u32]) -> u32 {
    if values.is_empty() {
        return base;
    }
    let count = values.len() as u64;
    let modification = SCALE - SCALE / count;
    let score = values.iter().fold(u64::from(base), |score, &value| {
        let c = u64::from(value);
        let adjusted = c + ((SCALE - c) * modification / SCALE) * c / SCALE;
        score * adjusted / SCALE
    });
    u32::try_from(score)
        .expect("a product of per-mille values with a per-mille base is at most the base")
}

/// One choice, its considerations and its score, as the explanation table prints them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scored {
    /// The choice.
    pub choice: Choice,
    /// Its considerations, in the order the score multiplies them.
    pub considerations: Vec<(Consideration, u32)>,
    /// The score, per mille.
    pub score: u32,
}

/// Every choice the observation offers, scored.
pub fn score_all(context: &Context<'_>, settings: &Settings) -> Vec<Scored> {
    options::choices(context)
        .into_iter()
        .map(|choice| {
            let considerations = considerations::of(&choice, context, settings);
            let values: Vec<u32> = considerations.iter().map(|(_, value)| *value).collect();
            let score = combine(options::base(&choice, settings), &values);
            Scored {
                choice,
                considerations,
                score,
            }
        })
        .collect()
}

/// The choice to make: a seeded draw, weighted by score, among the choices within `top_band` per mille of the
/// best. A door is then re-drawn among the doors in that band with `door_key`, which the caller derives from
/// the commitment window, so a person keeps heading for one door for the whole window (SD-DLb-5).
///
/// `None` when nothing scores above zero.
pub fn select<'a>(
    scored: &'a [Scored],
    top_band: u32,
    draw: &Draw,
    door_key: u64,
) -> Option<&'a Choice> {
    let best = u64::from(scored.iter().map(|entry| entry.score).max()?);
    if best == 0 {
        return None;
    }
    let threshold = best * (SCALE - u64::from(top_band)) / SCALE;
    let band: Vec<&Scored> = scored
        .iter()
        .filter(|entry| entry.score > 0 && u64::from(entry.score) >= threshold)
        .collect();
    let total: u64 = band.iter().map(|entry| u64::from(entry.score)).sum();
    let picked = weighted(&band, draw.below(total, 0))?;
    if !matches!(picked.choice, Choice::Door(_)) {
        return Some(&picked.choice);
    }
    let doors: Vec<&Scored> = band
        .iter()
        .copied()
        .filter(|entry| matches!(entry.choice, Choice::Door(_)))
        .collect();
    let door_total: u64 = doors.iter().map(|entry| u64::from(entry.score)).sum();
    weighted(&doors, door_key % door_total).map(|entry| &entry.choice)
}

/// The entry at `point` in the cumulative score of `entries`, where `point` is below their total.
fn weighted<'a>(entries: &[&'a Scored], point: u64) -> Option<&'a Scored> {
    let mut accumulated = 0_u64;
    for entry in entries {
        accumulated += u64::from(entry.score);
        if point < accumulated {
            return Some(entry);
        }
    }
    None
}
