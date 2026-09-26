//! A hand-made world of five entities, and the rules that govern it.
//!
//! Every rule in this file is a *server* rule. There is no kernel here — the kernel is being
//! written on another branch — so this module plays the parts the kernel and a handful of System
//! Packs would play: it holds authoritative state, it computes an `Observation` per observer, it
//! decides whether an `ActionIntent` is allowed, and it records `Event`s. The clients decide
//! nothing.
//!
//! # The identities are chosen to break a naive client
//!
//! `PLACE`, `ALICE` and `MUG` are above 2^53, which is where an IEEE-754 double stops being able
//! to count. As doubles, `9007199254740995` (Alice) and `9007199254740997` (the mug) are both
//! `9007199254740996`: two distinct entities collapse into one. That is the `DD-15` trap, made
//! unmissable rather than theoretical.

use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, ActionTypeId, Affordance, Causation, ComponentRecord,
    EntityId, EntityType, Event, EventEnvelope, EventId, EventRecord, LocalPosition, Location,
    Millidegrees, Millimetres, Orientation, PerceivedEntity, PerceivedEvent, PlaceId, Provenance,
    Rejection, SpatialRequirement, Tag, Tags, Visibility, WorldTime,
};
use serde_json::Value;

use crate::vocabulary::{
    Body, ConversationStarted, DisplayName, MoveTo, Moved, PickUp, PlaceExtent, Signage, Talk,
};

/// `2^53 + 1`. A double rounds it to `9007199254740992`.
pub const PLACE: u64 = 9_007_199_254_740_993;
/// A small id, for contrast: this one survives a double unharmed.
pub const PLAYER: u64 = 101;
/// `2^53 + 3`. A double rounds it to `9007199254740996`.
pub const ALICE: u64 = 9_007_199_254_740_995;
/// A small id.
pub const BOB: u64 = 103;
/// `2^53 + 5`. A double also rounds it to `9007199254740996` — the same value as Alice.
pub const MUG: u64 = 9_007_199_254_740_997;

/// How far the authoritative body walks per tick: 2.5 m/s at 10 Hz.
const STEP_MM: i32 = 250;
/// The reach the conversation system declares for `talk`.
const TALK_REACH_MM: i32 = 2_500;
/// The reach the inventory system declares for `pick-up`.
const REACH_MM: i32 = 1_200;

/// One inhabitant or object the world knows about.
struct Thing {
    id: EntityId,
    entity_type: EntityType,
    name: &'static str,
    tags: &'static [&'static str],
    body: Option<Body>,
    position: LocalPosition,
    facing: Orientation,
    /// Whether the conversation system considers this thing available to be acted upon.
    available: bool,
}

/// The whole world.
pub struct SpikeWorld {
    now: WorldTime,
    place: PlaceId,
    extent: PlaceExtent,
    things: Vec<Thing>,
    /// Where the authoritative player body is walking to, and which request asked for it.
    goal: Option<(LocalPosition, ActionId)>,
    next_event: u64,
    log: Vec<EventEnvelope<Value>>,
}

fn millimetres(x: i32, y: i32, z: i32) -> LocalPosition {
    LocalPosition::new(
        Millimetres::new(x),
        Millimetres::new(y),
        Millimetres::new(z),
    )
}

/// Yaw as a compass bearing in millidegrees: `0` faces `+y`, `90_000` faces `+x`.
///
/// This convention is the spike's, not the contract's. `Orientation` fixes the *range* of yaw
/// and nothing about which way `0` points or which way the angle turns (`FINDINGS.md` F5).
fn bearing(millidegrees: i32) -> Orientation {
    Orientation::facing(Millidegrees::new(millidegrees))
}

fn tags(labels: &[&str]) -> Tags {
    Tags::new(labels.iter().map(|label| Tag::new(*label).expect("legal tag")))
}

