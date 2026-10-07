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
//! one (`ENGINEERING_RULES.md` §8): a stride it proposes may be refused `TooFarAway`, and that is the
//! world's answer, not this controller's mistake to pre-empt. Every distance here is a *proposal*.

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, Component, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PlaceId, SimDuration,
};
use mineworld_conversation::{Talk, Utterance};
use mineworld_movement::{MAX_STRIDE, Move, Passage, Passages};
use serde::Serialize;
use serde_json::Value;

use crate::{agenda, social};
use crate::{disclosed_history, may_talk_to, newest_per_speaker, reply_to};

/// How close an approach stops short of the person approached: near enough to talk, not on top of
/// them. A literal from the requirement — conversation's range is three metres — not derived from
/// anything this controller computes (`ARC-23` rule 2).
const APPROACH_STOPS_AT: i64 = 1_000;

/// Within this distance of somebody, approaching them again is pointless; the controller wanders.
const CLOSE_ENOUGH: i64 = 1_500;

/// The largest offset a wander proposes on each axis. `1 400² + 1 400²` is under `MAX_STRIDE²`, so a
/// wander is a legal stride by construction rather than by a clamp computed from the result.
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
/// and it holds for long enough to arrive: a door eighteen metres off is nine strides away, about
/// fifteen consults at the street's rate of walking on, under four hours at `mineworld run`'s pace.
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
    /// says I may speak to them; otherwise, by the seeded draw, greet somebody I may speak to,
    /// walk toward somebody, walk toward or through a doorway, wander a stride, or do nothing. In a
    /// place with several doorways the draw favours walking on toward one of them.
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
            && let Some(step) = self.head_for(observation, agenda.place())
        {
            return Some(step);
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
        // Heading for a door: never while part of an activity, never away from the agenda's place, and
        // — with an agenda elsewhere — only ever the door toward it, so a person on their way takes
        // no detour through somebody else's door. With no agenda, the seeded door, as before.
        let leave = |observation| match &mine {
            _ if member || at_agenda => None,
            Some(agenda) => self.head_for(observation, agenda.place()),
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
            leave(observation).or_else(|| wander(observation, &draw))
        } else if roll < APPROACHES_BELOW {
            approach(observation, &draw).or_else(|| wander(observation, &draw))
        } else if roll < LEAVES_BELOW {
            leave(observation)
        } else if roll < WANDERS_BELOW {
            wander(observation, &draw)
        } else {
            None
        }
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

    /// Toward a doorway this place discloses, or through it when it is a stride away.
    ///
    /// Which doorway is a draw over `(seed, observer, instant ÷ DOOR_WINDOW)`: the same door for the
    /// whole window, so a person crossing a street keeps walking to one door rather than turning at
    /// every consult, and a different door in the next window, so every place is visited. A place with
    /// one doorway has only one to choose. The passages are disclosed in `PlaceId` order, and before
    /// S8 this always took the first — on a street of five doors that sent everybody into the
    /// lowest-numbered place and never back (`step-09-social.md` F-6).
    fn leave(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
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
        through(observation, here, passage)
    }

    /// Toward `place`: through the doorway that leads there if this place discloses one, otherwise
    /// through the seeded door [`Self::leave`] would take — which in a star town is the way to the
    /// street, and from the street every place is one door away (`step-09-social.md` L-3). The route
    /// is never computed beyond "the disclosed door whose `to` is the place".
    pub(crate) fn head_for(
        &self,
        observation: &Observation<Value>,
        place: PlaceId,
    ) -> Option<ActionRequest> {
        let here = *observation.self_location()?;
        let leads_there = doorways(observation, here)?
            .iter()
            .copied()
            .find(|passage| passage.to() == place);
        match leads_there {
            Some(passage) => through(observation, here, passage),
            None => self.leave(observation),
        }
    }
}

