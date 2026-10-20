//! The choices a consult offers, and what each one submits (`pr-dl-b-motivated.md` §5.2).
//!
//! Every choice is built from something the observation offers: an available affordance, a disclosed
//! doorway, a perceived person the server says may be addressed, the observer's own agenda or activity
//! (I-DLb-3). Nothing here computes a server verdict. The context is read once per consult, from the
//! observation alone (I-DLb-4: every input is optional, and an absent one is neutral).

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionRequest, Component, EntityId, LocalPosition, Location, Observation,
    PerceivedEntity, PersonId, PlaceId, SimDuration, Tag,
};
use mineworld_conversation::{Talk, Utterance};
use mineworld_group_activity::{
    ActivityKind, Invite, JoinGroupActivity, LeaveGroupActivity, Participation,
};
use mineworld_movement::{Destination, Passages, WalkTo};
use mineworld_relationships::Acquaintances;
use mineworld_rule_controller::commute::{self, Commute, Mode};
use mineworld_rule_controller::{act, boss, read};
use mineworld_schedule::Agenda;
use mineworld_weather::{component::WeatherNow, day::Condition};
use serde_json::Value;

use crate::settings::Settings;

/// The kinds an invitation names, in the order a draw picks among them (`rule`'s `KINDS`).
pub const KINDS: [&str; 3] = ["coffee", "chat", "walk"];

/// The largest offset a wander names on each axis, in millimetres (`rule`'s `WANDER_AXIS`).
pub const WANDER_AXIS: i64 = 1_400;

/// Within this distance of somebody, walking to them again is pointless (`rule`'s `CLOSE_ENOUGH`).
pub const CLOSE_ENOUGH: i64 = 1_500;

/// Seconds in an hour.
const HOUR: i64 = 3_600;

/// One thing a person could do at this consult.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Choice {
    /// Head for the agenda's place.
    Agenda,
    /// Do nothing.
    Stay,
    /// Say hello to somebody I may speak to.
    Greet(PersonId),
    /// Walk to somebody in my place.
    Approach(PersonId),
    /// Invite somebody to an activity.
    Invite(PersonId),
    /// Join somebody's activity.
    Join(PersonId),
    /// Leave my activity.
    Leave,
    /// Walk through the doorway to this place.
    Door(PlaceId),
    /// Attempt the complete affordance at this index of the observation's affordances.
    Offered(usize),
    /// Wander to a point nearby.
    Wander,
    /// Set out for the punctual next part of my day, at this place (`ARC-282`).
    Commute(PlaceId),
    /// Penalize the late member of staff the complete `penalize` at this index of the observation's
    /// affordances names (`ARC-287`).
    Penalize(usize),
}

/// A punctual next part of my day I am not at yet, and how urgent setting out for it is (§0.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pending {
    /// Where it is.
    pub place: PlaceId,
    /// Per mille: 0 while waiting two more paces could not make me late, 1 000 from the instant I
    /// should have set out, linear between.
    pub urgency: u32,
}

impl Pending {
    /// The pending commute of `observation` by `commute`, at a host consulting every `pace` and striding
    /// every `stride`; nothing with no punctual next part ahead of me. In `at-boundary` mode the urgency
    /// stays 0: the person sets out when the part begins, by the agenda.
    pub fn of(
        observation: &Observation<Value>,
        commute: &Commute,
        pace: SimDuration,
        stride: SimDuration,
    ) -> Option<Self> {
        let next = commute::pending(observation)?;
        if commute.mode == Mode::AtBoundary {
            return Some(Self {
                place: next.place,
                urgency: 0,
            });
        }
        let leave_by = commute.leave_by(observation, &next, stride);
        let now = observation.at().seconds();
        let window = 2 * pace.seconds().max(1);
        let urgency = if now + window <= leave_by {
            0
        } else if now >= leave_by {
            1_000
        } else {
            1_000 * (now + window - leave_by) / window
        };
        Some(Self {
            place: next.place,
            urgency: u32::try_from(urgency.clamp(0, 1_000)).unwrap_or(1_000),
        })
    }

    /// Whether setting out has begun to matter.
    pub const fn pressing(&self) -> bool {
        self.urgency > 0
    }
}