impl SpikeWorld {
    /// The world as authored, and as every run restarts it.
    pub fn new() -> Self {
        let place = PlaceId::new(EntityId::from_raw(PLACE), EntityType::Place)
            .expect("the place entity is a place");
        Self {
            now: WorldTime::from_seconds(32_400),
            place,
            extent: PlaceExtent {
                min: millimetres(-6_000, -4_000, 0),
                max: millimetres(6_000, 4_000, 3_000),
            },
            things: vec![
                Thing {
                    id: EntityId::from_raw(PLAYER),
                    entity_type: EntityType::Person,
                    name: "You",
                    tags: &["player"],
                    body: Some(Body {
                        height_mm: 1_800,
                        radius_mm: 300,
                    }),
                    position: millimetres(0, 3_200, 0),
                    facing: bearing(180_000),
                    available: true,
                },
                Thing {
                    id: EntityId::from_raw(ALICE),
                    entity_type: EntityType::Person,
                    name: "Alice",
                    tags: &["barista", "staff"],
                    body: Some(Body {
                        height_mm: 1_680,
                        radius_mm: 280,
                    }),
                    position: millimetres(1_500, -2_200, 0),
                    facing: bearing(0),
                    available: true,
                },
                Thing {
                    id: EntityId::from_raw(BOB),
                    entity_type: EntityType::Person,
                    name: "Bob",
                    tags: &["regular"],
                    body: Some(Body {
                        height_mm: 1_750,
                        radius_mm: 300,
                    }),
                    position: millimetres(-4_200, 2_400, 0),
                    facing: bearing(90_000),
                    // Bob is engaged with something. The server says so; no client works it out.
                    available: false,
                },
                Thing {
                    id: EntityId::from_raw(MUG),
                    entity_type: EntityType::Item,
                    name: "Chipped mug",
                    tags: &["crockery"],
                    body: Some(Body {
                        height_mm: 110,
                        radius_mm: 45,
                    }),
                    position: millimetres(1_600, -2_600, 950),
                    facing: bearing(0),
                    available: true,
                },
            ],
            goal: None,
            next_event: 9_007_199_254_741_101,
            log: Vec::new(),
        }
    }

    /// Puts the player back at the door, so every client run starts from the same state.
    pub fn reset_player(&mut self) {
        let start = millimetres(0, 3_200, 0);
        let thing = self.thing_mut(EntityId::from_raw(PLAYER));
        thing.position = start;
        thing.facing = bearing(180_000);
        self.goal = None;
        self.log.clear();
    }

    fn thing(&self, id: EntityId) -> &Thing {
        self.things
            .iter()
            .find(|thing| thing.id == id)
            .expect("the world knows this entity")
    }

    fn thing_mut(&mut self, id: EntityId) -> &mut Thing {
        self.things
            .iter_mut()
            .find(|thing| thing.id == id)
            .expect("the world knows this entity")
    }

    fn location_of(&self, id: EntityId) -> Location {
        let thing = self.thing(id);
        Location::in_place(self.place)
            .with_local(thing.position)
            .with_facing(thing.facing)
    }

    /// Advances the clock and walks the authoritative player body toward its goal.
    ///
    /// Movement is the *server's*. A client asks to be somewhere; this decides how fast the body
    /// gets there. Neither client moves anything.
    pub fn tick(&mut self) {
        self.now = WorldTime::from_seconds(self.now.seconds() + 1);
        let Some((goal, action)) = self.goal else {
            return;
        };
        let player = self.thing(EntityId::from_raw(PLAYER));
        let dx = i64::from(goal.x().value() - player.position.x().value());
        let dy = i64::from(goal.y().value() - player.position.y().value());
        let distance = ((dx * dx + dy * dy) as f64).sqrt() as i64;

        if distance <= i64::from(STEP_MM) {
            self.thing_mut(EntityId::from_raw(PLAYER)).position = goal;
            self.goal = None;
            self.record::<Moved>(
                Moved { to: goal },
                Causation::Action(action),
                crate::vocabulary::MOVEMENT,
                Some(action),
            );
            return;
        }
        let step_x = dx * i64::from(STEP_MM) / distance;
        let step_y = dy * i64::from(STEP_MM) / distance;
        let player = self.thing_mut(EntityId::from_raw(PLAYER));
        player.position = LocalPosition::new(
            Millimetres::new(player.position.x().value() + step_x as i32),
            Millimetres::new(player.position.y().value() + step_y as i32),
            player.position.z(),
        );
    }

