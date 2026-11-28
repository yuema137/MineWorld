//! Reading a saved run's posture facts, checking the step's invariants after every request, and measuring
//! how people sit (`pr-s34d-npcs-agents-sit.md` SD-d-1, AC-d-1, AC-d-2; S34-d1).
//!
//! ```text
//! decode     place-sit-points, sat-down, stood-up (posture), arrived (presence), gave-way and person-shoved
//!            (bodies), each with its owner's published type, into one small closed enum
//! settle     the invariants are held at every request boundary: before each fact that is not a reaction
//!            (its cause is not another fact), and at the end — "after every request" (step §12)
//! I-S34-1    no two persons seated on one (place, point)
//! I-S34-2    a seated person's last arrival is the point's place and position; the sit itself faces the
//!            point's facing (BC-2 of pr-s34a: facing may change while seated, position may not)
//! I-S34-7    no gave-way names a seated person as the one who stepped aside; no person-shoved targets one
//! measure    the seated set at every whole hour 09:00-18:00; every spell (sat-down to stood-up), its length
//!            and how it ended
//! ```
//!
//! The expected values come from the journal: a point's position and facing are read from the world's own
//! `place-sit-points` fact, never from posture's code (test rules §25).

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use mineworld_bodies::{GaveWay, PersonShoved};
use mineworld_contracts::{Causation, EntityId, EventEnvelope};
use mineworld_posture::{PlaceSitPoints, SatDown, StandCause, StoodUp};
use mineworld_presence::Arrived;

use crate::social::decoded;

/// Where a person stands or sits: place, position (mm) and yaw (millidegrees) when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spot {
    pub place: EntityId,
    pub at: Option<(i32, i32)>,
    pub yaw: Option<i32>,
}

/// A posture-relevant fact, decoded with its owner's type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    /// A place's sit points: key → (x, y, yaw).
    SitPoints {
        place: EntityId,
        points: BTreeMap<String, (i32, i32, i32)>,
    },
    SatDown {
        person: EntityId,
        place: EntityId,
        point: String,
        at: (i32, i32),
        yaw: i32,
    },
    StoodUp {
        person: EntityId,
        requested: bool,
    },
    Arrived {
        person: EntityId,
        spot: Spot,
    },
    /// `person` stepped aside for a walker.
    GaveWay {
        person: EntityId,
    },
    /// `person` was shoved.
    Shoved {
        person: EntityId,
    },
}

/// One fact of a run: its instant, whether it begins a request (its cause is not another fact), and what
/// it says, if it is one of the kinds read here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Read {
    pub at: i64,
    pub begins_request: bool,
    pub fact: Option<Fact>,
}

/// Decodes one stored fact.
pub fn read(envelope: &EventEnvelope) -> Read {
    let begins_request = !matches!(envelope.caused_by(), Causation::Event(_));
    let fact = match envelope.event_type().as_str() {
        "place-sit-points" => decoded::<PlaceSitPoints>(envelope).map(|stated| Fact::SitPoints {
            place: stated.place().entity_id(),
            points: stated
                .points()
                .iter()
                .map(|point| {
                    (
                        point.key().as_str().to_owned(),
                        (
                            point.at().x().value(),
                            point.at().y().value(),
                            point.facing().value(),
                        ),
                    )
                })
                .collect(),
        }),
        "sat-down" => decoded::<SatDown>(envelope).map(|sat| Fact::SatDown {
            person: sat.person().entity_id(),
            place: sat.place().entity_id(),
            point: sat.point().as_str().to_owned(),
            at: (sat.at().x().value(), sat.at().y().value()),
            yaw: sat.facing().yaw().value(),
        }),
        "stood-up" => decoded::<StoodUp>(envelope).map(|stood| Fact::StoodUp {
            person: stood.person().entity_id(),
            requested: stood.cause() == StandCause::Requested,
        }),
        "arrived" => decoded::<Arrived>(envelope).map(|arrived| {
            let location = arrived.location();
            Fact::Arrived {
                person: arrived.person().entity_id(),
                spot: Spot {
                    place: location.place().entity_id(),
                    at: location.local().map(|at| (at.x().value(), at.y().value())),
                    yaw: location.facing().map(|facing| facing.yaw().value()),
                },
            }
        }),
        "gave-way" => decoded::<GaveWay>(envelope).map(|gave| Fact::GaveWay {
            person: gave.person().entity_id(),
        }),
        "person-shoved" => decoded::<PersonShoved>(envelope).map(|shoved| Fact::Shoved {
            person: shoved.person().entity_id(),
        }),
        _ => None,
    };
    Read {
        at: envelope.at().seconds(),
        begins_request,
        fact,
    }
}

