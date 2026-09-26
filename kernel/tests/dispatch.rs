//! The central pipeline, from outside the kernel: what a request is answered with, what the facts it
//! caused say about themselves, and in what order they reach the systems that reduce them.
//!
//! An external crate, so every system here is written the way a **System Pack** writes one: implement
//! [`System`], declare what you own, provide and emit, and never touch a store. The claims are §4.4's
//! C4 list:
//!
//! ```text
//! an action no enabled system provides is Unavailable, and no system is asked   — INV-10
//! a rejection comes back exactly as the system stated it
//! recorded facts name the intent as their cause                                 — AC-9, INV-15
//! reduction reaches subscribers in registration order, twice identically        — AC-12, BD-4
//! a two-system cycle errors naming both systems                                 — BD-7
//! a deferral comes back for the scheduler rather than happening now             — D-6
//! a fact outside a system's declaration is refused
//! ```
//!
//! None of these systems knows anything. One counts arrivals, one keeps a flag, one keeps a tally;
//! the kernel never learns what any of it means (`INV-12`).

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, Causation, EntityId,
    EntityKey, EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, Rejection,
    RejectionCode, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    CASCADE_DEPTH_LIMIT, Declarations, Dispatched, Emission, KernelError, System,
    SystemDeclaration, SystemIdentity, SystemVersion, World, WorldRead, WorldView, owned_component,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// The provider: one system that owns a count, answers for one action and emits one kind of fact.
// ---------------------------------------------------------------------------------------------

/// The system that answers for `"enter"`.
struct Bellringer;

impl SystemIdentity for Bellringer {
    const ID: SystemId = SystemId::from_static("bellringer");
}

/// The state `Bellringer` owns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Arrivals {
    count: u32,
}

owned_component! {
    component = Arrivals,
    owner = Bellringer,
    component_type = "arrivals",
    schema_version = 1,
}

/// The request it answers.
#[derive(Serialize, Deserialize)]
struct Enter {
    party: u32,
}

impl Action for Enter {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("enter");
    const OWNER: SystemId = Bellringer::ID;
}

/// The fact it emits.
#[derive(Serialize, Deserialize)]
struct Rung {
    times: u32,
}

impl Event for Rung {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rung");
    const OWNER: SystemId = Bellringer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Bellringer {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Arrivals>()
            .providing::<Enter>()
            .emitting::<Rung>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Arrivals>()
    }

    /// Never asked about an action it does not provide, and this assertion is what proves it: a
    /// dispatcher that consulted systems to discover who provides an action would trip it.
    fn validate(&self, _world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        assert_eq!(
            *intent.action_type(),
            Enter::ACTION_TYPE,
            "a system must never be consulted about an action it does not provide"
        );
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let count = world
            .read()
            .component::<Arrivals>(intent.actor())
            .map_or(1, |arrivals| arrivals.count + 1);
        world.insert(intent.actor(), Arrivals { count })?;
        Ok(vec![
            Emission::new::<Rung>(payload(&Rung { times: count }), Visibility::Public)
                .about(vec![intent.actor()]),
        ])
    }
}

// ---------------------------------------------------------------------------------------------
// Two subscribers. Each reacts by emitting a fact of its own, which is how reduction order becomes
// observable from outside: the recorded facts name their emitters, in the order they were recorded.
// ---------------------------------------------------------------------------------------------

/// The first subscriber, installed second and sorting last.
struct Watch;

impl SystemIdentity for Watch {
    const ID: SystemId = SystemId::from_static("watch");
}

#[derive(Serialize, Deserialize)]
struct Noted;

impl Event for Noted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("noted");
    const OWNER: SystemId = Watch::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Watch {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Noted>()
            .subscribing_to::<Rung>()
    }

    fn react(
        &self,
        _world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Noted>(
            payload(&Noted),
            Visibility::SystemInternal,
        )])
    }
}

/// The second subscriber, installed third and sorting first. Nothing subscribes to what either of
/// them emits, so reduction ends after one more generation.
struct Almanac;

impl SystemIdentity for Almanac {
    const ID: SystemId = SystemId::from_static("almanac");
}

#[derive(Serialize, Deserialize)]
struct Tallied;

impl Event for Tallied {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("tallied");
    const OWNER: SystemId = Almanac::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Almanac {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Tallied>()
            .subscribing_to::<Rung>()
    }

