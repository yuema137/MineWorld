//! A rule controller that takes initiative, consulted on a fixed pace, and remembers nothing.
//!
//! ```text
//! RuleController         reactive: answers whoever spoke, once — remembers which line it answered
//!                        (a hosted world's --agent; F-13 stays with it)
//! PacedRuleController    consulted every `pace` seconds by a headless run; answers, greets,
//!                        approaches, wanders and uses doorways — and holds no state at all
//! ```
//!
//! `docs/DECISIONS.md` `ARC-27` is the decision this implements. Two things make it what a headless
//! run needs:
//!
//! **It does something.** The real System Packs act only when asked, and a controller that only
//! answers is never asked anything in a world with no players: three hundred simulated days of such
//! a world are five genesis facts, perfectly reproducible and proving nothing (`ARC-23`). This
//! controller speaks first and walks.
//!
//! **It is a function.** [`PacedRuleController::decide`] takes `&self`: what it attempts is a pure
//! function of its seed, its pace and the observation, and the compiler says so. Every choice is a
//! mix of `(seed, observer, instant)`. "Answer each line once" is kept without a record of having
//! answered: the run consults each seat once per `pace`, never two seats at one instant, so a line
//! heard at `h` lies in exactly one of its listener's windows `(t − pace, t]`. A controller rebuilt
//! after a restart therefore decides exactly as the one that died — which is what lets a killed run
//! finish the same world (`AC-6` with controllers in the loop).
//!
//! Like [`RuleController`](crate::RuleController) it reads the server's verdicts and never computes
//! one (`ENGINEERING_RULES.md` §8): a walk it asks for may be refused (`no-route`, `TooFarAway`), and
//! that is the world's answer, not this controller's mistake to pre-empt.
//!
//! **It names destinations; the world walks there.** Every walking band asks movement's `walk-to` —
//! a doorway's place, a person, a point nearby — and movement plans the route, round whatever the
//! world's geometry holds (`DECISIONS.md` `ARC-75`). The strides are asked one at a time through
//! [`PacedRuleController::step`], at the cadence of whoever drives the controller. No route, stride or
//! wall is computed here.
//!
//! # The draws
//!
//! Every independent choice takes its own draw index, so adding a band never moves another band's
//! draws, and a world in which a band has nothing to do decides exactly as it did before that band
//! existed:
//!
//! ```text
//! 0        the walking roll: greet, approach, doorway, wander, stand        paced.rs
//! 1        answer an in-window line                                          paced.rs
//! 2, 3     greet: whom, and which greeting                                   paced.rs
//! 4        approach: whom                                                    paced.rs
//! 5, 6     wander: the offset on each axis                                   paced.rs
//! 7        free
//! 8 … 12   answer an invitation, initiative, invitee, kind, joined          social.rs
//! 13       follow the agenda                                                 agenda.rs
//! 14, 15   attempt a complete affordance, and which one                      offered.rs
//! ```
//!
//! The doorway choice is not a draw index: it is a separate mix over `(seed, observer, instant ÷ six
//! hours)`, so that a door is kept long enough to arrive at.

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, Component, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, SimDuration,
};
use mineworld_conversation::{Talk, Utterance};
use mineworld_movement::{Destination, Passage, Passages};
use serde::Serialize;
use serde_json::Value;

use crate::walking::{Walked, walk_to};
use crate::{agenda, offered, social};
use crate::{disclosed_history, may_talk_to, newest_per_speaker, reply_to};

/// Within this distance of somebody, approaching them again is pointless; the controller wanders.
/// A walk to a person ends within movement's `PERSON_APPROACH` (1 200 mm) of them, inside this.
const CLOSE_ENOUGH: i64 = 1_500;

/// The largest offset a wander names on each axis: a point a stride or two away. The walk there is
/// movement's, which moves a goal that lies in furniture or off the floor to the nearest free point.
const WANDER_AXIS: i64 = 1_400;

/// What a person says when they speak first. A fixed set, because a rule does not compose speech.
const GREETINGS: [&str; 6] = [
    "Good morning.",
    "Lovely day, isn't it?",
    "Have you been here long?",
    "Hello again.",
    "Busy in here today.",
    "Nice to see you.",
];

/// Out of 100: how often an in-window line is answered at all. Below 100 so that conversations end
/// and people get on with walking — a rule's version of losing interest.
const ANSWERS: u64 = 75;

