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
//! —        a due commute takes no draw                                       commute.rs
//! ```
//!
//! The doorway choice is not a draw index: it is a separate mix over `(seed, observer, instant ÷ six
//! hours)`, so that a door is kept long enough to arrive at.

use mineworld_contracts::{
    Action, ActionRequest, ActionTypeId, EntityId, LocalPosition, Millimetres, Observation,
    PerceivedEntity, PersonId, PlaceId, SimDuration,
};
use mineworld_conversation::{Talk, Utterance};
use mineworld_group_activity::{
    AcceptInvitation, DeclineInvitation, Invite, JoinGroupActivity, LeaveGroupActivity,
};
// The seeded draws, moved verbatim to the trait crate every model shares (DL-a SD-DLa-2).
use mineworld_decision::{Draw, mix};
use mineworld_movement::{Destination, Passage, WalkStep, WalkTo};
use serde_json::Value;

use crate::act::{GREETINGS, Walked, head_for, record, reply_to, walk_to};
use crate::boss::Boss;
use crate::commute::{Commute, Precedence};
use crate::read::{
    disclosed_history, distance, doorways, in_activity, may_talk_to, newest_per_speaker, others,
};
use crate::{agenda, offered, social};

/// Within this distance of somebody, approaching them again is pointless; the controller wanders.
/// A walk to a person ends within movement's `PERSON_APPROACH` (1 200 mm) of them, inside this.
const CLOSE_ENOUGH: i64 = 1_500;

/// The largest offset a wander names on each axis: a point a stride or two away. The walk there is
/// movement's, which moves a goal that lies in furniture or off the floor to the nearest free point.
const WANDER_AXIS: i64 = 1_400;

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

/// Every action type this rule builds a request for by name: its own bands decide them, so the generic
/// offered band leaves them out of its draw (`offered.rs`, `DECISIONS.md` `ARC-151`). A band that comes to
/// decide another action by name adds its type here.
pub(crate) const DECIDED_BY_NAME: [ActionTypeId; 8] = [
    Talk::ACTION_TYPE,
    WalkTo::ACTION_TYPE,
    WalkStep::ACTION_TYPE,
    Invite::ACTION_TYPE,
    AcceptInvitation::ACTION_TYPE,
    DeclineInvitation::ACTION_TYPE,
    JoinGroupActivity::ACTION_TYPE,
    LeaveGroupActivity::ACTION_TYPE,
];

/// A Person whose actions are a seeded rule, consulted every `pace` simulated seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacedRuleController {
    seed: u64,
    pace: SimDuration,
    /// The commute policy and the host's stride cadence; [`None`] plans no commute (`commute.rs`).
    commute: Option<(Commute, SimDuration)>,
    /// The boss policy, for a manager offered `penalize`; [`None`] penalizes nobody (`boss.rs`).
    boss: Option<Boss>,
}

impl PacedRuleController {
    /// A controller with this seed, consulted every `pace` seconds by whoever drives it, planning no
    /// commute: it sets out for every part of its day when the part begins.
    ///
    /// The pace is the controller's to know because its answering window is one pace long; a driver
    /// that consulted it at another rate would make it answer lines twice or never.
    pub const fn new(seed: u64, pace: SimDuration) -> Self {
        Self {
            seed,
            pace,
            commute: None,
            boss: None,
        }
    }

    /// The same controller, setting out for a punctual part of its day by `commute`, with the host
    /// asking for a stride every `stride` world seconds (`ARC-282`).
    #[must_use]
    pub const fn commuting(mut self, commute: Commute, stride: SimDuration) -> Self {
        self.commute = Some((commute, stride));
        self
    }

    /// The same controller, penalizing late staff by `boss` when it manages any (`ARC-287`).
    #[must_use]
    pub const fn bossing(mut self, boss: Boss) -> Self {
        self.boss = Some(boss);
        self
    }

    /// The place a due commute heads for at this consult, if any (`commute.rs`).
    fn due(&self, observation: &Observation<Value>) -> Option<PlaceId> {
        let (commute, stride) = self.commute?;
        commute.due(observation, self.pace, stride)
    }

    /// The precedence of a due commute among the hard rungs.
    fn precedence(&self) -> Precedence {
        self.commute
            .map_or(Precedence::First, |(commute, _)| commute.precedence)
    }