    fn react(
        &self,
        _world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Tallied>(
            payload(&Tallied),
            Visibility::SystemInternal,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// The systems the awkward cases need.
// ---------------------------------------------------------------------------------------------

/// A system that refuses everything, with a reason of its own.
struct Doorkeeper;

impl SystemIdentity for Doorkeeper {
    const ID: SystemId = SystemId::from_static("doorkeeper");
}

#[derive(Serialize, Deserialize)]
struct Knock;

impl Action for Knock {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("knock");
    const OWNER: SystemId = Doorkeeper::ID;
}

/// The reason it gives. A code of the system's own: the kernel must never learn what
/// `closed-for-the-night` means.
const CLOSED: RejectionCode = RejectionCode::from_static("closed-for-the-night");

impl System for Doorkeeper {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().providing::<Knock>()
    }

    fn validate(&self, _world: &WorldRead<'_>, _intent: &ActionIntent) -> Result<(), Rejection> {
        Err(Rejection::System {
            code: CLOSED,
            detail: Some("the last bell rang an hour ago".to_owned()),
        })
    }
}

/// A system that asks for a fact to happen later rather than now.
struct Postponer;

impl SystemIdentity for Postponer {
    const ID: SystemId = SystemId::from_static("postponer");
}

#[derive(Serialize, Deserialize)]
struct Later;

impl Action for Later {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("later");
    const OWNER: SystemId = Postponer::ID;
}

#[derive(Serialize, Deserialize)]
struct Elapsed;

impl Event for Elapsed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("elapsed");
    const OWNER: SystemId = Postponer::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The instant `Postponer` defers to, and the one dispatch is told to work in.
const NOW: WorldTime = WorldTime::from_seconds(100);
const LATER: WorldTime = WorldTime::from_seconds(160);

impl System for Postponer {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<Later>()
            .emitting::<Elapsed>()
            .subscribing_to::<Elapsed>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        _intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        world.defer(
            LATER,
            Emission::new::<Elapsed>(payload(&Elapsed), Visibility::SystemInternal),
        )?;
        Ok(Vec::new())
    }
}

/// A system that provides an action and leaves `resolve` at the trait default.
struct Mute;

impl SystemIdentity for Mute {
    const ID: SystemId = SystemId::from_static("mute");
}

#[derive(Serialize, Deserialize)]
struct Mumble;

impl Action for Mumble {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("mumble");
    const OWNER: SystemId = Mute::ID;
}

impl System for Mute {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().providing::<Mumble>()
    }
}

/// A system that emits a kind of fact its own declaration does not list.
struct Smuggler;

impl SystemIdentity for Smuggler {
    const ID: SystemId = SystemId::from_static("smuggler");
}

#[derive(Serialize, Deserialize)]
struct Smuggle;

impl Action for Smuggle {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("smuggle");
    const OWNER: SystemId = Smuggler::ID;
}

#[derive(Serialize, Deserialize)]
struct Undeclared;

impl Event for Undeclared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("undeclared");
    const OWNER: SystemId = Smuggler::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Smuggler {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().providing::<Smuggle>()
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        _intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Undeclared>(
            payload(&Undeclared),
            Visibility::SystemInternal,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// Two systems that react to each other, forever, inside one instant.
// ---------------------------------------------------------------------------------------------

/// The system that starts the loop.
struct Ping;

impl SystemIdentity for Ping {
    const ID: SystemId = SystemId::from_static("ping");
}

/// The system that answers it, which makes it start again.
struct Pong;

impl SystemIdentity for Pong {
    const ID: SystemId = SystemId::from_static("pong");
}

/// A bystander: enabled, installed between the two, and subscribed to nothing. It must not appear in
/// the cascade error, because it is not what will not stop.
struct Bystander;

impl SystemIdentity for Bystander {
    const ID: SystemId = SystemId::from_static("bystander");
}

#[derive(Serialize, Deserialize)]
struct Pinged;

impl Event for Pinged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("pinged");
    const OWNER: SystemId = Ping::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Ponged;

impl Event for Ponged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("ponged");
    const OWNER: SystemId = Pong::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Serialize, Deserialize)]
struct Start;

impl Action for Start {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("start");
    const OWNER: SystemId = Ping::ID;
}

impl System for Ping {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<Start>()
            .emitting::<Pinged>()
            .subscribing_to::<Ponged>()
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        _intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Pinged>(
            payload(&Pinged),
            Visibility::SystemInternal,
        )])
    }

    fn react(
        &self,
        _world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Pinged>(
            payload(&Pinged),
            Visibility::SystemInternal,
        )])
    }
}

impl System for Bystander {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl System for Pong {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .emitting::<Ponged>()
            .subscribing_to::<Pinged>()
    }

    fn react(
        &self,
        _world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        Ok(vec![Emission::new::<Ponged>(
            payload(&Ponged),
            Visibility::SystemInternal,
        )])
    }
}

// ---------------------------------------------------------------------------------------------
// Helpers.
// ---------------------------------------------------------------------------------------------

