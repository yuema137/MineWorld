//! `WorldRead::at()`: every read knows the world-time instant it is taken at (`DECISIONS.md` `ARC-86`,
//! S24 PR FG-a K1), at each of the places the kernel builds one:
//!
//! ```text
//! validate      the instant of the request being dispatched
//! resolve       the instant the call works in (WorldView::read)
//! react         the instant of the fact being reduced (WorldView::read)
//! World::read   the world's now(), after an advance
//! ```
//!
//! The systems here report what their read said, through a rejection's detail and through fact
//! payloads, so the test observes the value the system saw rather than a kernel internal.

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, EntityKey,
    EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, Rejection, RejectionCode,
    SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, WorldView,
};
use serde::{Deserialize, Serialize};

/// The system that answers `stamp`, and says what instant its reads were at.
struct Clerk;

impl SystemIdentity for Clerk {
    const ID: SystemId = SystemId::from_static("clerk");
}

/// The request: refused when `refuse`, so validation's reading comes back as the refusal's detail.
#[derive(Serialize, Deserialize)]
struct Stamp {
    refuse: bool,
}

impl Action for Stamp {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("stamp");
    const OWNER: SystemId = Clerk::ID;
}

/// What resolution's read said.
#[derive(Serialize, Deserialize)]
struct Stamped {
    read_at: i64,
}

impl Event for Stamped {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("stamped");
    const OWNER: SystemId = Clerk::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

const READ_AT: RejectionCode = RejectionCode::from_static("read-at");

impl System for Clerk {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .providing::<Stamp>()
            .emitting::<Stamped>()
    }

    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let stamp: Stamp =
            serde_json::from_slice(intent.payload().payload()).expect("the test's payload");
        if stamp.refuse {
            return Err(Rejection::System {
                code: READ_AT,
                detail: Some(world.at().seconds().to_string()),
            });
        }
        Ok(())
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        _intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read_at = world.read().at().seconds();
        Ok(vec![Emission::new::<Stamped>(
            serde_json::to_vec(&Stamped { read_at }).expect("encodes"),
            Visibility::SystemInternal,
        )])
    }
}

/// A subscriber that reports what its reaction's read said.
struct Witness;

impl SystemIdentity for Witness {
    const ID: SystemId = SystemId::from_static("witness");
}

#[derive(Serialize, Deserialize)]
struct Witnessed {
    read_at: i64,
}

impl Event for Witnessed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("witnessed");
    const OWNER: SystemId = Witness::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Witness {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .subscribing_to::<Stamped>()
            .emitting::<Witnessed>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let read_at = world.read().at().seconds();
        Ok(vec![Emission::new::<Witnessed>(
            serde_json::to_vec(&Witnessed { read_at }).expect("encodes"),
            Visibility::SystemInternal,
        )])
    }
}

fn world() -> (World, mineworld_contracts::EntityId) {
    let mut world = World::new();
    let actor = world
        .create_entity(
            EntityKey::new("alice").expect("a legal key"),
            EntityType::Person,
        )
        .expect("an entity is created");
    world.install(Clerk).expect("installs");
    world.install(Witness).expect("installs");
    (world, actor)
}

fn stamp(id: u64, actor: mineworld_contracts::EntityId, refuse: bool, at: i64) -> ActionIntent {
    ActionIntent::new(
        ActionId::from_raw(id),
        actor,
        ActionRecord::new::<Stamp>(serde_json::to_vec(&Stamp { refuse }).expect("encodes")),
        WorldTime::from_seconds(at),
    )
}

/// Validation reads at the instant of the request being dispatched.
#[test]
fn a_validation_read_is_at_the_dispatch_instant() {
    let (mut world, actor) = world();
    let dispatched = world
        .dispatch(
            &stamp(1, actor, true, 4_321),
            WorldTime::from_seconds(4_321),
        )
        .expect("dispatch answers");
    assert_eq!(
        *dispatched.result(),
        ActionResult::Rejected(Rejection::System {
            code: READ_AT,
            detail: Some("4321".to_owned()),
        })
    );
}

/// Resolution's read and a subscriber's reaction read are at the instant the fact is recorded.
#[test]
fn resolution_and_reaction_reads_are_at_the_facts_instant() {
    let (mut world, actor) = world();
    let dispatched = world
        .dispatch(
            &stamp(1, actor, false, 86_400),
            WorldTime::from_seconds(86_400),
        )
        .expect("dispatch answers");
    let read_ats: Vec<(String, i64)> = dispatched
        .events()
        .iter()
        .map(|event| {
            let value: serde_json::Value =
                serde_json::from_slice(event.payload().payload()).expect("json");
            (
                event.event_type().as_str().to_owned(),
                value["read_at"].as_i64().expect("an integer"),
            )
        })
        .collect();
    assert_eq!(
        read_ats,
        [
            ("stamped".to_owned(), 86_400),
            ("witnessed".to_owned(), 86_400)
        ]
    );
    assert!(
        dispatched
            .events()
            .iter()
            .all(|event| event.at().seconds() == 86_400)
    );
}

/// `World::read()` is at the world's `now()`: the epoch before anything, the advanced-to instant after.
#[test]
fn the_worlds_own_read_is_at_now() {
    let (mut world, _) = world();
    assert_eq!(world.read().at(), WorldTime::EPOCH);
    let _nothing_was_due = world
        .advance_to(WorldTime::from_seconds(7_200))
        .expect("advances");
    assert_eq!(world.now(), WorldTime::from_seconds(7_200));
    assert_eq!(world.read().at(), WorldTime::from_seconds(7_200));
}
