//! **CP-3: a headless person attempts an action of a pack the controller has never been compiled
//! against** (step-10 §2.3, C-5; `docs/DECISIONS.md` `ARC-34`).
//!
//! `chimes` is a System Pack defined only in this file, so no library — the controller least of all —
//! is compiled against it. It provides `ring { bell }` and states its own `rang { ringer, bell }`. It
//! offers one **complete** `ring` per bell its belfry hangs, required to be at the belfry, so the same
//! offers are available in the belfry and unavailable, with the reason, anywhere else; and one
//! incomplete offer beside them, which no controller that does not know `ring` can complete.
//!
//! The world is built by hand — a belfry and a hall, two people in each, presence and chimes — and
//! driven exactly as `mineworld run` drives a World Pack (`ARC-27`): seat *k* is consulted at
//! `genesis + k + m·P`, with P = 900 s; at each consult the world advances, presence's real `observe`
//! builds the observation from every provider, `PacedRuleController::decide` decides, the request is
//! allocated an identity and an instant, and the kernel dispatches it.

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionResult, ActionTypeId, Causation, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, Location, PersonId, PlaceId,
    Rejection, SimDuration, SpatialRequirement, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem, arrival, observe};
use mineworld_rule_controller::PacedRuleController;
use serde::{Deserialize, Serialize};

/// `mineworld run`'s pace (`tools/cli/src/run.rs` `PACE`).
const PACE: i64 = 900;
const DAY: i64 = 86_400;
const DAYS: i64 = 10;
const SEED: u64 = 7;

/// The bells the belfry hangs: the bounded choice that lets chimes offer every request it would take.
const BELLS: [&str; 2] = ["low", "high"];

// ---------------------------------------------------------------------------------------------
// chimes: the synthetic System Pack
// ---------------------------------------------------------------------------------------------

/// The pack. Its one piece of configuration is where its belfry is; it owns no component, because
/// ringing a bell changes nothing but the record that it was rung.
#[derive(Clone, Copy)]
struct Chimes {
    belfry: PlaceId,
}

impl SystemIdentity for Chimes {
    const ID: SystemId = SystemId::from_static("chimes");
}

/// The request: ring this bell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Ring {
    bell: String,
}

impl Action for Ring {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("ring");
    const OWNER: SystemId = Chimes::ID;
}

/// The fact: this person rang this bell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Rang {
    ringer: EntityId,
    bell: String,
}

impl Event for Rang {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rang");
    const OWNER: SystemId = Chimes::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Chimes {
    /// Where a ring may be attempted from: the belfry, and nowhere else.
    const fn requirement(self) -> SpatialRequirement {
        SpatialRequirement::at_place(self.belfry)
    }
}

impl System for Chimes {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .providing::<Ring>()
            .emitting::<Rang>()
    }

    /// The pack's own rule, the same one its offers declare: at the belfry, a bell it hangs.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let ring: Ring = serde_json::from_slice(
            intent
                .payload()
                .payload_for::<Ring>()
                .map_err(|_| Rejection::PreconditionFailed)?,
        )
        .map_err(|_| Rejection::PreconditionFailed)?;
        if !BELLS.contains(&ring.bell.as_str()) {
            return Err(Rejection::PreconditionFailed);
        }
        let here = world
            .component::<Presence>(intent.actor())
            .map(Presence::location)
            .ok_or(Rejection::PreconditionFailed)?;
        self.requirement().evaluate(&here, None, true)
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let ring: Ring = serde_json::from_slice(intent.payload().payload())
            .expect("validated before it is resolved");
        let rang = Rang {
            ringer: intent.actor(),
            bell: ring.bell,
        };
        Ok(vec![
            Emission::new::<Rang>(
                serde_json::to_vec(&rang).expect("a fact encodes"),
                Visibility::Place(self.belfry),
            )
            .about(vec![intent.actor()]),
        ])
    }
}

