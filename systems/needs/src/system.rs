//! The installable system: what it declares, the facts it reduces, and what it discloses.

use mineworld_contracts::{ComponentRecord, EntityId, Event, EventEnvelope, SystemId, WorldTime};
use mineworld_inventory::ItemsConsumed;
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions;
use mineworld_sleep::{FellAsleep, Woke};
use serde_json::Value;

use crate::codec;
use crate::component::{Needs, Nourishment};
use crate::disclose::disclosed;
use crate::event::{NeedsAssigned, NourishmentAssigned};
use crate::interactions::{parameters, rates};
use crate::level::Mode;

/// Hunger and tiredness.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NeedsSystem;

impl SystemIdentity for NeedsSystem {
    const ID: SystemId = SystemId::from_static("needs");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): the
/// `needs:` section it owns (`ARC-31`) and its section of the World's Interaction List (`ARC-63`),
/// which is its configuration. No fact is biographical.
impl SystemPack for NeedsSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
    mineworld_sdk::interactions!();
}

impl System for NeedsSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on nothing in the registry: it hears inventory's `items-consumed` and sleep's
    /// `fell-asleep` and `woke` through their published types (`ARC-28`). No Process: decay is lazy.
    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .owning::<Needs>()
                .owning::<Nourishment>()
                .emitting::<NeedsAssigned>()
                .emitting::<NourishmentAssigned>()
                .subscribing_to::<NeedsAssigned>()
                .subscribing_to::<NourishmentAssigned>()
                .subscribing_to::<ItemsConsumed>()
                .subscribing_to::<FellAsleep>()
                .subscribing_to::<Woke>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<Needs>()?;
        tables.component::<Nourishment>()
    }

    /// The only writes of this pack's state (SD-Nb-6); each folds first, at the fact's instant:
    ///
    /// - `needs-assigned` inserts [`Needs`] at the fact's instant, awake; `nourishment-assigned`
    ///   inserts [`Nourishment`] on the item kind;
    /// - `items-consumed` by a person with needs, when `eating` restores hunger, lowers hunger by the
    ///   kind's nourishment × count; a kind with none, or with 0, writes nothing;
    /// - `fell-asleep` and `woke` bring both needs to the instant and turn their mode.
    ///
    /// A fact about anybody without [`Needs`] writes nothing. States nothing.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let kind = event.event_type();
        let at = event.at();
        if *kind == NeedsAssigned::EVENT_TYPE {
            let assigned: NeedsAssigned = codec::event_payload(event.payload())?;
            let levels = assigned.levels();
            world.insert(
                assigned.person().entity_id(),
                Needs::assigned(levels.hunger, levels.tiredness, at),
            )?;
        } else if *kind == NourishmentAssigned::EVENT_TYPE {
            let assigned: NourishmentAssigned = codec::event_payload(event.payload())?;
            world.insert(
                assigned.item().entity_id(),
                Nourishment::new(assigned.hunger()),
            )?;
        } else if *kind == ItemsConsumed::EVENT_TYPE {
            let consumed: ItemsConsumed = codec::event_payload(event.payload())?;
            let fed = fed(&world.read(), &consumed, at);
            if let Some(needs) = fed {
                world.insert(consumed.holder(), needs)?;
            }
        } else if *kind == FellAsleep::EVENT_TYPE {
            let fell: FellAsleep = codec::event_payload(event.payload())?;
            turn(world, fell.person().entity_id(), Mode::Asleep, at)?;
        } else if *kind == Woke::EVENT_TYPE {
            let woke: Woke = codec::event_payload(event.payload())?;
            turn(world, woke.person().entity_id(), Mode::Awake, at)?;
        }
        Ok(Vec::new())
    }
}

/// The holder's needs after a meal, or [`None`] when the meal changes nothing: no needs, eating does
/// not restore hunger here, or the kind nourishes nothing.
fn fed(read: &WorldRead<'_>, consumed: &ItemsConsumed, at: WorldTime) -> Option<Needs> {
    let holder = consumed.holder();
    let needs = read.component::<Needs>(holder)?;
    let found = parameters(read, holder);
    if !found.restore.eating() {
        return None;
    }
    let by = read
        .component::<Nourishment>(consumed.item().entity_id())
        .map_or(0, Nourishment::hunger);
    if by == 0 {
        return None;
    }
    let by = u32::from(by).saturating_mul(consumed.count());
    Some(needs.fed(by, at, &rates(&found)))
}

/// `person`'s needs brought to `at` and turned to `mode`; nothing for a person without needs.
fn turn(
    world: &mut WorldView<'_, NeedsSystem>,
    person: EntityId,
    mode: Mode,
    at: WorldTime,
) -> Result<(), KernelError> {
    let read = world.read();
    let Some(needs) = read.component::<Needs>(person).copied() else {
        return Ok(());
    };
    let turned = needs.turned(mode, at, &rates(&parameters(&read, person)));
    world.insert(person, turned)?;
    Ok(())
}

impl PerceptionProvider for NeedsSystem {
    /// The `needs` record, to its holder alone (`src/disclose.rs`).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        disclosed(world, observer, subject)
    }
}
