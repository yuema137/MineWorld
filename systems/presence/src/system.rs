//! The installable system: what it declares, what it installs, and how it reduces `arrived`. It
//! provides no action, so a world never asks it to validate or resolve one (`ARC-26`).

use mineworld_contracts::{
    EntityType, EntityTypeSet, Event, EventEnvelope, Rejection, Relation, RelationTypeDeclaration,
    RelationTypeId, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldView,
};
use mineworld_sdk::SystemPack;

use crate::codec;
use crate::component::Presence;
use crate::event::{Arrived, PersonEnteredPlace, admit, entered_place};
use crate::interaction::PerceptionProvider;

/// Where people are, and what each of them perceives.
///
/// A unit struct, like most System Packs: a system owns no fields, because its mutable state is the
/// components it owns and those live in the world behind a gated view (`INV-7`). That is also why
/// holding a second value of this type — to hand it to perception after a world has taken the first
/// one — is safe rather than merely convenient.
#[derive(Default)]
pub struct PresenceSystem;

impl SystemIdentity for PresenceSystem {
    const ID: SystemId = SystemId::from_static("presence");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing.
/// It declares no biographical fact and owns no authored section; a person's `location` is a field
/// of the World Pack format, bound to this pack by the loader (`ARC-31`, point 5).
impl SystemPack for PresenceSystem {}

/// The name of the edge this pack declares: a person is in a place.
///
/// A function rather than a constant because [`RelationTypeId`] is validated at construction and has
/// no `const` path; the literal is checked here, once, and every caller gets the same value.
pub fn present_in() -> RelationTypeId {
    RelationTypeId::new("present-in").expect("a legal relation type name")
}

/// What this pack declares about that edge: directed, from a person to a place.
///
/// The edge and the [`Presence`] component are the same fact at two resolutions, and both are
/// written in one place — [`PresenceSystem::react`] — so they cannot drift. Keeping both is
/// deliberate: the component carries the position a 3D client needs, while the edge is what makes
/// "who is in this place" a question the relation graph can answer, and what lets an
/// [`Observation`](mineworld_contracts::Observation) state containment as a relation rather than
/// leaving a client to infer it (`contracts/src/observation.rs`, `spike/FINDINGS.md` F3).
///
/// Only people, for now. An item on a table is a relation an inventory or interior pack will
/// declare about state it owns; widening this endpoint set to cover it here would be this pack
/// claiming that state.
pub fn present_in_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        present_in(),
        PresenceSystem::ID,
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Place]).expect("a non-empty set"),
    )
}

/// This pack refusing to let its state take a value, as the kernel's error for it.
///
/// Reached only when a fact is about to be built or reduced after the checks that should have
/// refused it earlier passed — so it is a defect in whoever stated the fact, never an answer to a
/// request, which is why it is a [`KernelError`] rather than a [`Rejection`] result.
fn refused(reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: PresenceSystem::ID,
        event_type: Arrived::EVENT_TYPE,
        reason,
    }
}

impl System for PresenceSystem {
    /// Version 2: no longer provides `arrive`, and states [`PersonEnteredPlace`] (`ARC-26`). A save
    /// written by version 1 is refused by name rather than resumed into a world that answers
    /// differently (S5).
    const VERSION: SystemVersion = SystemVersion::new(2);

    /// Owns where people are, and provides **no action**: who may move a person is another system's
    /// decision (`DECISIONS.md` `ARC-26`). A world places its people by genesis (`ARC-15`) and lets
    /// whatever system it installs for the purpose decide where they go; that system states this
    /// pack's `arrived` for this pack to reduce, and this pack never learns its name. So the only requests this
    /// system would ever be asked to validate do not exist, and the kernel's defaults for `validate`
    /// and `resolve` are never reached.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Presence>()
            .emitting::<Arrived>()
            .emitting::<PersonEnteredPlace>()
            .subscribing_to::<Arrived>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Presence>()?;
        tables.relation(present_in_declaration())
    }

    /// Reduces an arrival into the state this pack owns: the position, and the edge that says which
    /// place it is in — and, when the place changed, states that the person entered it.
    ///
    /// The old edge is removed when the place changes. Leaving it would make a person present in two
    /// places at once — a stale edge is not a harmless leftover, it is a false fact about the world,
    /// and it is exactly what a reader of the relation graph would believe.
    ///
    /// **Occupancy changes are this pack's facts.** A [`PersonEnteredPlace`] is stated here, caused by
    /// the `arrived`, whenever a person who was known to be in one place is now in another — whoever
    /// decided the move. Not on a first placement (genesis: nobody *entered*, they were there) and not
    /// within a place (nobody's occupancy changed).
    ///
    /// **The owner still decides** (`DECISIONS.md` `ARC-26`). Another system may state an `arrived`
    /// in this pack's vocabulary, so before writing anything the value is put to [`admit`] against
    /// the world as it is now. A refusal writes nothing and is returned as
    /// [`KernelError::FactRefusedByOwner`]: the stating system bypassed [`arrival`], which would have
    /// refused the same value before it was recorded, and the world says so rather than taking it.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Arrived::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let arrived: Arrived = codec::event_payload(event.payload())?;
        admit(&world.read(), arrived.person(), arrived.location()).map_err(refused)?;
        let person = arrived.person().entity_id();
        let arriving_at = arrived.location();

        let left = world
            .read()
            .component::<Presence>(person)
            .map(Presence::location)
            .filter(|previous| previous.place() != arriving_at.place());
        let mut entered = Vec::new();
        if let Some(left) = left {
            let edge = {
                let read = world.read();
                Relation::between(
                    &present_in_declaration(),
                    read.require_entity(person)?,
                    read.require_entity(left.place().entity_id())?,
                )?
            };
            world.unrelate(&edge)?;
            entered.push(entered_place(
                arrived.person(),
                arriving_at.place(),
                left.place(),
            ));
        }

        world.insert(person, Presence::at(arriving_at))?;
        world.relate(&present_in(), person, arriving_at.place().entity_id())?;
        Ok(entered)
    }
}

/// Presence offers no action and discloses no component: it answers perception's *questions* in
/// [`observe`](crate::observe()) rather than contributing offers of its own. Implemented — with the
/// trait's defaults — so that a world lists this pack among its providers like any other.
impl PerceptionProvider for PresenceSystem {}
