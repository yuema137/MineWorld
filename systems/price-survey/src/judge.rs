//! The one decision `survey`'s offer, validation and resolution all make (I-c-10), and the kinds
//! filter the witness rule shares.
//!
//! A survey copies only what economy discloses to the surveyor where it stands (I-c-4, SD-S31c-3):
//! the place's shop listing and the `asks` view of each person present, read through economy's own
//! `discloses`. It never reads `Holdings` or `Asks`, so it cannot learn a person seller's count.

use mineworld_contracts::{
    Component, EntityId, EntityType, ItemId, LifecycleState, PersonId, PlaceId, Rejection,
    RejectionCode,
};
use mineworld_economy::{Asks, AsksView, EconomySystem, Listing, Shop};
use mineworld_item::ItemKind;
use mineworld_kernel::WorldRead;
use mineworld_presence::{PerceptionProvider, Presence};
use serde::de::DeserializeOwned;

use crate::component::PriceBook;
use crate::event::Noted;

/// This pack's own refusal codes, each a `Rejection::System` code a client maps to words.
pub mod codes {
    use mineworld_contracts::RejectionCode;

    /// A section names a category that no declared kind of the world has.
    pub const SURVEY_CATEGORY_UNKNOWN: RejectionCode =
        RejectionCode::from_static("survey-category-unknown");
    /// No tracked ask is on display where the surveyor stands.
    pub const NOTHING_TO_SURVEY: RejectionCode = RejectionCode::from_static("nothing-to-survey");
    /// This place was surveyed less than `resurvey_after` ago.
    pub const SURVEYED_RECENTLY: RejectionCode = RejectionCode::from_static("surveyed-recently");
}

/// The refusal carrying `code`, with no detail.
pub(crate) const fn refusal(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

/// What an accepted survey notes: where, and the asks on display there, by `(item, seller)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The surveyor.
    pub surveyor: PersonId,
    /// Where it stands.
    pub place: PlaceId,
    /// The tracked asks on display there.
    pub notes: Vec<Noted>,
}

/// Whether `entity` is a living Person.
pub(crate) fn living_person(world: &WorldRead<'_>, entity: EntityId) -> Option<PersonId> {
    let record = world.entity(entity)?;
    if record.lifecycle() == LifecycleState::Destroyed {
        return None;
    }
    PersonId::new(entity, record.entity_type()).ok()
}

/// Whether `book` tracks `item`: a living Item with an `ItemKind` whose category the book lists.
pub(crate) fn tracked(world: &WorldRead<'_>, book: &PriceBook, item: ItemId) -> bool {
    let entity = item.entity_id();
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Item && record.lifecycle() != LifecycleState::Destroyed
    }) && world
        .component::<ItemKind>(entity)
        .is_some_and(|kind| book.tracks(kind.category()))
}

/// The record of component type `C` economy discloses to `observer` about `subject`, decoded as `V`.
fn disclosed<C: Component, V: DeserializeOwned>(
    world: &WorldRead<'_>,
    observer: EntityId,
    subject: EntityId,
) -> Option<V> {
    EconomySystem
        .discloses(world, observer, subject)
        .into_iter()
        .find(|record| *record.component_type() == C::COMPONENT_TYPE)
        .and_then(|record| serde_json::from_value(record.payload().clone()).ok())
}

/// Every tracked ask economy discloses to `actor` in `place`, by `(item, seller)`: the shop's
/// listing with its stock counts, then each living person present (the actor included) with
/// availability only.
pub fn asks_here(
    world: &WorldRead<'_>,
    actor: EntityId,
    place: PlaceId,
    book: &PriceBook,
) -> Vec<Noted> {
    let mut notes = Vec::new();
    if let Some(listing) = disclosed::<Shop, Listing>(world, actor, place.entity_id()) {
        let operator = listing.operator().entity_id();
        notes.extend(
            listing
                .listed()
                .iter()
                .filter(|listed| tracked(world, book, listed.item()))
                .map(|listed| {
                    Noted::new(
                        listed.item(),
                        operator,
                        listed.price(),
                        Some(listed.in_stock()),
                    )
                }),
        );
    }
    for (person, presence) in world.components::<Presence>() {
        if presence.location().place() != place || living_person(world, person).is_none() {
            continue;
        }
        if let Some(view) = disclosed::<Asks, AsksView>(world, actor, person) {
            notes.extend(
                view.asks()
                    .iter()
                    .filter(|ask| tracked(world, book, ask.item()))
                    .map(|ask| Noted::new(ask.item(), person, ask.price(), None)),
            );
        }
    }
    notes.sort_by_key(|note| (note.item(), note.seller()));
    notes
}

/// Who may survey, and where: a living Person holding a book (`NoSupportedInteraction`
/// otherwise), placed (`PreconditionFailed` otherwise).
pub(crate) fn surveyor<'w>(
    world: &WorldRead<'w>,
    actor: EntityId,
) -> Result<(PersonId, &'w PriceBook, PlaceId), Rejection> {
    let person = living_person(world, actor).ok_or(Rejection::NoSupportedInteraction)?;
    let book = world
        .component::<PriceBook>(actor)
        .ok_or(Rejection::NoSupportedInteraction)?;
    let place = world
        .component::<Presence>(actor)
        .map(|presence| presence.location().place())
        .ok_or(Rejection::PreconditionFailed)?;
    Ok((person, book, place))
}

/// `survey`'s whole decision, in order (§5.6): a surveyor, placed; something tracked on display
/// (`nothing-to-survey`); this place not surveyed within `resurvey_after` (`surveyed-recently`).
///
/// # Errors
///
/// The first of those that fails.
pub fn judge_survey(world: &WorldRead<'_>, actor: EntityId) -> Result<Plan, Rejection> {
    let (surveyor, book, place) = surveyor(world, actor)?;
    let notes = asks_here(world, actor, place, book);
    if notes.is_empty() {
        return Err(refusal(codes::NOTHING_TO_SURVEY));
    }
    if let Some(at) = book.last_surveyed(place)
        && world.at().seconds() < at.seconds() + i64::from(book.resurvey_after())
    {
        return Err(refusal(codes::SURVEYED_RECENTLY));
    }
    Ok(Plan {
        surveyor,
        place,
        notes,
    })
}