/// A doorway this place discloses, and the tags of the place it leads to (the `to_tags` disclosure, ARC-82).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Door {
    /// The place the doorway leads to.
    pub to: PlaceId,
    /// The tags of that place, as the observation discloses them.
    pub tags: Vec<Tag>,
}

/// Everything a consult reads, gathered once from the observation.
pub struct Context<'a> {
    /// The observation the consult is decided from.
    pub observation: &'a Observation<Value>,
    /// The observer.
    pub me: EntityId,
    /// The instant, in whole seconds.
    pub now: i64,
    /// The hour of the day, 0 … 23 (no calendar is read; see F-DLb-3).
    pub hour: i64,
    /// Where the observer stands, if perception discloses it.
    pub here: Option<Location>,
    /// The tags of the place the observer stands in.
    pub here_tags: Vec<Tag>,
    /// The observer's agenda, if disclosed.
    pub agenda: Option<Agenda>,
    /// Whether the observer stands at the agenda's place.
    pub at_agenda: bool,
    /// The observer's activity, if they are part of one.
    pub activity: Option<Participation>,
    /// The weather at the observer's place, if disclosed.
    pub weather: Option<Condition>,
    /// The observer's acquaintances, if disclosed.
    pub acquaintances: Option<Acquaintances>,
    /// Hours since the observer's newest contact with anybody, if any is known.
    pub last_contact_hours: Option<i64>,
    /// The doorways of the place the observer stands in.
    pub doors: Vec<Door>,
    /// A punctual next part I am not at yet, set by the consult from its host context (`Pending::of`);
    /// [`None`] when there is none.
    pub commute: Option<Pending>,
    /// The `penalize` offers I may take as a manager: each one's affordance index and the lateness
    /// count it is for (`ARC-287`). Empty for everybody else.
    pub penalties: Vec<(usize, u32)>,
}

impl<'a> Context<'a> {
    /// The context of `observation`, or nothing when the observer is not in it.
    pub fn new(observation: &'a Observation<Value>) -> Option<Self> {
        let me = observation.observer();
        let mine = observation.entity(me)?;
        let at = observation.at();
        let now = at.seconds();
        let hour = now.div_euclid(HOUR).rem_euclid(24);
        let here = observation.self_location().copied();
        let here_entity =
            here.and_then(|location| observation.entity(location.place().entity_id()));
        let here_tags = here_entity
            .map(|entity| entity.tags().iter().cloned().collect())
            .unwrap_or_default();
        let agenda: Option<Agenda> = read::disclosed(mine);
        let at_agenda = match (&agenda, here) {
            (Some(agenda), Some(location)) => location.place() == agenda.place(),
            _ => false,
        };
        let acquaintances: Option<Acquaintances> = read::disclosed(mine);
        let last_contact_hours = last_contact_hours(&acquaintances, mine, now);
        Some(Self {
            observation,
            me,
            now,
            hour,
            here,
            here_tags,
            agenda,
            at_agenda,
            activity: read::disclosed(mine),
            weather: here_entity
                .and_then(read::disclosed::<WeatherNow>)
                .map(|now| now.condition),
            acquaintances,
            last_contact_hours,
            doors: here
                .map(|location| doors(observation, location))
                .unwrap_or_default(),
            commute: None,
            penalties: boss::offered(observation)
                .iter()
                .map(|offered| (offered.index, offered.late.count))
                .collect(),
        })
    }

    /// The acquaintance record of `person`, if I hold one.
    pub fn knows(&self, person: PersonId) -> Option<&mineworld_relationships::RelationshipValues> {
        self.acquaintances.as_ref()?.of(person)
    }
}

/// Hours since the newest contact: the newest `last_contact` over my acquaintances, else the newest line
/// I heard (`social.rs`'s pseudo-need), else nothing.
fn last_contact_hours(
    acquaintances: &Option<Acquaintances>,
    mine: &PerceivedEntity<Value>,
    now: i64,
) -> Option<i64> {
    let from_relationships = acquaintances.as_ref().and_then(|held| {
        held.known()
            .iter()
            .map(|entry| entry.values().last_contact().seconds())
            .max()
    });
    let newest = from_relationships.or_else(|| {
        read::disclosed_history(mine)?
            .heard()
            .iter()
            .map(|heard| heard.at().seconds())
            .max()
    })?;
    Some((now - newest).max(0).div_euclid(HOUR))
}