/// A spell: from a person's sat-down to their next stood-up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spell {
    pub seconds: i64,
    pub requested: bool,
}

/// A seated row as the journal states it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    place: EntityId,
    point: String,
    since: i64,
}

/// The journal's posture state, rebuilt fact by fact, with every invariant breach named.
#[derive(Debug, Default)]
pub struct Ledger {
    points: BTreeMap<EntityId, BTreeMap<String, (i32, i32, i32)>>,
    seated: BTreeMap<EntityId, Row>,
    spots: BTreeMap<EntityId, Spot>,
    pub spells: Vec<Spell>,
    pub breaches: Vec<String>,
    /// Hour samples 09:00-18:00: (instant, how many persons were seated).
    pub samples: Vec<(i64, usize)>,
    next_sample: Option<i64>,
}

/// The first and last whole hour sampled each day.
const FIRST_HOUR: i64 = 9;
const LAST_HOUR: i64 = 18;
const HOUR: i64 = 3_600;
const DAY: i64 = 86_400;

/// The first sampled instant at or after `at`.
fn sample_at_or_after(at: i64) -> i64 {
    let mut instant = at.div_euclid(HOUR) * HOUR;
    if instant < at {
        instant += HOUR;
    }
    loop {
        let hour = instant.rem_euclid(DAY) / HOUR;
        if (FIRST_HOUR..=LAST_HOUR).contains(&hour) {
            return instant;
        }
        instant += HOUR;
    }
}

impl Ledger {
    /// Feeds every fact of a run, in order, and settles at the end of the run's last instant.
    pub fn of(reads: impl IntoIterator<Item = Read>, until: i64) -> Self {
        let mut ledger = Self::default();
        for read in reads {
            ledger.feed(&read);
        }
        ledger.settle();
        ledger.sample_through(until);
        ledger
    }

    /// One fact: settle first if it begins a new request, then take the samples before its instant, then
    /// apply it.
    pub fn feed(&mut self, read: &Read) {
        if read.begins_request {
            self.settle();
        }
        self.sample_before(read.at);
        let Some(fact) = &read.fact else {
            return;
        };
        match fact {
            Fact::SitPoints { place, points } => {
                self.points.insert(*place, points.clone());
            }
            Fact::SatDown {
                person,
                place,
                point,
                at,
                yaw,
            } => self.sat_down(*person, *place, point, *at, *yaw, read.at),
            Fact::StoodUp { person, requested } => {
                if let Some(row) = self.seated.remove(person) {
                    self.spells.push(Spell {
                        seconds: read.at - row.since,
                        requested: *requested,
                    });
                } else {
                    self.breaches
                        .push(format!("{person} stood up at {} without sitting", read.at));
                }
            }
            Fact::Arrived { person, spot } => {
                self.spots.insert(*person, *spot);
            }
            Fact::GaveWay { person } if self.seated.contains_key(person) => self
                .breaches
                .push(format!("I-S34-7: seated {person} gave way at {}", read.at)),
            Fact::Shoved { person } if self.seated.contains_key(person) => self.breaches.push(
                format!("I-S34-7: seated {person} was shoved at {}", read.at),
            ),
            Fact::GaveWay { .. } | Fact::Shoved { .. } => {}
        }
    }