    fn record<E: Event>(
        &mut self,
        payload: E,
        cause: Causation,
        emitter: mineworld_contracts::SystemId,
        decision: Option<ActionId>,
    ) -> EventId {
        let id = EventId::from_raw(self.next_event);
        self.next_event += 1;
        let mut provenance = Provenance::new(emitter);
        if let Some(action) = decision {
            provenance = provenance.from_controller_decision(action);
        }
        let record = EventRecord::new::<E>(
            serde_json::to_value(payload).expect("a spike payload serializes"),
        );
        let envelope = EventEnvelope::new(
            id,
            self.now,
            record,
            cause,
            Visibility::Place(self.place),
            provenance,
        )
        .about(vec![EntityId::from_raw(PLAYER)])
        .at_place(self.place);
        self.log.push(envelope);
        id
    }

    // ---------------------------------------------------------------------------------------
    // The System Packs' declared spatial requirements. Data, declared once, on the server.
    // ---------------------------------------------------------------------------------------

    fn talk_requirement(&self) -> SpatialRequirement {
        SpatialRequirement::same_place()
            .within(Millimetres::new(TALK_REACH_MM))
            .expect("a positive reach")
            .requiring_target_available()
    }

    fn pick_up_requirement(&self) -> SpatialRequirement {
        SpatialRequirement::same_place()
            .within(Millimetres::new(REACH_MM))
            .expect("a positive reach")
    }

    // ---------------------------------------------------------------------------------------
    // Observation
    // ---------------------------------------------------------------------------------------

    fn perceived(&self, thing: &Thing) -> PerceivedEntity<Value> {
        let mut components = vec![component::<DisplayName>(
            thing.id,
            DisplayName {
                name: thing.name.to_owned(),
            },
        )];
        if let Some(body) = &thing.body {
            components.push(component::<Body>(thing.id, body.clone()));
        }
        PerceivedEntity::new(thing.id, thing.entity_type)
            .at(self.location_of(thing.id))
            .with_tags(tags(thing.tags))
            .with_components(components)
    }

    /// The place itself, as an entity, so that a client can learn its shape.
    ///
    /// It has no `Location` of its own: a place's position in a larger world is not something
    /// `Location` can say about a place (`FINDINGS.md` F3).
    fn perceived_place(&self) -> PerceivedEntity<Value> {
        let id = self.place.entity_id();
        PerceivedEntity::new(id, EntityType::Place)
            .with_tags(tags(&["cafe", "interior"]))
            .with_components(vec![
                component::<DisplayName>(
                    id,
                    DisplayName {
                        name: "Lakeside Cafe".to_owned(),
                    },
                ),
                component::<PlaceExtent>(id, self.extent.clone()),
                component::<Signage>(
                    id,
                    Signage {
                        text: "Lakeside Cafe — open".to_owned(),
                        // Both above 2^53 and neither is an entity id. A wire encoder that
                        // descended into this payload would corrupt them.
                        catalogue_id: 9_007_199_254_740_999,
                        id: 9_007_199_254_741_001,
                    },
                ),
            ])
    }

    /// What the player may attempt, with the server's answer for each.
    ///
    /// Every answer comes from `SpatialRequirement::evaluate`, the contract's one implementation
    /// of the check, run here against the server's own authoritative locations.
    fn affordances(&self) -> Vec<Affordance> {
        let actor = self.location_of(EntityId::from_raw(PLAYER));
        let mut affordances = vec![Affordance::available(
            ActionTypeId::from_static("move-to"),
            None,
            SpatialRequirement::NONE,
        )];

        for thing in &self.things {
            if thing.id.raw() == PLAYER {
                continue;
            }
            let (action, requirement) = match thing.entity_type {
                EntityType::Person => (ActionTypeId::from_static("talk"), self.talk_requirement()),
                EntityType::Item => (
                    ActionTypeId::from_static("pick-up"),
                    self.pick_up_requirement(),
                ),
                _ => continue,
            };
            let target = self.location_of(thing.id);
            affordances.push(
                match requirement.evaluate(&actor, Some(&target), thing.available) {
                    Ok(()) => Affordance::available(action, Some(thing.id), requirement),
                    Err(reason) => {
                        Affordance::unavailable(action, Some(thing.id), requirement, reason)
                    }
                },
            );
        }
        affordances
    }

