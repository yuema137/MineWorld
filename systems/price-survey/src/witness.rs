//! Witnessing: which surveyors perceived a `sold`, decided by presence's own audience rule
//! (`ARC-43`, SD-S31c-2), never by a second "same place" check (F-S31c-5).
//!
//! `react` sees its generation reduced up to the `sold` and not past it (A-4). `sold` moves nobody,
//! so [`Whereabouts::from_world`] read here equals the audience fold advanced past the `sold` — the
//! state [`admits`] is defined against (F-S31c-6). It is built inside each reaction, never cached.

use mineworld_contracts::{EventEnvelope, PersonId};
use mineworld_economy::Sold;
use mineworld_kernel::{Emission, WorldRead};
use mineworld_presence::audience::{Whereabouts, admits};

use crate::component::PriceBook;
use crate::event::{Noted, PriceNoted, Source};
use crate::judge::{living_person, tracked};

/// One `price-noted { source: sale }` per surveyor that perceived `sold` and tracks its kind, in
/// surveyor id order.
pub(crate) fn witnesses(
    world: &WorldRead<'_>,
    envelope: &EventEnvelope,
    sold: &Sold,
) -> Vec<Emission> {
    let whereabouts = Whereabouts::from_world(world);
    world
        .components::<PriceBook>()
        .filter(|(surveyor, _)| admits(&whereabouts, envelope, *surveyor))
        .filter(|(_, book)| tracked(world, book, sold.item()))
        .filter_map(|(surveyor, _)| living_person(world, surveyor))
        .map(|surveyor: PersonId| {
            PriceNoted::new(
                surveyor,
                sold.place(),
                Source::Sale,
                vec![Noted::new(sold.item(), sold.seller(), sold.price(), None)],
            )
            .emission()
        })
        .collect()
}