fn payload<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal authoring name")
}

/// One request, of whichever kind, by whoever.
fn intent<A: Action>(id: u64, actor: EntityId, request: &A) -> ActionIntent {
    ActionIntent::new(
        ActionId::from_raw(id),
        actor,
        ActionRecord::new::<A>(payload(request)),
        NOW,
    )
}

/// A world with an entity in it, and nothing else.
fn world_with_an_actor() -> (World, EntityId) {
    let mut world = World::new();
    let actor = world
        .create_entity(key("alice"), EntityType::Person)
        .expect("an entity is created");
    (world, actor)
}

/// Which system emitted each recorded fact, in the order the facts were recorded.
fn emitters(dispatched: &Dispatched) -> Vec<&SystemId> {
    dispatched
        .events()
        .iter()
        .map(|event| event.provenance().emitted_by())
        .collect()
}

// ---------------------------------------------------------------------------------------------
// The claims.
// ---------------------------------------------------------------------------------------------

/// An action no enabled system provides does not exist in that world, and saying so is an ordinary
/// answer rather than a failure (`INV-10`). No system is consulted to produce it: the route map is
/// the whole decision, and `Bellringer::validate` asserts that it is never asked about anything else.
#[test]
fn an_action_no_enabled_system_provides_is_unavailable() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Bellringer).expect("a system installs");

    let asked = intent(1, actor, &Knock);
    let dispatched = world.dispatch(&asked, NOW).expect("dispatch answers");

    assert_eq!(*dispatched.result(), ActionResult::Unavailable);
    assert!(dispatched.events().is_empty());
    assert!(dispatched.deferred().is_empty());
    // Nothing happened: the one installed system's state is untouched.
    assert_eq!(world.components().count::<Arrivals>(), 0);
}

/// A rejection is the owning system's judgement about this request, and it comes back exactly as the
/// system stated it — including a code the kernel does not understand.
#[test]
fn a_rejection_comes_back_exactly_as_the_system_stated_it() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Doorkeeper).expect("a system installs");

    let asked = intent(1, actor, &Knock);
    let dispatched = world.dispatch(&asked, NOW).expect("dispatch answers");

    assert_eq!(
        *dispatched.result(),
        ActionResult::Rejected(Rejection::System {
            code: CLOSED,
            detail: Some("the last bell rang an hour ago".to_owned()),
        })
    );
    // A rejected request leaves no debris, and the type is what guarantees it: `validate` is handed a
    // read-only view, so there is nothing for it to write through.
    assert!(dispatched.events().is_empty());
}

/// Resolution's facts name the request as their cause, and the emitting system as their author
/// (`AC-9`, `INV-15`). A system supplies the payload and the audience; identity, instant, causation
/// and provenance are the kernel's, so no system can misstate why a fact happened.
#[test]
fn recorded_facts_name_the_intent_that_caused_them() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Bellringer).expect("a system installs");

    let asked = intent(7, actor, &Enter { party: 2 });
    let dispatched = world.dispatch(&asked, NOW).expect("dispatch answers");

    let recorded = dispatched.events();
    assert_eq!(recorded.len(), 1, "nothing subscribes to what it emitted");
    let event = &recorded[0];
    assert_eq!(*event.caused_by(), Causation::Action(ActionId::from_raw(7)));
    assert_eq!(*event.provenance().emitted_by(), Bellringer::ID);
    assert_eq!(
        event.provenance().controller_decision(),
        Some(ActionId::from_raw(7))
    );
    assert_eq!(event.at(), NOW, "the instant is the one dispatch was told");
    assert_eq!(*event.event_type(), Rung::EVENT_TYPE);
    assert_eq!(event.subjects(), [actor]);
    assert_eq!(*event.visibility(), Visibility::Public);

    // The answer lists the identities of the facts, and the facts are the world's first.
    assert_eq!(
        *dispatched.result(),
        ActionResult::Accepted {
            events: vec![event.id()],
        }
    );
    assert_eq!(event.id().raw(), 1);

    // And the resolution's write landed, through a view gated on ownership.
    assert_eq!(
        world.components().get::<Arrivals>(actor),
        Some(&Arrivals { count: 1 })
    );
}

