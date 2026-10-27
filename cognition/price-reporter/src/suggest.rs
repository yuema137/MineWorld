//! The suggestion rule (`pr-s31d-reporter-policies.md` §5.3): pure, integer, one line per kind.
//!
//! ```text
//! sales  every Sale note's price                         each sale is its own observation
//! asks   the newest Ask note per (seller, place)          ask notes repeat across surveys (SD-S31d-4)
//! base   median(sales) if ≥ min_basis, else median(asks) if ≥ min_basis, else no line
//! stock  median in_stock of those asks that have one: < scarce_below → + scarce ‰; > plenty_above → − plenty ‰
//! cap    at most 32 lines: the largest basis kept (ties: the lower item), then by item
//! ```

use std::collections::BTreeMap;

use mineworld_notices::{NOTICE_PRICE_MAX, PRICE_LINES_MAX, PriceLine};
use mineworld_price_survey::{KindNotes, Note, PriceBook, Source};

use crate::ReporterParameters;

/// The median of `values`: the middle one, or the floor of the mean of the two middle ones. `None` when
/// empty.
fn median(mut values: Vec<u64>) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        Some(values[middle])
    } else {
        let sum = u128::from(values[middle - 1]) + u128::from(values[middle]);
        u64::try_from(sum / 2).ok()
    }
}

/// `base` scaled by `(1 000 ± permille) ÷ 1 000`, floored.
fn scaled(base: u64, permille: u128, up: bool) -> u64 {
    let factor = if up {
        1_000 + permille
    } else {
        1_000_u128.saturating_sub(permille)
    };
    u64::try_from(u128::from(base) * factor / 1_000).unwrap_or(u64::MAX)
}

/// The line for one kind, if its notes are enough.
fn line(kind: &KindNotes, parameters: &ReporterParameters) -> Option<PriceLine> {
    let notes = kind.notes();
    let sales: Vec<u64> = notes
        .iter()
        .filter(|note| note.source() == Source::Sale)
        .map(Note::price)
        .collect();
    // Notes are oldest first, so the last ask note of a (seller, place) is its newest.
    let mut newest: BTreeMap<(u64, u64), &Note> = BTreeMap::new();
    for note in notes.iter().filter(|note| note.source() == Source::Ask) {
        newest.insert((note.seller().raw(), note.place().entity_id().raw()), note);
    }
    let asks: Vec<u64> = newest.values().map(|note| note.price()).collect();
    let stock: Vec<u64> = newest
        .values()
        .filter_map(|note| note.in_stock())
        .map(u64::from)
        .collect();
    let least = usize::try_from(parameters.min_basis).unwrap_or(usize::MAX);
    let (mut base, basis) = if sales.len() >= least {
        (median(sales.clone())?, sales.len())
    } else if asks.len() >= least {
        (median(asks.clone())?, asks.len())
    } else {
        return None;
    };
    if let Some(stock) = median(stock) {
        if parameters.scarce_below > 0 && stock < u64::from(parameters.scarce_below) {
            base = scaled(base, u128::from(parameters.scarce_permille), true);
        } else if stock > u64::from(parameters.plenty_above) {
            base = scaled(base, u128::from(parameters.plenty_permille), false);
        }
    }
    let price = base.clamp(1, NOTICE_PRICE_MAX);
    let basis = u16::try_from(basis).unwrap_or(u16::MAX);
    Some(PriceLine::new(kind.item().entity_id(), price, basis))
}

/// One suggested line per kind with enough notes, at most 32, in item order.
pub fn suggest(book: &PriceBook, parameters: &ReporterParameters) -> Vec<PriceLine> {
    let mut lines: Vec<PriceLine> = book
        .kinds()
        .iter()
        .filter_map(|kind| line(kind, parameters))
        .collect();
    if lines.len() > PRICE_LINES_MAX {
        lines.sort_by(|a, b| {
            b.basis()
                .cmp(&a.basis())
                .then_with(|| a.item().raw().cmp(&b.item().raw()))
        });
        lines.truncate(PRICE_LINES_MAX);
    }
    lines.sort_by_key(|line| line.item().raw());
    lines
}
