//! Who learns of a recorded fact: the one audience rule, owned here (`DECISIONS.md` `ARC-43`).
//!
//! A fact declares who *could* have learned of it — its [`Visibility`]. Whether a particular person
//! *did* is a perception judgement, and this pack is the perception system a world installs, so the
//! judgement is made here and nowhere else. The server delivers it three ways — the facts of an
//! observation frame, the reliable `perceived` stream, and `mineworld perceived` over a save — and
//! all three call the same two functions, which is what lets a live stream, a resumed one and an
//! offline export agree fact for fact.
//!
//! ```text
//! SystemInternal   nobody
//! Public           everybody
//! Participants     exactly the fact's participants
//! Entities(S)      exactly S
//! Place(p)         the fact's participants and subjects, and everybody whose whereabouts,
//!                  after this fact is applied, is p
//! ```
//!
//! # Judged against where people were when it happened
//!
//! A `Place` fact is heard by the people in that place *at that moment*. So the rule reads
//! [`Whereabouts`] — a fold of this pack's own `arrived` facts — advanced fact by fact in log order:
//! a person whose arrival is the fact hears it (they are there once it is applied), and a person
//! whose departure was recorded before it does not. Asking at any later moment, against wherever
//! people have walked to since, would answer a different question.
//!
//! # What this reads
//!
//! The envelope only, and of payloads only this pack's own `arrived`. It names no other pack's event
//! type: a fact narrowed by a world's interaction rules arrives here with its narrowed
//! [`Visibility`] already stated, and needs nothing from this function to be honoured.

use std::borrow::Borrow;
use std::collections::BTreeMap;

use mineworld_contracts::{EntityId, Event, EventEnvelope, EventId, PlaceId, Visibility};
use mineworld_kernel::WorldRead;

use crate::codec;
use crate::component::Presence;
use crate::event::Arrived;

/// Which place each person is in, as far as this pack's facts have said so far.
///
/// The same state as the [`Presence`] components, at the resolution the audience rule needs — the
/// place, not the position in it — and kept as a fold rather than read off the world, so that it can
/// be advanced one fact at a time between two facts of one dispatch, and rebuilt from a save's log
/// without resuming the world. Because [`Presence`] is itself a projection of `arrived`, a fold from
/// the first fact of a world equals [`Whereabouts::from_world`] of that world at the same revision.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Whereabouts {
    places: BTreeMap<EntityId, PlaceId>,
}

impl Whereabouts {
    /// Nobody anywhere: the state before a world's first fact.
    pub fn new() -> Self {
        Self::default()
    }

    /// Where everybody is in `world` now, read off this pack's own [`Presence`] components.
    ///
    /// The seed for a fold that continues from a running world rather than from its first fact.
    pub fn from_world(world: &WorldRead<'_>) -> Self {
        world
            .components::<Presence>()
            .map(|(person, presence)| (person, presence.location().place()))
            .collect()
    }

    /// Advances the fold past `fact`: an `arrived` moves its person; every other fact moves nobody.
    ///
    /// Total. An `arrived` whose payload this pack cannot read is a fact this pack's reducer would
    /// have refused, so it never reaches a log; it is ignored here rather than treated as an error.
    pub fn apply(&mut self, fact: &EventEnvelope) {
        if *fact.event_type() != Arrived::EVENT_TYPE {
            return;
        }
        if let Ok(arrived) = codec::event_payload::<Arrived>(fact.payload()) {
            self.places
                .insert(arrived.person().entity_id(), arrived.location().place());
        }
    }

    /// The place `person` is in, if this pack has been told of one.
    pub fn place_of(&self, person: EntityId) -> Option<PlaceId> {
        self.places.get(&person).copied()
    }
}

/// A hand-assembled state: each person in the place given, the last one given winning.
impl FromIterator<(EntityId, PlaceId)> for Whereabouts {
    fn from_iter<I: IntoIterator<Item = (EntityId, PlaceId)>>(people: I) -> Self {
        Self {
            places: people.into_iter().collect(),
        }
    }
}