/// Out of 100, cumulative: greet, approach, head for a doorway, wander; the rest is standing still.
const GREETS_BELOW: u64 = 20;
const APPROACHES_BELOW: u64 = 50;
const LEAVES_BELOW: u64 = 62;
const WANDERS_BELOW: u64 = 85;

/// In a place with more than one doorway — a street, which every other place opens onto — heading
/// for a door takes the bands from greeting up to here: people greet whoever they pass, and otherwise
/// mostly walk on. Without it a person on a street of five doors twelve to eighteen metres apart
/// spends most of a day getting to any of them (`step-09-social.md` C3).
const PASSES_THROUGH_BELOW: u64 = 80;

/// How long a person keeps heading for the same door, in simulated seconds: six hours. Which door is a
/// draw over `(seed, observer, instant ÷ this)`, so it is still a pure function of the observation —
/// and it keeps a person from changing their mind about where they are going at every consult.
const DOOR_WINDOW: i64 = 21_600;

/// A Person whose actions are a seeded rule, consulted every `pace` simulated seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacedRuleController {
    seed: u64,
    pace: SimDuration,
}

impl PacedRuleController {
    /// A controller with this seed, consulted every `pace` seconds by whoever drives it.
    ///
    /// The pace is the controller's to know because its answering window is one pace long; a driver
    /// that consulted it at another rate would make it answer lines twice or never.
    pub const fn new(seed: u64, pace: SimDuration) -> Self {
        Self { seed, pace }
    }

    /// The pace this controller expects to be consulted at.
    pub const fn pace(&self) -> SimDuration {
        self.pace
    }

    /// What this Person attempts at the instant of `observation`, or nothing.
    ///
    /// In order: answer the newest line somebody said to me since my last consult, if the server
    /// says I may speak to them; then the agenda and the social initiative; then, sometimes, a
    /// complete affordance the world offers; otherwise, by the seeded draw, greet somebody I may
    /// speak to, walk to somebody, walk through a doorway, wander to a point nearby, or do nothing.
    /// In a place with several doorways the draw favours walking on through one of them. A walk my own
    /// disclosed walk already makes is not asked again: that consult asks for nothing.
    pub fn decide(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
        let draw = Draw::new(self.seed, observation);
        // An invitation waiting for an answer first, then a line waiting for a reply: being addressed
        // comes before taking initiative (`social.rs`).
        if let Some(answer) = social::answer_invitation(observation, &draw) {
            return Some(answer);
        }
        if let Some(reply) = self.answer(observation, &draw) {
            return Some(reply);
        }
        // Then the day: away from where my agenda says, usually head there; at it, stay — no doorway
        // (`agenda.rs`). No agenda disclosed, nothing here changes and no draw is taken.
        let mine = agenda::own(observation);
        let at_agenda = mine
            .as_ref()
            .is_some_and(|agenda| agenda::is_there(observation, agenda));
        if let Some(agenda) = mine.as_ref().filter(|_| !at_agenda)
            && agenda::follows(&draw)
            && let Some(walked) = head_for(observation, agenda.place())
        {
            return walked.request();
        }
        // Part of an activity: sometimes leave it, and never head for a door — walking into another
        // place would leave it anyway. Part of nothing: sometimes invite somebody, or join somebody.
        let member = social::in_activity(observation);
        let social = if member {
            social::maybe_leave(observation, &draw)
        } else {
            social::initiative(observation, &draw)
        };
        if social.is_some() {
            return social;
        }
        // What the world offers complete — an action of a pack this controller was never compiled
        // against — sometimes (`offered.rs`, `ARC-34`). Nothing complete offered, nothing changes.
        if let Some(offered) = offered::attempt(observation, &draw) {
            return Some(offered);
        }
        // Heading for a door: never while part of an activity, never away from the agenda's place, and
        // — with an agenda elsewhere — only ever toward it, so a person on their way takes no detour
        // through somebody else's door. With no agenda, the seeded door, as before.
        let leave = |observation| match &mine {
            _ if member || at_agenda => None,
            Some(agenda) => head_for(observation, agenda.place()),
            None => self.leave(observation),
        };
        let roll = draw.below(100, 0);
        let passing_through = observation
            .self_location()
            .and_then(|here| doorways(observation, *here))
            .is_some_and(|doors| doors.iter().count() > 1);
        if roll < GREETS_BELOW {
            greet(observation, &draw)
        } else if passing_through && roll < PASSES_THROUGH_BELOW {
            leave(observation)
                .or_else(|| wander(observation, &draw))
                .and_then(Walked::request)
        } else if roll < APPROACHES_BELOW {
            approach(observation, &draw)
                .or_else(|| wander(observation, &draw))
                .and_then(Walked::request)
        } else if roll < LEAVES_BELOW {
            leave(observation).and_then(Walked::request)
        } else if roll < WANDERS_BELOW {
            wander(observation, &draw).and_then(Walked::request)
        } else {
            None
        }
    }