    /// Everything the player is shown, and nothing else.
    pub fn observation(&self) -> mineworld_contracts::Observation<Value> {
        let mut entities = vec![self.perceived_place()];
        entities.extend(self.things.iter().map(|thing| self.perceived(thing)));

        mineworld_contracts::Observation::new(EntityId::from_raw(PLAYER), self.now)
            .at_location(self.location_of(EntityId::from_raw(PLAYER)))
            .perceiving(entities)
            .with_events(
                self.log
                    .iter()
                    .rev()
                    .take(6)
                    .rev()
                    .cloned()
                    .map(PerceivedEvent::new)
                    .collect(),
            )
            .offering(self.affordances())
    }

    // ---------------------------------------------------------------------------------------
    // Dispatch
    // ---------------------------------------------------------------------------------------

    /// Resolves a submitted request. The one place a world rule runs.
    ///
    /// `intent.actor_location()` is read only for the log: the requirement is evaluated against
    /// the server's own authoritative position, per `ENGINEERING_RULES.md` §8.
    pub fn resolve(&mut self, intent: &ActionIntent<Value>) -> ActionResult {
        let actor = self.location_of(EntityId::from_raw(PLAYER));
        match intent.action_type().as_str() {
            "move-to" => {
                let Ok(payload) = intent.payload().payload_for::<MoveTo>() else {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                };
                let Ok(request) = serde_json::from_value::<MoveTo>(payload.clone()) else {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                };
                if !self.inside(request.to) {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                }
                if let Some(facing) = request.facing {
                    self.thing_mut(EntityId::from_raw(PLAYER)).facing = facing;
                }
                self.goal = Some((request.to, intent.action_id()));
                ActionResult::Accepted { events: Vec::new() }
            }
            "talk" => {
                let Some(target) = intent.target() else {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                };
                if intent.payload().payload_for::<Talk>().is_err() {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                }
                let Some(thing) = self.things.iter().find(|thing| thing.id == target) else {
                    return ActionResult::Rejected(Rejection::NoSupportedInteraction);
                };
                if thing.entity_type != EntityType::Person {
                    return ActionResult::Rejected(Rejection::NoSupportedInteraction);
                }
                let available = thing.available;
                let name = thing.name.to_owned();
                let target_location = self.location_of(target);
                match self
                    .talk_requirement()
                    .evaluate(&actor, Some(&target_location), available)
                {
                    Ok(()) => {
                        let event = self.record::<ConversationStarted>(
                            ConversationStarted {
                                greeting: format!("{name} looks up."),
                            },
                            Causation::Action(intent.action_id()),
                            crate::vocabulary::CONVERSATION,
                            Some(intent.action_id()),
                        );
                        ActionResult::Accepted {
                            events: vec![event],
                        }
                    }
                    Err(reason) => ActionResult::Rejected(reason),
                }
            }
            "pick-up" => {
                let Some(target) = intent.target() else {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                };
                if intent.payload().payload_for::<PickUp>().is_err() {
                    return ActionResult::Rejected(Rejection::PreconditionFailed);
                }
                let target_location = self.location_of(target);
                match self
                    .pick_up_requirement()
                    .evaluate(&actor, Some(&target_location), true)
                {
                    Ok(()) => ActionResult::Accepted { events: Vec::new() },
                    Err(reason) => ActionResult::Rejected(reason),
                }
            }
            // No enabled system provides this action type, so it does not exist here (INV-10).
            _ => ActionResult::Unavailable,
        }
    }

    fn inside(&self, position: LocalPosition) -> bool {
        position.x() >= self.extent.min.x()
            && position.x() <= self.extent.max.x()
            && position.y() >= self.extent.min.y()
            && position.y() <= self.extent.max.y()
    }
}

fn component<C: mineworld_contracts::Component>(
    entity: EntityId,
    value: C,
) -> ComponentRecord<Value> {
    ComponentRecord::new::<C>(
        entity,
        serde_json::to_value(value).expect("a spike component serializes"),
    )
}