    /// A due commute's request: a `walk-to` its place, asked with no draw; when my walk already goes
    /// there, only an in-window line may be answered between strides, and nothing redirects the walk.
    fn commute(
        &self,
        observation: &Observation<Value>,
        draw: &Draw,
        place: PlaceId,
    ) -> Option<Option<ActionRequest>> {
        match head_for(observation, place)? {
            Walked::Ask(request) => Some(Some(request)),
            Walked::Continue => Some(self.answer(observation, draw)),
        }
    }

    /// The pace this controller expects to be consulted at.
    pub const fn pace(&self) -> SimDuration {
        self.pace
    }

    /// What this Person attempts at the instant of `observation`, or nothing.
    ///
    /// A due commute (`commute.rs`) comes first, or after the two answers when the policy says
    /// `after-replies`: it asks `walk-to` the next part's place, or, when my walk already goes there,
    /// at most answers a line between strides. Otherwise,
    /// in order: answer the newest line somebody said to me since my last consult, if the server
    /// says I may speak to them; then the agenda and the social initiative; then, sometimes, a
    /// complete affordance the world offers; otherwise, by the seeded draw, greet somebody I may
    /// speak to, walk to somebody, walk through a doorway, wander to a point nearby, or do nothing.
    /// In a place with several doorways the draw favours walking on through one of them. A walk my own
    /// disclosed walk already makes is not asked again: that consult asks for nothing.
    pub fn decide(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
        let draw = Draw::new(self.seed, observation);
        // A due commute, by the policy's precedence: first, or after being addressed (`commute.rs`).
        // Not due — no punctual next part, or not yet — nothing here changes and no draw is taken.
        let due = self.due(observation);
        if let Some(place) = due.filter(|_| self.precedence() == Precedence::First)
            && let Some(decided) = self.commute(observation, &draw, place)
        {
            return decided;
        }
        // An invitation waiting for an answer first, then a line waiting for a reply: being addressed
        // comes before taking initiative (`social.rs`).
        if let Some(answer) = social::answer_invitation(observation, &draw) {
            return Some(answer);
        }
        if let Some(reply) = self.answer(observation, &draw) {
            return Some(reply);
        }
        if let Some(place) = due
            && let Some(decided) = self.commute(observation, &draw, place)
        {
            return decided;
        }
        // A manager offered `penalize` decides by the boss policy, with no draw (`boss.rs`). Nobody
        // else is offered one, so for everybody else nothing here changes.
        if let Some(penalty) = self.boss.and_then(|boss| boss.decide(observation)) {
            return Some(penalty);
        }
        // Then the day: away from where my agenda says, usually head there; at it, stay — no doorway
        // (`agenda.rs`). No agenda disclosed, nothing here changes and no draw is taken.
        let mine = agenda::own(observation);
        // Arrived early at a punctual next part's place, I wait there: it counts as being where my day
        // says (`commute.rs`).
        let waiting = self
            .commute
            .is_some_and(|(commute, _)| crate::commute::holds(observation, &commute, self.pace));
        let at_agenda = waiting
            || mine
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
        let member = in_activity(observation);
        let social = if member {
            social::maybe_leave(observation, &draw)
        } else {
            social::initiative(observation, &draw)
        };
        if social.is_some() {
            return social;
        }
        // What the world offers complete — an action of a pack this controller was never compiled
        // against — sometimes (`offered.rs`, `ARC-34`). Nothing complete offered, nothing changes; what
        // the bands above decide by name is not drawn a second time (`ARC-151`).
        if let Some(offered) = offered::attempt(observation, &draw, &DECIDED_BY_NAME) {
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
        crate::act::step(observation)
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

/// One of `choices`, by the `n`th draw.
fn pick<'a, T>(choices: &'a [T], draw: &Draw, n: u64) -> Option<&'a T> {
    if choices.is_empty() {
        return None;
    }
    choices.get(usize::try_from(draw.below(choices.len() as u64, n)).ok()?)
}

fn shifted(value: Millimetres, by: i64) -> Option<Millimetres> {
    i32::try_from(i64::from(value.value()) + by)
        .ok()
        .map(Millimetres::new)
}