    /// The next stride of my own walk, or nothing: what a host asks between two consults of `decide`
    /// while my walk is disclosed (`step-11-bodies.md` SD-N16).
    ///
    /// A separate function, not a band of [`Self::decide`], because `decide` must be asked exactly one
    /// pace apart for "answer each line once" to hold. `step` answers nothing, greets nobody and takes
    /// no draw, so asking it at any instant changes nothing `decide` relies on.
    pub fn step(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
        crate::walking::step(observation)
    }

    /// The reply to the lowest-id speaker whose newest line was heard in this consult's window and
    /// whom the server says I may answer — sometimes.
    fn answer(&self, observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
        let me = observation.observer();
        let history = disclosed_history(observation.entity(me)?)?;
        let newest = newest_per_speaker(&history, me);
        let now = observation.at().seconds();
        let opened = now.checked_sub(self.pace.seconds())?;
        let (speaker, heard) = newest.iter().find(|(speaker, heard)| {
            let at = heard.at().seconds();
            opened < at && at <= now && may_talk_to(observation, **speaker)
        })?;
        if draw.below(100, 1) >= ANSWERS {
            return None;
        }
        let said = reply_to(heard, &newest, *speaker, observation)?;
        Some(ActionRequest::new(me, record(&Talk::new(said))).with_target(*speaker))
    }

    /// A walk through a doorway this place discloses, into the place it leads to.
    ///
    /// Which doorway is a draw over `(seed, observer, instant ÷ DOOR_WINDOW)`: the same door for the
    /// whole window, so a person crossing a street keeps walking to one door rather than turning at
    /// every consult, and a different door in the next window, so every place is visited. A place with
    /// one doorway has only one to choose. The passages are disclosed in `PlaceId` order, and before
    /// S8 this always took the first — on a street of five doors that sent everybody into the
    /// lowest-numbered place and never back (`step-09-social.md` F-6).
    fn leave(&self, observation: &Observation<Value>) -> Option<Walked> {
        let here = *observation.self_location()?;
        let doors: Vec<Passage> = doorways(observation, here)?.iter().copied().collect();
        let window = observation
            .at()
            .seconds()
            .div_euclid(DOOR_WINDOW)
            .cast_unsigned();
        let chosen = mix(
            mix(self.seed ^ 0x646f_6f72, observation.observer().raw()),
            window,
        ) % (doors.len() as u64);
        let passage = *doors.get(usize::try_from(chosen).ok()?)?;
        head_for(observation, passage.to())
    }
}

/// A walk into `place`, wherever it is: movement plans the way, through as many doorways as it takes
/// (`ARC-75`; step-11 N-D15). The destination is the place itself, with no point in it, so the walk
/// ends on entering it. Nothing here looks for a door.
fn head_for(observation: &Observation<Value>, place: PlaceId) -> Option<Walked> {
    observation.self_location()?;
    walk_to(observation, Destination::Place(Location::in_place(place)))
}

/// The seeded draws for one decision: a pure function of the seed, the observer and the instant.
pub(crate) struct Draw {
    mixed: u64,
}

impl Draw {
    pub(crate) fn new(seed: u64, observation: &Observation<Value>) -> Self {
        let observer = observation.observer().raw();
        // Two's complement bits of the instant: any i64 is a distinct input.
        let at = observation.at().seconds().cast_unsigned();
        Self {
            mixed: mix(mix(seed ^ 0x6d69_6e65_776f_726c, observer), at),
        }
    }

    /// The `n`th independent draw, in `0..bound`.
    pub(crate) fn below(&self, bound: u64, n: u64) -> u64 {
        mix(self.mixed, n) % bound
    }