/// The doorways of the place `here`, each with the tags of the place it leads to.
fn doors(observation: &Observation<Value>, here: Location) -> Vec<Door> {
    let Some(passages) = read::doorways(observation, here) else {
        return Vec::new();
    };
    let tags = disclosed_destination_tags(observation, here);
    passages
        .iter()
        .map(|passage| Door {
            to: passage.to(),
            tags: tags.get(&passage.to()).cloned().unwrap_or_default(),
        })
        .collect()
}

/// The `to_tags` of each doorway, from the disclosed passages payload. Read here because `to_tags` is a
/// disclosure only: movement never stores it, and no typed record carries it.
fn disclosed_destination_tags(
    observation: &Observation<Value>,
    here: Location,
) -> BTreeMap<PlaceId, Vec<Tag>> {
    let mut tags = BTreeMap::new();
    let Some(place) = observation.entity(here.place().entity_id()) else {
        return tags;
    };
    let Some(payload) = place
        .components()
        .iter()
        .find(|record| *record.component_type() == Passages::COMPONENT_TYPE)
        .and_then(|record| record.payload_for::<Passages>().ok())
    else {
        return tags;
    };
    let Some(leads) = payload.get("leads_to").and_then(Value::as_array) else {
        return tags;
    };
    for entry in leads {
        let Some(to) = entry
            .get("to")
            .and_then(|value| serde_json::from_value::<PlaceId>(value.clone()).ok())
        else {
            continue;
        };
        let names: Vec<Tag> = entry
            .get("to_tags")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(Value::as_str)
                    .filter_map(|name| Tag::new(name).ok())
                    .collect()
            })
            .unwrap_or_default();
        tags.insert(to, names);
    }
    tags
}

/// Whether the server offers `walk-to` to me at all.
fn walk_offered(observation: &Observation<Value>) -> bool {
    observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == WalkTo::ACTION_TYPE && affordance.is_available()
    })
}

/// Every choice the observation offers, in a fixed order: the agenda, stay, then the people and the activity
/// options in listed order, the doorways, the complete affordances and the wander.
pub fn choices(context: &Context<'_>) -> Vec<Choice> {
    let observation = context.observation;
    let mut choices = Vec::new();
    // Heading back to the part in force would undo a commute that has begun to matter, so the agenda is
    // not offered while one is pressing (`ARC-282`).
    let pressing = context.commute.is_some_and(|pending| pending.pressing());
    if context.agenda.is_some() && !context.at_agenda && !pressing {
        choices.push(Choice::Agenda);
    }
    if let Some(pending) = context.commute {
        choices.push(Choice::Commute(pending.place));
    }
    choices.push(Choice::Stay);
    let walk = walk_offered(observation);
    let others: Vec<&PerceivedEntity<Value>> = read::others(observation).collect();
    for entity in &others {
        let Some(person) = PersonId::new(entity.id(), entity.entity_type()).ok() else {
            continue;
        };
        if read::may_talk_to(observation, entity.id()) {
            choices.push(Choice::Greet(person));
        }
        if let (true, Some(here), Some(at)) = (walk, context.here, entity.location())
            && at.place() == here.place()
            && let (Some(from), Some(to)) = (here.local(), at.local())
            && read::distance(from, to) > CLOSE_ENOUGH
        {
            choices.push(Choice::Approach(person));
        }
    }
    for person in read::people_offering::<Invite>(observation) {
        if let Some(person) = person_of(observation, person) {
            choices.push(Choice::Invite(person));
        }
    }
    for person in read::people_offering::<JoinGroupActivity>(observation) {
        if let Some(person) = person_of(observation, person) {
            choices.push(Choice::Join(person));
        }
    }
    if read::available::<LeaveGroupActivity>(observation, None) {
        choices.push(Choice::Leave);
    }
    let in_activity = context.activity.is_some();
    let away_from_agenda = context.agenda.is_some() && !context.at_agenda;
    if !in_activity && !away_from_agenda && !pressing {
        for door in &context.doors {
            choices.push(Choice::Door(door.to));
        }
    }
    for (index, affordance) in observation.affordances().iter().enumerate() {
        if affordance.is_available()
            && affordance.payload().is_some()
            && !boss::is_penalty(affordance)
        {
            choices.push(Choice::Offered(index));
        }
    }
    for (index, _) in &context.penalties {
        choices.push(Choice::Penalize(*index));
    }
    if walk && context.here.and_then(|here| here.local()).is_some() {
        choices.push(Choice::Wander);
    }
    choices
}