impl PerceptionProvider for Chimes {
    fn offers(
        &self,
        _world: &WorldRead<'_>,
        _observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let mut offers: Vec<Offer> = BELLS
            .iter()
            .map(|bell| {
                Offer::complete(
                    &Ring {
                        bell: (*bell).to_owned(),
                    },
                    self.requirement(),
                )
                .expect("a ring encodes")
            })
            .collect();
        // The incomplete offer: a requester that does not know `ring` cannot complete it.
        offers.push(Offer::new::<Ring>(self.requirement()));
        offers
    }
}

// ---------------------------------------------------------------------------------------------
// The world, and the run
// ---------------------------------------------------------------------------------------------

/// What one run produced.
struct Run {
    /// Every fact, encoded, in the order recorded — the bytes two runs are compared by.
    facts: Vec<u8>,
    /// Every request made: (action id, actor, action type, payload as JSON, accepted).
    requests: Vec<(ActionId, EntityId, ActionTypeId, serde_json::Value, bool)>,
    /// Every `rang` fact recorded.
    rang: Vec<EventEnvelope>,
    /// The four people, belfry first.
    in_belfry: [EntityId; 2],
    in_hall: [EntityId; 2],
}

fn person(world: &mut World, key: &str) -> EntityId {
    world
        .create_entity(EntityKey::new(key).expect("a key"), EntityType::Person)
        .expect("a person")
}

fn encode(fact: &EventEnvelope, into: &mut Vec<u8>) {
    into.extend(serde_json::to_vec(fact).expect("a fact encodes"));
    into.push(b'\n');
}

/// Builds the world and drives it for `DAYS`, as `mineworld run` would; `chimes_enabled: false`
/// disables the pack before the first consult.
fn run(chimes_enabled: bool) -> Run {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    let belfry = PlaceId::new(
        world
            .create_entity(EntityKey::new("belfry").unwrap(), EntityType::Place)
            .unwrap(),
        EntityType::Place,
    )
    .unwrap();
    let hall = PlaceId::new(
        world
            .create_entity(EntityKey::new("hall").unwrap(), EntityType::Place)
            .unwrap(),
        EntityType::Place,
    )
    .unwrap();
    let chimes = Chimes { belfry };
    world.install(chimes).expect("chimes installs");
    let in_belfry = [person(&mut world, "ann"), person(&mut world, "ben")];
    let in_hall = [person(&mut world, "cal"), person(&mut world, "dee")];

    let genesis = WorldTime::EPOCH;
    let placed: Vec<Emission> = in_belfry
        .iter()
        .map(|who| (*who, belfry))
        .chain(in_hall.iter().map(|who| (*who, hall)))
        .map(|(who, place)| {
            arrival(
                &world.read(),
                PersonId::new(who, EntityType::Person).unwrap(),
                Location::in_place(place),
            )
            .expect("presence admits the placement")
        })
        .collect();
    let mut facts = Vec::new();
    for fact in world.genesis(genesis, placed).expect("genesis") {
        encode(&fact, &mut facts);
    }
    if !chimes_enabled {
        world
            .disable(&Chimes::ID)
            .expect("nothing depends on chimes");
    }

    let seats: Vec<EntityId> = in_belfry.iter().chain(&in_hall).copied().collect();
    let controller = PacedRuleController::new(SEED, SimDuration::from_seconds(PACE));
    let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &chimes];
    let end = genesis.seconds() + DAYS * DAY;
    let mut next_action = 1;
    let mut requests = Vec::new();
    let mut rang = Vec::new();
    'rounds: for round in 0.. {
        for (k, seat) in seats.iter().enumerate() {
            let at = genesis.seconds() + i64::try_from(k).unwrap() + round * PACE;
            if at >= end {
                break 'rounds;
            }
            let now = WorldTime::from_seconds(at);
            for fact in world.advance_to(now).expect("advance").into_events() {
                encode(&fact, &mut facts);
            }
            let seen = observe(&world, *seat, now, &providers);
            let Some(request) = controller.decide(&seen) else {
                continue;
            };
            let action_type = request.action_type().clone();
            let payload: serde_json::Value = serde_json::from_slice(request.payload().payload())
                .expect("the controller encodes JSON");
            let id = ActionId::from_raw(next_action);
            next_action += 1;
            let intent = ActionIntent::allocate(request, id, now);
            let dispatched = world.dispatch(&intent, now).expect("dispatch answers");
            let accepted = matches!(dispatched.result(), ActionResult::Accepted { .. });
            requests.push((id, *seat, action_type, payload, accepted));
            for fact in dispatched.events() {
                encode(fact, &mut facts);
                if *fact.event_type() == Rang::EVENT_TYPE {
                    rang.push(fact.clone());
                }
            }
        }
    }
    Run {
        facts,
        requests,
        rang,
        in_belfry,
        in_hall,
    }
}