    fn sat_down(
        &mut self,
        person: EntityId,
        place: EntityId,
        point: &str,
        at: (i32, i32),
        yaw: i32,
        instant: i64,
    ) {
        let Some(&(x, y, facing)) = self.points.get(&place).and_then(|points| points.get(point))
        else {
            self.breaches.push(format!(
                "{person} sat on {point}, which {place} never listed"
            ));
            return;
        };
        if at != (x, y) || yaw != facing {
            self.breaches.push(format!(
                "I-S34-2: {person} sat on {point} at {at:?} facing {yaw}; the point is ({x}, {y}) facing {facing}"
            ));
        }
        if let Some((other, _)) = self
            .seated
            .iter()
            .find(|(other, row)| **other != person && row.place == place && row.point == point)
        {
            self.breaches.push(format!(
                "I-S34-1: {person} sat on {point} of {place} at {instant}, held by {other}"
            ));
        }
        self.seated.insert(
            person,
            Row {
                place,
                point: point.to_owned(),
                since: instant,
            },
        );
    }

    /// The invariants at a request boundary: every seated person's last arrival is on their point. The
    /// facing of the sit itself is held at `sat-down` (BC-2).
    pub fn settle(&mut self) {
        for (person, row) in &self.seated {
            let Some(&(x, y, facing)) = self
                .points
                .get(&row.place)
                .and_then(|points| points.get(&row.point))
            else {
                continue;
            };
            let spot = self.spots.get(person);
            let on_point =
                spot.is_some_and(|spot| spot.place == row.place && spot.at == Some((x, y)));
            if !on_point {
                self.breaches.push(format!(
                    "I-S34-2: seated {person} on {} is at {spot:?}, not ({x}, {y}) facing {facing}",
                    row.point
                ));
            }
        }
        // One breach of a long-held row is named once, not at every later request.
        self.breaches.dedup();
    }

    fn sample_before(&mut self, instant: i64) {
        let next = *self
            .next_sample
            .get_or_insert_with(|| sample_at_or_after(instant));
        if next < instant {
            self.sample_through(instant - 1);
        }
    }

    /// Takes every sample at or before `until`.
    fn sample_through(&mut self, until: i64) {
        let Some(mut next) = self.next_sample else {
            return;
        };
        while next <= until {
            self.samples.push((next, self.seated.len()));
            next = sample_at_or_after(next + 1);
        }
        self.next_sample = Some(next);
    }

    /// Out of the hour samples, how many show somebody seated.
    pub fn showing_seating(&self) -> usize {
        self.samples
            .iter()
            .filter(|(_, seated)| *seated > 0)
            .count()
    }

    /// The spells that ended by the sitter's own `stand` within one consult (SD-d-1).
    pub fn bobbing(&self, pace: i64) -> usize {
        self.spells
            .iter()
            .filter(|spell| spell.requested && spell.seconds <= pace)
            .count()
    }

    /// The median spell, in seconds; 0 with none.
    pub fn median_spell(&self) -> i64 {
        let mut lengths: Vec<i64> = self.spells.iter().map(|spell| spell.seconds).collect();
        lengths.sort_unstable();
        lengths.get(lengths.len() / 2).copied().unwrap_or(0)
    }