    /// The `n`th draw, in `-reach..=reach`.
    fn offset(&self, reach: i64, n: u64) -> i64 {
        let span = reach.cast_unsigned() * 2 + 1;
        self.below(span, n).cast_signed() - reach
    }
}

/// SplitMix64's finalizer over two words: a pure, well-mixed function, the same on every machine.
fn mix(a: u64, b: u64) -> u64 {
    let mut z = a ^ b.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Says hello to somebody the server says I may speak to, chosen by the draw.
fn greet(observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
    let me = observation.observer();
    let reachable: Vec<EntityId> = others(observation)
        .map(PerceivedEntity::id)
        .filter(|person| may_talk_to(observation, *person))
        .collect();
    let target = *pick(&reachable, draw, 2)?;
    let words = GREETINGS[usize::try_from(draw.below(GREETINGS.len() as u64, 3)).ok()?];
    let said = Utterance::new(words).ok()?;
    Some(ActionRequest::new(me, record(&Talk::new(said))).with_target(target))
}

/// A walk to somebody in the same place, which ends near enough to talk; nothing if already close.
fn approach(observation: &Observation<Value>, draw: &Draw) -> Option<Walked> {
    let here = *observation.self_location()?;
    let from = here.local()?;
    let people: Vec<&PerceivedEntity<Value>> = others(observation)
        .filter(|person| {
            person
                .location()
                .is_some_and(|at| at.place() == here.place())
        })
        .collect();
    let target = pick(&people, draw, 4)?;
    let to = target.location()?.local()?;
    if distance(from, to) <= CLOSE_ENOUGH {
        return None;
    }
    let person = PersonId::new(target.id(), target.entity_type()).ok()?;
    walk_to(observation, Destination::Person(person))
}

/// A walk to a point in a seeded direction nearby, in the same place.
fn wander(observation: &Observation<Value>, draw: &Draw) -> Option<Walked> {
    let here = *observation.self_location()?;
    let from = here.local()?;
    let (dx, dy) = (draw.offset(WANDER_AXIS, 5), draw.offset(WANDER_AXIS, 6));
    if dx == 0 && dy == 0 {
        return None;
    }
    let to = LocalPosition::new(shifted(from.x(), dx)?, shifted(from.y(), dy)?, from.z());
    walk_to(observation, Destination::Place(here.with_local(to)))
}

/// Everybody the observation lists as a person, except the observer, in the order it lists them.
fn others(observation: &Observation<Value>) -> impl Iterator<Item = &PerceivedEntity<Value>> {
    let me = observation.observer();
    observation
        .entities()
        .iter()
        .filter(move |entity| entity.entity_type() == EntityType::Person && entity.id() != me)
}

/// The doorways the place I stand in discloses, as the observation carries them.
fn doorways(observation: &Observation<Value>, here: Location) -> Option<Passages> {
    let place = observation.entity(here.place().entity_id())?;
    let record = place
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)?;
    serde_json::from_value(record.payload_for::<Passages>().ok()?.clone()).ok()
}

/// One of `choices`, by the `n`th draw.
fn pick<'a, T>(choices: &'a [T], draw: &Draw, n: u64) -> Option<&'a T> {
    if choices.is_empty() {
        return None;
    }
    choices.get(usize::try_from(draw.below(choices.len() as u64, n)).ok()?)
}

/// Straight-line distance on the floor, in whole millimetres, rounded down: only to choose whom to
/// walk to ([`CLOSE_ENOUGH`]), never to measure a stride.
pub(crate) fn distance(a: LocalPosition, b: LocalPosition) -> i64 {
    let (dx, dy) = delta(a, b);
    isqrt(dx * dx + dy * dy)
}

fn delta(a: LocalPosition, b: LocalPosition) -> (i64, i64) {
    (
        i64::from(b.x().value()) - i64::from(a.x().value()),
        i64::from(b.y().value()) - i64::from(a.y().value()),
    )
}

fn shifted(value: Millimetres, by: i64) -> Option<Millimetres> {
    i32::try_from(i64::from(value.value()) + by)
        .ok()
        .map(Millimetres::new)
}

/// The integer square root of a non-negative number, rounded down.
fn isqrt(value: i64) -> i64 {
    value.max(0).cast_unsigned().isqrt().cast_signed()
}

/// This controller's own payload encoding, as [`RuleController`](crate::RuleController)'s.
fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(
        serde_json::to_vec(action)
            .expect("a request payload is JSON-representable by construction"),
    )
}