/// Whether `observer` learns of `fact`, with `whereabouts` already advanced past it.
///
/// Total over [`Visibility`]: every audience a world can state has one answer, and the safe one for
/// bookkeeping (`SystemInternal`) is nobody.
pub fn admits(whereabouts: &Whereabouts, fact: &EventEnvelope, observer: EntityId) -> bool {
    match fact.visibility() {
        Visibility::SystemInternal => false,
        Visibility::Public => true,
        Visibility::Participants => fact.participants().contains(&observer),
        Visibility::Entities(audience) => audience.contains(&observer),
        Visibility::Place(place) => {
            fact.participants().contains(&observer)
                || fact.subjects().contains(&observer)
                || whereabouts.place_of(observer) == Some(*place)
        }
    }
}

/// The facts of a world's whole log, from its first fact in log order, that `observer` learned of,
/// with an [`EventId`] greater than `since` when one is given.
///
/// A fresh fold from the first fact: the facts at or before `since` are still applied, because where
/// people were is part of every later judgement, and only their delivery is skipped. The function a
/// resumed `perceived` stream and `mineworld perceived` both run.
pub fn perceived_by<I, F>(
    facts: I,
    observer: EntityId,
    since: Option<EventId>,
) -> impl Iterator<Item = F>
where
    I: IntoIterator<Item = F>,
    F: Borrow<EventEnvelope>,
{
    let mut whereabouts = Whereabouts::new();
    facts.into_iter().filter(move |fact| {
        let fact = fact.borrow();
        whereabouts.apply(fact);
        since.is_none_or(|since| fact.id() > since) && admits(&whereabouts, fact, observer)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use mineworld_contracts::{
        Causation, EntityType, EventRecord, Location, PersonId, Provenance, WorldTime,
    };
    use mineworld_kernel::SystemIdentity;

    use super::*;
    use crate::event::PersonEnteredPlace;
    use crate::system::PresenceSystem;

    const ANN: EntityId = EntityId::from_raw(1);
    const BEN: EntityId = EntityId::from_raw(2);
    const CAT: EntityId = EntityId::from_raw(3);
    const DAN: EntityId = EntityId::from_raw(4);
    const EVERYONE: [EntityId; 4] = [ANN, BEN, CAT, DAN];

    fn place(raw: u64) -> PlaceId {
        PlaceId::new(EntityId::from_raw(raw), EntityType::Place).expect("a place")
    }

    fn hall() -> PlaceId {
        place(10)
    }

    fn yard() -> PlaceId {
        place(11)
    }

    /// A fact that moves nobody: presence's own `person-entered-place` label, used only for its
    /// envelope, so the fixture names no other pack's vocabulary.
    fn noise(id: u64, visibility: Visibility) -> EventEnvelope {
        EventEnvelope::new(
            EventId::from_raw(id),
            WorldTime::from_seconds(0),
            EventRecord::new::<PersonEnteredPlace>(b"{}".to_vec()),
            Causation::WorldGenesis,
            visibility,
            Provenance::new(PresenceSystem::ID),
        )
    }

    /// `person` arrived in `to`, stated as presence states it — or, with `listed: false`, with no
    /// participant or subject named, so only the fold can put the person in the audience.
    fn arrived(id: u64, person: EntityId, to: PlaceId, listed: bool) -> EventEnvelope {
        let who = PersonId::new(person, EntityType::Person).expect("a person");
        let fact = EventEnvelope::new(
            EventId::from_raw(id),
            WorldTime::from_seconds(0),
            EventRecord::new::<Arrived>(codec::encode(&Arrived::new(who, Location::in_place(to)))),
            Causation::WorldGenesis,
            Visibility::Place(to),
            Provenance::new(PresenceSystem::ID),
        );
        if listed {
            fact.about(vec![person]).with_participants(vec![person])
        } else {
            fact
        }
    }

    fn audience(whereabouts: &Whereabouts, fact: &EventEnvelope) -> Vec<EntityId> {
        EVERYONE
            .into_iter()
            .filter(|observer| admits(whereabouts, fact, *observer))
            .collect()
    }

    /// Ann and Ben in the hall, Cat in the yard, Dan nowhere this pack knows of.
    fn placed() -> Whereabouts {
        [(ANN, hall()), (BEN, hall()), (CAT, yard())]
            .into_iter()
            .collect()
    }

    #[test]
    fn each_visibility_has_exactly_its_audience() {
        let here = placed();
        let listed = |fact: EventEnvelope| fact.about(vec![CAT]).with_participants(vec![DAN]);

        assert_eq!(
            audience(&here, &listed(noise(1, Visibility::SystemInternal))),
            Vec::<EntityId>::new(),
            "bookkeeping reaches nobody, not even the people it names"
        );
        assert_eq!(audience(&here, &noise(2, Visibility::Public)), EVERYONE);
        assert_eq!(
            audience(&here, &listed(noise(3, Visibility::Participants))),
            vec![DAN],
            "participants, and not the subjects"
        );
        assert_eq!(
            audience(
                &here,
                &listed(noise(4, Visibility::Entities(BTreeSet::from([ANN, CAT]))))
            ),
            vec![ANN, CAT],
            "exactly the set, and not the people the fact names"
        );
        assert_eq!(
            audience(&here, &noise(5, Visibility::Place(hall()))),
            vec![ANN, BEN],
            "the people in the place"
        );
        assert_eq!(
            audience(&here, &listed(noise(6, Visibility::Place(hall())))),
            vec![ANN, BEN, CAT, DAN],
            "plus its subjects and participants, wherever they are"
        );
        assert_eq!(
            audience(&here, &noise(7, Visibility::Place(place(99)))),
            Vec::<EntityId>::new(),
            "a place nobody is in"
        );
    }

    #[test]
    fn a_place_fact_is_not_heard_by_someone_elsewhere_or_nowhere() {
        let here = placed();
        let fact = noise(1, Visibility::Place(hall()));
        assert!(
            !admits(&here, &fact, CAT),
            "Cat is in the yard, neither a participant nor a subject"
        );
        assert!(
            !admits(&here, &fact, DAN),
            "Dan is nowhere this pack knows of, and names no participant"
        );
    }

    #[test]
    fn a_place_fact_is_judged_against_where_people_are_once_it_is_applied() {
        let log = vec![
            arrived(1, ANN, hall(), true),
            arrived(2, BEN, hall(), true),
            arrived(3, CAT, yard(), true),
            noise(4, Visibility::Place(hall())),
            // Ben leaves the hall: the arrival is heard in the yard, by Ben and by Cat.
            arrived(5, BEN, yard(), true),
            noise(6, Visibility::Place(hall())),
            // Dan's arrival names nobody: only the fold, advanced past it, puts Dan in the hall.
            arrived(7, DAN, hall(), false),
            noise(8, Visibility::Place(hall())),
        ];
        let heard = |observer: EntityId| -> Vec<u64> {
            perceived_by(&log, observer, None)
                .map(|fact| fact.id().raw())
                .collect()
        };

        assert_eq!(heard(ANN), vec![1, 2, 4, 6, 7, 8]);
        assert_eq!(
            heard(BEN),
            vec![2, 4, 5],
            "not Ann's arrival before Ben was anywhere; the line said before Ben left, and not \
             the one after"
        );
        assert_eq!(heard(CAT), vec![3, 5]);
        assert_eq!(
            heard(DAN),
            vec![7, 8],
            "Dan's own arrival is heard, judged after it is applied"
        );
    }

    #[test]
    fn since_skips_delivery_but_not_the_fold() {
        let log = [
            arrived(1, ANN, hall(), true),
            arrived(2, BEN, yard(), true),
            noise(3, Visibility::Place(hall())),
            noise(4, Visibility::Place(hall())),
        ];
        let after = |since: u64| -> Vec<u64> {
            perceived_by(log.iter(), ANN, Some(EventId::from_raw(since)))
                .map(|fact| fact.id().raw())
                .collect()
        };
        assert_eq!(after(0), vec![1, 3, 4]);
        assert_eq!(
            after(2),
            vec![3, 4],
            "Ann is still in the hall although her arrival is not delivered"
        );
        assert_eq!(after(3), vec![4]);
        assert_eq!(after(4), Vec::<u64>::new());
    }

    #[test]
    fn only_a_readable_arrived_moves_anybody() {
        let mut whereabouts = placed();
        let before = whereabouts.clone();
        whereabouts.apply(&noise(1, Visibility::Public));
        let mut unreadable = noise(2, Visibility::Public);
        unreadable = EventEnvelope::new(
            unreadable.id(),
            unreadable.at(),
            EventRecord::new::<Arrived>(b"not json".to_vec()),
            Causation::WorldGenesis,
            Visibility::Public,
            Provenance::new(PresenceSystem::ID),
        );
        whereabouts.apply(&unreadable);
        assert_eq!(whereabouts, before);

        whereabouts.apply(&arrived(3, CAT, hall(), true));
        assert_eq!(whereabouts.place_of(CAT), Some(hall()));
        assert_eq!(whereabouts.place_of(DAN), None);
    }
}