    /// The persons who sat at least once, by how many spells they began.
    pub fn sitters(&self) -> BTreeSet<EntityId> {
        self.seated.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    //! The checker against planted journals: each breach it exists to find is found (X-d-7).

    use super::*;

    const CAFE: u64 = 1;
    const ANN: u64 = 2;
    const BEN: u64 = 3;

    fn id(raw: u64) -> EntityId {
        EntityId::from_raw(raw)
    }

    fn at(instant: i64, begins_request: bool, fact: Fact) -> Read {
        Read {
            at: instant,
            begins_request,
            fact: Some(fact),
        }
    }

    fn points() -> Read {
        at(
            0,
            true,
            Fact::SitPoints {
                place: id(CAFE),
                points: BTreeMap::from([
                    ("a".to_owned(), (1_000, 1_000, 90_000)),
                    ("b".to_owned(), (2_000, 1_000, 270_000)),
                ]),
            },
        )
    }

    fn arrive(instant: i64, person: u64, x: i32, yaw: i32, begins: bool) -> Read {
        at(
            instant,
            begins,
            Fact::Arrived {
                person: id(person),
                spot: Spot {
                    place: id(CAFE),
                    at: Some((x, 1_000)),
                    yaw: Some(yaw),
                },
            },
        )
    }

    fn sit(instant: i64, person: u64, point: &str, x: i32, yaw: i32) -> Read {
        at(
            instant,
            false,
            Fact::SatDown {
                person: id(person),
                place: id(CAFE),
                point: point.to_owned(),
                at: (x, 1_000),
                yaw,
            },
        )
    }

    #[test]
    fn a_clean_sit_and_stand_breaches_nothing_and_is_one_spell() {
        let ledger = Ledger::of(
            [
                points(),
                arrive(100, ANN, 1_000, 90_000, true),
                sit(100, ANN, "a", 1_000, 90_000),
                arrive(2_000, ANN, 1_200, 0, true),
                at(
                    2_000,
                    false,
                    Fact::StoodUp {
                        person: id(ANN),
                        requested: false,
                    },
                ),
            ],
            2_000,
        );
        assert_eq!(ledger.breaches, Vec::<String>::new());
        assert_eq!(
            ledger.spells,
            vec![Spell {
                seconds: 1_900,
                requested: false
            }]
        );
    }

    #[test]
    fn a_sit_facing_away_from_the_point_is_named() {
        let ledger = Ledger::of(
            [
                points(),
                arrive(100, ANN, 1_000, 0, true),
                sit(100, ANN, "a", 1_000, 0),
            ],
            100,
        );
        assert!(
            ledger
                .breaches
                .iter()
                .any(|breach| breach.starts_with("I-S34-2") && breach.contains("facing 0")),
            "{:?}",
            ledger.breaches
        );
    }

    #[test]
    fn a_seated_person_moved_off_the_point_without_standing_is_named_at_the_next_request() {
        let ledger = Ledger::of(
            [
                points(),
                arrive(100, ANN, 1_000, 90_000, true),
                sit(100, ANN, "a", 1_000, 90_000),
                arrive(200, ANN, 1_300, 90_000, true),
            ],
            200,
        );
        assert!(
            ledger
                .breaches
                .iter()
                .any(|breach| breach.starts_with("I-S34-2")),
            "{:?}",
            ledger.breaches
        );
    }

    #[test]
    fn two_on_one_point_a_seated_person_giving_way_or_shoved_are_named() {
        let ledger = Ledger::of(
            [
                points(),
                arrive(100, ANN, 1_000, 90_000, true),
                sit(100, ANN, "a", 1_000, 90_000),
                arrive(200, BEN, 1_000, 90_000, true),
                sit(200, BEN, "a", 1_000, 90_000),
                at(300, true, Fact::GaveWay { person: id(ANN) }),
                at(400, true, Fact::Shoved { person: id(ANN) }),
            ],
            400,
        );
        for invariant in [
            "I-S34-1",
            "I-S34-7: seated 2 gave way",
            "I-S34-7: seated 2 was shoved",
        ] {
            assert!(
                ledger
                    .breaches
                    .iter()
                    .any(|breach| breach.starts_with(invariant)),
                "{invariant}: {:?}",
                ledger.breaches
            );
        }
    }

    #[test]
    fn hour_samples_fall_on_whole_hours_from_nine_to_six() {
        let ledger = Ledger::of(
            [at(
                DAY + 8 * HOUR + 1,
                true,
                Fact::GaveWay { person: id(ANN) },
            )],
            2 * DAY,
        );
        let hours: Vec<i64> = ledger
            .samples
            .iter()
            .map(|(instant, _)| instant.rem_euclid(DAY) / HOUR)
            .collect();
        assert_eq!(hours, (FIRST_HOUR..=LAST_HOUR).collect::<Vec<_>>());
    }
}