// ---------------------------------------------------------------------------------------------
// The checkpoint
// ---------------------------------------------------------------------------------------------

/// People in the belfry ring every simulated day; people elsewhere never ring; nothing but a complete
/// offer is ever attempted; and every `rang` is caused by the request that asked for it (`AC-9`).
#[test]
fn a_headless_person_rings_a_bell_the_controller_never_heard_of() {
    let run = run(true);

    let mut rang_by_day: BTreeMap<(EntityId, i64), u64> = BTreeMap::new();
    for fact in &run.rang {
        let rang: Rang =
            serde_json::from_slice(fact.payload().payload_for::<Rang>().expect("labelled rang"))
                .expect("a rang payload");
        let Causation::Action(cause) = fact.caused_by() else {
            panic!("a rang caused by something other than a request: {fact:?}");
        };
        let asked = run
            .requests
            .iter()
            .find(|(id, ..)| id == cause)
            .expect("the cause is a request this run made");
        assert_eq!(asked.1, rang.ringer, "rung by whoever asked");
        assert_eq!(asked.2, Ring::ACTION_TYPE);
        assert_eq!(
            asked.3,
            serde_json::json!({ "bell": rang.bell }),
            "the bell asked for"
        );
        assert!(asked.4, "a request that caused a fact was accepted");
        *rang_by_day
            .entry((rang.ringer, fact.at().seconds() / DAY))
            .or_default() += 1;
    }

    for ringer in run.in_belfry {
        for day in 0..DAYS {
            assert!(
                rang_by_day.get(&(ringer, day)).copied().unwrap_or(0) > 0,
                "{ringer} in the belfry rang on day {day}: {rang_by_day:?}"
            );
        }
    }
    for elsewhere in run.in_hall {
        assert!(
            run.requests.iter().all(|(_, who, ..)| *who != elsewhere),
            "{elsewhere} is offered only unavailable rings and attempts none"
        );
    }
    let rings = run
        .requests
        .iter()
        .filter(|(_, _, action_type, ..)| *action_type == Ring::ACTION_TYPE)
        .count();
    assert_eq!(
        rings,
        run.rang.len(),
        "every ring asked for was a complete offer and was accepted"
    );
    assert_eq!(
        rings,
        run.requests.len(),
        "nothing else is offered here, and nothing else is attempted"
    );
}

/// The same seed twice: the same facts, byte for byte (`AC-12`).
#[test]
fn two_runs_of_one_seed_are_byte_identical() {
    let (first, second) = (run(true), run(true));
    assert!(
        !first.rang.is_empty(),
        "the comparison is of runs that rang"
    );
    assert!(first.facts == second.facts, "the two runs' facts differ");
}

/// With chimes disabled, its offers are gone from every observation and nobody asks to ring
/// (`INV-10`, `AC-2`).
#[test]
fn with_chimes_disabled_no_ring_is_ever_requested() {
    let run = run(false);
    assert!(
        run.requests
            .iter()
            .all(|(_, _, action_type, ..)| *action_type != Ring::ACTION_TYPE),
        "a ring was requested from a world without chimes"
    );
    assert!(run.rang.is_empty());
}

/// The controller's manifest names no `chimes`: it was never compiled against the pack it rings.
#[test]
fn the_controller_was_never_compiled_against_chimes() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../cognition/rule-controller/Cargo.toml");
    let text = std::fs::read_to_string(&manifest).expect("the controller's manifest is readable");
    assert!(
        text.contains("mineworld-contracts"),
        "this is the controller's manifest"
    );
    assert!(!text.contains("chimes"), "{text}");
}