/// A stride toward `passage`'s doorway, or through it when it is a stride away.
fn through(
    observation: &Observation<Value>,
    here: Location,
    passage: Passage,
) -> Option<ActionRequest> {
    let from = here.local()?;
    let door = passage.here()?;
    if within(from, door, i64::from(MAX_STRIDE.value())) {
        // Through: to the same doorway on the other side, which is within a stride of itself.
        let there = passage.there()?;
        return walk(
            observation,
            Location::in_place(passage.to()).with_local(there),
        );
    }
    walk(observation, here.with_local(toward(from, door, 0)?))
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

/// A stride toward somebody in the same place, stopping short of them; nothing if already close.
fn approach(observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
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
    let stride = toward(from, to, APPROACH_STOPS_AT)?;
    walk(observation, here.with_local(stride))
}

/// A stride in a seeded direction within the place.
fn wander(observation: &Observation<Value>, draw: &Draw) -> Option<ActionRequest> {
    let here = *observation.self_location()?;
    let from = here.local()?;
    let (dx, dy) = (draw.offset(WANDER_AXIS, 5), draw.offset(WANDER_AXIS, 6));
    if dx == 0 && dy == 0 {
        return None;
    }
    let to = LocalPosition::new(shifted(from.x(), dx)?, shifted(from.y(), dy)?, from.z());
    walk(observation, here.with_local(to))
}

/// A `move` request to `to`, if the server offers `move` to this observer at all.
fn walk(observation: &Observation<Value>, to: Location) -> Option<ActionRequest> {
    let offered = observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == Move::ACTION_TYPE && affordance.is_available()
    });
    offered.then(|| ActionRequest::new(observation.observer(), record(&Move::new(to))))
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

/// Whether `b` is at most `limit` from `a` on the floor, decided exactly on squared integers — never
/// on a rounded distance, which would call 2 000.4 mm "2 000" and propose a crossing the movement
/// system refuses.
pub(crate) fn within(a: LocalPosition, b: LocalPosition, limit: i64) -> bool {
    let (dx, dy) = delta(a, b);
    dx * dx + dy * dy <= limit * limit
}

/// Straight-line distance on the floor, in whole millimetres, rounded down.
pub(crate) fn distance(a: LocalPosition, b: LocalPosition) -> i64 {
    let (dx, dy) = delta(a, b);
    isqrt(dx * dx + dy * dy)
}

/// A stride from `from` toward `to`, ending `stop_short` short of it and never longer than
/// [`MAX_STRIDE`]; `None` when there is nowhere to go.
///
/// Floor division toward zero on each axis over the distance rounded **up**, so the stride's length
/// never exceeds the travel asked for: with `D ≥ √(dx² + dy²)`, `|dx·travel/D| ≤ |dx|·travel/D`, and
/// the stride's length is at most `√(dx² + dy²)·travel/D ≤ travel`.
///
/// Rounded up, not down. Dividing by the floored root — as this did until step-09 C3 — makes `D`
/// slightly *short* of the true distance and the stride a fraction of a millimetre *over* `travel`:
/// (1 640, 1 145) is 2 000.16 mm, which the movement system rightly refuses `TooFarAway`. The old test
/// measured strides with the same floored root and could not see it (`DECISIONS.md` `ARC-23`).
pub(crate) fn toward(
    from: LocalPosition,
    to: LocalPosition,
    stop_short: i64,
) -> Option<LocalPosition> {
    let (dx, dy) = delta(from, to);
    let d = isqrt_up(dx * dx + dy * dy);
    if d <= stop_short {
        return None;
    }
    let travel = (d - stop_short).min(i64::from(MAX_STRIDE.value()));
    let (sx, sy) = (dx * travel / d, dy * travel / d);
    if sx == 0 && sy == 0 {
        return None;
    }
    Some(LocalPosition::new(
        shifted(from.x(), sx)?,
        shifted(from.y(), sy)?,
        from.z(),
    ))
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

/// The integer square root of a non-negative number, rounded up: the least `r` with `r² ≥ value`.
fn isqrt_up(value: i64) -> i64 {
    let root = isqrt(value);
    if root * root < value { root + 1 } else { root }
}

/// This controller's own payload encoding, as [`RuleController`](crate::RuleController)'s.
fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(
        serde_json::to_vec(action)
            .expect("a request payload is JSON-representable by construction"),
    )
}
