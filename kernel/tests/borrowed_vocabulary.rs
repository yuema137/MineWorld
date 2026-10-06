//! Stating a fact in another system's vocabulary (`docs/DECISIONS.md` `ARC-26`), from outside the
//! kernel.
//!
//! The rule, and the claims this file makes good:
//!
//! ```text
//! a system may declare that it emits a kind of fact another system owns only if it depends on
//!     that owner — otherwise installation is refused by name and the world is unchanged
//! with the dependency, the fact is recorded with the stating system as its provenance, while its
//!     type stays the owner's — and the owner alone reduces it into the state it owns
//! a system's own vocabulary needs no dependency on itself
//! ```
//!
//! `Ledger` owns a tally and the fact that changes it; `Clerk` decides when a count happens and
//! states `Ledger`'s fact. The names are arbitrary: the kernel never learns what a tally is
//! (`INV-12`).

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, EntityId, EntityKey,
    EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId, SystemId, Visibility,
    WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    World, WorldView, owned_component,
};
use serde::{Deserialize, Serialize};

const NOW: WorldTime = WorldTime::from_seconds(60);

/// Owns the tally, and the vocabulary that changes it. Provides no action of its own.
struct Ledger;

impl SystemIdentity for Ledger {
    const ID: SystemId = SystemId::from_static("ledger");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Tally {
    count: u32,
}

owned_component! {
    component = Tally,
    owner = Ledger,
    component_type = "tally",
    schema_version = 1,
}

/// `Ledger`'s fact: the subject's tally is now `count`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Counted {
    count: u32,
}

impl Event for Counted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("counted");
    const OWNER: SystemId = Ledger::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// `Ledger`'s public constructor for its own fact — what a stating system is required to use.
fn counted(subject: EntityId, count: u32) -> Emission {
    Emission::new::<Counted>(
        serde_json::to_vec(&Counted { count }).expect("encodes"),
        Visibility::Public,
    )
    .about(vec![subject])
}

impl System for Ledger {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Tally>()
            .emitting::<Counted>()
            .subscribing_to::<Counted>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Tally>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Counted::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let fact: Counted = serde_json::from_slice(event.payload().payload()).expect("own payload");
        world.insert(event.subjects()[0], Tally { count: fact.count })?;
        Ok(Vec::new())
    }
}

/// The request a clerk answers.
#[derive(Serialize, Deserialize)]
struct Count {
    to: u32,
}

impl Action for Count {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("count");
    const OWNER: SystemId = Clerk::ID;
}

/// Decides counts, owns nothing, and states `Ledger`'s fact — declaring the dependency.
struct Clerk;

impl SystemIdentity for Clerk {
    const ID: SystemId = SystemId::from_static("clerk");
}

impl System for Clerk {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([Ledger::ID])
            .providing::<Count>()
            .emitting::<Counted>()
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let bytes = intent.payload().payload_for::<Count>().expect("a count");
        let count: Count = serde_json::from_slice(bytes).expect("a count payload");
        Ok(vec![counted(intent.actor(), count.to)])
    }
}

/// The same clerk, without the dependency: what installation has to refuse.
struct UndeclaredClerk;

impl SystemIdentity for UndeclaredClerk {
    const ID: SystemId = SystemId::from_static("undeclared-clerk");
}

impl System for UndeclaredClerk {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>().emitting::<Counted>()
    }
}

fn snapshot_bytes(world: &World) -> Vec<u8> {
    serde_json::to_vec(&world.snapshot().expect("a snapshot")).expect("encodes")
}

/// Without a dependency on the owner, declaring another system's vocabulary is refused at
/// installation, naming all three — even though the owner happens to be installed, because the
/// declaration is wrong in any world. Nothing changes.
#[test]
fn borrowing_a_vocabulary_without_depending_on_its_owner_is_refused_by_name() {
    let mut world = World::new();
    world.install(Ledger).expect("the owner installs");
    let before = snapshot_bytes(&world);

    let refusal = world
        .install(UndeclaredClerk)
        .expect_err("a borrowed vocabulary needs a dependency on its owner");

    assert_eq!(
        refusal,
        KernelError::EmittedEventOwnerNotADependency {
            system: UndeclaredClerk::ID,
            event_type: Counted::EVENT_TYPE,
            owner: Ledger::ID,
        }
    );
    assert_eq!(world.systems().len(), 1, "only the owner is installed");
    assert_eq!(snapshot_bytes(&world), before, "a refusal changes nothing");
}

/// With the dependency the borrowing system installs, and its fact is recorded with **its** name as
/// provenance and the **owner's** type — the two things `ARC-26` keeps apart — while only the owner
/// writes the state. The owner's own emission of the same type needs no dependency on itself.
#[test]
fn a_borrowed_fact_names_who_stated_it_and_is_reduced_by_its_owner() {
    let mut world = World::new();
    world.install(Ledger).expect("the owner installs");
    world
        .install(Clerk)
        .expect("a system that depends on the owner may state its vocabulary");
    let till = world
        .create_entity(EntityKey::new("till").expect("a key"), EntityType::Item)
        .expect("an entity");

    let intent = ActionIntent::new(
        ActionId::from_raw(1),
        till,
        ActionRecord::new::<Count>(serde_json::to_vec(&Count { to: 7 }).expect("encodes")),
        NOW,
    );
    let dispatched = world.dispatch(&intent, NOW).expect("dispatch answers");

    assert!(matches!(dispatched.result(), ActionResult::Accepted { .. }));
    assert_eq!(
        dispatched.events().len(),
        1,
        "one fact, located before reading it"
    );
    let fact = &dispatched.events()[0];
    assert_eq!(*fact.event_type(), Counted::EVENT_TYPE);
    assert_eq!(
        *fact.provenance().emitted_by(),
        Clerk::ID,
        "provenance names the system that stated the fact, not the vocabulary's owner"
    );
    assert_eq!(
        world.components().get::<Tally>(till),
        Some(&Tally { count: 7 }),
        "and the owner reduced it into the state only it writes"
    );
}