/// The person a perceived entity id names, when the observation lists them as a person.
fn person_of(observation: &Observation<Value>, entity: EntityId) -> Option<PersonId> {
    let perceived = observation.entity(entity)?;
    PersonId::new(perceived.id(), perceived.entity_type()).ok()
}

/// The request a chosen `choice` submits, or nothing (`Stay` submits nothing, by design).
///
/// `greeting` and `kind` are the draws the caller has already taken for the wording and the activity kind;
/// `wander` is the offset pair. Each is used by one choice only.
pub fn request(
    choice: &Choice,
    context: &Context<'_>,
    greeting: &str,
    kind: &str,
    wander_offset: (i64, i64),
) -> Option<ActionRequest> {
    let me = context.me;
    let observation = context.observation;
    match choice {
        Choice::Agenda => {
            let agenda = context.agenda.as_ref()?;
            act::head_for(observation, agenda.place())?.request()
        }
        Choice::Stay => None,
        Choice::Greet(person) => {
            let said = Utterance::new(greeting).ok()?;
            Some(
                ActionRequest::new(me, act::record(&Talk::new(said)))
                    .with_target(person.entity_id()),
            )
        }
        Choice::Approach(person) => {
            act::walk_to(observation, Destination::Person(*person))?.request()
        }
        Choice::Invite(person) => {
            let kind = ActivityKind::new(kind).ok()?;
            Some(act::request(me, &Invite::new(kind)).with_target(person.entity_id()))
        }
        Choice::Join(person) => {
            Some(act::request(me, &JoinGroupActivity {}).with_target(person.entity_id()))
        }
        Choice::Leave => Some(act::request(me, &LeaveGroupActivity {})),
        Choice::Door(place) | Choice::Commute(place) => {
            act::head_for(observation, *place)?.request()
        }
        Choice::Offered(index) | Choice::Penalize(index) => {
            let affordance = observation.affordances().get(*index)?;
            affordance.request(me, encoded)
        }
        Choice::Wander => {
            let here = context.here?;
            let from = here.local()?;
            let (dx, dy) = wander_offset;
            if dx == 0 && dy == 0 {
                return None;
            }
            let to = LocalPosition::new(shifted(from.x(), dx)?, shifted(from.y(), dy)?, from.z());
            act::walk_to(observation, Destination::Place(here.with_local(to)))?.request()
        }
    }
}

/// `value` moved by `by` millimetres, if it stays in range.
fn shifted(
    value: mineworld_contracts::Millimetres,
    by: i64,
) -> Option<mineworld_contracts::Millimetres> {
    i32::try_from(i64::from(value.value()) + by)
        .ok()
        .map(mineworld_contracts::Millimetres::new)
}

/// This controller's payload encoding for an offered affordance, as `rule`'s `offered.rs`.
fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}

/// The base weight of a choice, from the settings.
pub fn base(choice: &Choice, settings: &Settings) -> u32 {
    let weights = &settings.base;
    match choice {
        Choice::Agenda => weights.agenda,
        Choice::Stay => weights.stay,
        Choice::Greet(_) => weights.greet,
        Choice::Approach(_) => weights.approach,
        Choice::Invite(_) => weights.invite,
        Choice::Join(_) => weights.join,
        Choice::Leave => weights.leave,
        Choice::Door(_) => weights.door,
        Choice::Offered(_) => weights.offered,
        Choice::Wander => weights.wander,
        Choice::Commute(_) => weights.commute,
        Choice::Penalize(_) => weights.penalize,
    }
}