/// Reduction reaches every enabled subscriber, in registration order, and the order is observable:
/// each subscriber emits a fact of its own, so the recorded sequence names its emitters in the order
/// they reduced (`BD-4`, `AC-12`).
///
/// `almanac` is installed last and sorts first, which is what makes this test able to tell
/// registration order from name order.
#[test]
fn reduction_reaches_subscribers_in_registration_order_twice_identically() {
    let sequences: Vec<Vec<SystemId>> = (0..2)
        .map(|_| {
            let (mut world, actor) = world_with_an_actor();
            world.install(Bellringer).expect("a system installs");
            world.install(Watch).expect("a second system installs");
            world.install(Almanac).expect("a third system installs");

            let asked = intent(1, actor, &Enter { party: 1 });
            let dispatched = world.dispatch(&asked, NOW).expect("dispatch answers");

            // Resolution's fact first, then one generation of reaction in registration order.
            assert_eq!(dispatched.events().len(), 3);
            assert_eq!(
                *dispatched.events()[1].caused_by(),
                Causation::Event(dispatched.events()[0].id()),
                "a consequence names the fact it is a consequence of"
            );
            // Identity is allocated in recording order, and the answer lists every fact the request
            // caused — including the ones the reducers caused.
            assert_eq!(
                *dispatched.result(),
                ActionResult::Accepted {
                    events: vec![
                        dispatched.events()[0].id(),
                        dispatched.events()[1].id(),
                        dispatched.events()[2].id(),
                    ],
                }
            );
            emitters(&dispatched).into_iter().cloned().collect()
        })
        .collect();

    assert_eq!(
        sequences[0],
        vec![Bellringer::ID, Watch::ID, Almanac::ID],
        "registration order, not the order the names sort in"
    );
    assert_eq!(sequences[0], sequences[1], "and the same order every run");
}

/// Two systems reacting to each other never finish inside one instant, so the cascade limit stops it
/// and names them (`BD-7`). A world that froze silently would be worse than one that says which
/// systems were cycling.
#[test]
fn a_reduction_cycle_errors_naming_the_cycling_systems() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Ping).expect("a system installs");
    world.install(Bystander).expect("a second system installs");
    world.install(Pong).expect("a third system installs");

    let asked = intent(1, actor, &Start);
    let refusal = world
        .dispatch(&asked, NOW)
        .expect_err("a cascade that will not terminate is refused");

    assert_eq!(
        refusal,
        KernelError::ReductionCascadeTooDeep {
            limit: CASCADE_DEPTH_LIMIT,
            systems: vec![Ping::ID, Pong::ID],
        }
    );
    // The bystander is enabled and installed between the two, and is not named: the error names the
    // systems that emitted while reducing, not everything in the world.
    assert!(world.systems().is_enabled(&Bystander::ID));
}

/// A deferral is work for the scheduler, not a fact that has happened. It comes back in
/// [`Dispatched::deferred`] at the instant the system asked for, and nothing reduces it here — S4
/// owns the queue (`D-6`, `BD-7`).
#[test]
fn a_deferral_comes_back_for_the_scheduler_rather_than_happening_now() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Postponer).expect("a system installs");

    let asked = intent(1, actor, &Later);
    let dispatched = world.dispatch(&asked, NOW).expect("dispatch answers");

    assert_eq!(dispatched.deferred().len(), 1);
    assert_eq!(dispatched.deferred()[0].at(), LATER);
    assert_eq!(
        *dispatched.deferred()[0].emission().event_type(),
        Elapsed::EVENT_TYPE
    );
    // It is not history yet: nothing was recorded, and the system that subscribes to it never ran.
    assert!(dispatched.events().is_empty());
    assert_eq!(
        *dispatched.result(),
        ActionResult::Accepted { events: Vec::new() }
    );
}

/// A system that emits a kind of fact its own declaration does not list is refused. The declaration
/// is what a reader of a world's composition goes by — which facts this world can produce, and from
/// whom — so a fact outside it would make that reading wrong.
#[test]
fn a_fact_outside_a_systems_declaration_is_refused() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Smuggler).expect("a system installs");

    let asked = intent(1, actor, &Smuggle);
    let refusal = world
        .dispatch(&asked, NOW)
        .expect_err("an undeclared event type is refused");

    assert_eq!(
        refusal,
        KernelError::EventTypeNotInSystemDeclaration {
            system: Smuggler::ID,
            event_type: Undeclared::EVENT_TYPE,
        }
    );
}

/// A system that provides an action and does not resolve it is a bug in that system, and the trait's
/// default says so by name rather than answering `Accepted` with no facts — which would look exactly
/// like a world that worked.
#[test]
fn a_provider_that_does_not_resolve_is_refused_by_name() {
    let (mut world, actor) = world_with_an_actor();
    world.install(Mute).expect("a system installs");

    let asked = intent(1, actor, &Mumble);
    let refusal = world
        .dispatch(&asked, NOW)
        .expect_err("an unresolved action is refused");

    assert_eq!(
        refusal,
        KernelError::ActionNotResolvedBySystem {
            system: Mute::ID,
            action_type: Mumble::ACTION_TYPE,
        }
    );
}
