//! The installable system: what it declares, the facts it reduces, and what it discloses.

use mineworld_contracts::{
    Causation, ComponentRecord, EntityId, Event, EventEnvelope, EventTypeId, Rejection,
    RejectionCode, SystemId, WorldTime,
};
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
use crate::component::{Hunger, Needs, Nourishment};
use crate::disclose::disclosed;
use crate::event::{NeedsAssigned, NourishmentAssigned, StartingHunger};
use crate::food_energy::Reserve;
use crate::interactions::{HungerModel, base_parameters, parameters, rates};
use crate::level::{Level, Mode};

/// An authored form of the other hunger model than the world's (`ARC-370`).
pub const MODEL_MISMATCH: RejectionCode = RejectionCode::from_static("needs-model-mismatch");

/// An initial reserve above the person's daily use, which is the reserve's capacity (`ARC-370`).
pub const INITIAL_ENERGY_OUT_OF_RANGE: RejectionCode =
    RejectionCode::from_static("needs-initial-energy-out-of-range");

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
    /// 2 since food energy (`ARC-370`): every component and fact of this pack changed shape.
    const VERSION: SystemVersion = SystemVersion::new(2);

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
    ///   inserts [`Nourishment`] on the item kind. Each is first checked against the world's hunger
    ///   model and, for a reserve, the person's daily use: at genesis a contradiction is the author's
    ///   ([`KernelError::InvalidGenesisContent`]), otherwise a stating system's
    ///   ([`KernelError::FactRefusedByOwner`]);
    /// - `items-consumed` by a person with needs, when `eating` restores hunger: a unitless person's
    ///   hunger falls by the kind's `hunger` × count; a reserve rises by its `energy_kcal` × count,
    ///   capped. A kind with none, or with 0, writes nothing;
    /// - `fell-asleep` and `woke` bring the needs to the instant and turn their mode.
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
            let person = assigned.person().entity_id();
            let hunger = starting(&world.read(), event, person, assigned.hunger())?;
            world.insert(person, Needs::assigned(hunger, assigned.tiredness(), at))?;
        } else if *kind == NourishmentAssigned::EVENT_TYPE {
            let assigned: NourishmentAssigned = codec::event_payload(event.payload())?;
            let item = assigned.item().entity_id();
            agrees(&world.read(), event, item, assigned.nourishment())?;
            world.insert(item, assigned.nourishment().clone())?;
        } else if *kind == ItemsConsumed::EVENT_TYPE {
            let consumed: ItemsConsumed = codec::event_payload(event.payload())?;
            if let Some(needs) = fed(&world.read(), &consumed, at)? {
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

/// A contradiction found reducing `event` about `subject`: the author's at genesis, a stating system's
/// otherwise.
fn contradiction(
    read: &WorldRead<'_>,
    event: &EventEnvelope,
    subject: EntityId,
    code: RejectionCode,
    field: &str,
    detail: String,
) -> KernelError {
    let key = read.entity(subject).map(|entity| entity.key().clone());
    match (event.caused_by(), key) {
        (Causation::WorldGenesis, Some(subject)) => KernelError::InvalidGenesisContent {
            system: NeedsSystem::ID,
            subject,
            code,
            field: field.into(),
            detail: detail.into(),
        },
        _ => refused(event.event_type(), code, detail),
    }
}

fn refused(event_type: &EventTypeId, code: RejectionCode, detail: String) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: NeedsSystem::ID,
        event_type: event_type.clone(),
        reason: Rejection::System {
            code,
            detail: Some(detail),
        },
    }
}

fn mismatch(model: HungerModel, form: &str) -> String {
    format!("the world's hunger model is {model}; this is the {form} form")
}

/// The hunger `person` starts with, checked against the world's model and their daily use.
fn starting(
    read: &WorldRead<'_>,
    event: &EventEnvelope,
    person: EntityId,
    hunger: StartingHunger,
) -> Result<Hunger, KernelError> {
    let found = parameters(read, person);
    let at = event.at();
    match (found.hunger_model, hunger) {
        (HungerModel::Unitless, StartingHunger::Unitless { hunger }) => {
            Ok(Hunger::Unitless(Level::new(hunger, at, Mode::Awake)))
        }
        (HungerModel::FoodEnergy, StartingHunger::FoodEnergy { initial_kcal }) => {
            if initial_kcal > found.energy_kcal_per_day {
                return Err(contradiction(
                    read,
                    event,
                    person,
                    INITIAL_ENERGY_OUT_OF_RANGE,
                    "food_energy.initial_kcal",
                    format!(
                        "{initial_kcal} kcal is above this person's daily use and capacity, {} kcal",
                        found.energy_kcal_per_day
                    ),
                ));
            }
            Ok(Hunger::FoodEnergy(Reserve::holding(initial_kcal, at)))
        }
        (model @ HungerModel::FoodEnergy, StartingHunger::Unitless { .. }) => Err(contradiction(
            read,
            event,
            person,
            MODEL_MISMATCH,
            "hunger",
            mismatch(model, "unitless (hunger, or `needs: {}`)"),
        )),
        (model @ HungerModel::Unitless, StartingHunger::FoodEnergy { .. }) => Err(contradiction(
            read,
            event,
            person,
            MODEL_MISMATCH,
            "food_energy.initial_kcal",
            mismatch(model, "food-energy"),
        )),
    }
}

/// Whether an item's nourishment is in the world's model.
fn agrees(
    read: &WorldRead<'_>,
    event: &EventEnvelope,
    item: EntityId,
    nourishment: &Nourishment,
) -> Result<(), KernelError> {
    let model = base_parameters(read).hunger_model;
    let field = match (model, nourishment) {
        (HungerModel::Unitless, Nourishment::Unitless { .. })
        | (HungerModel::FoodEnergy, Nourishment::FoodEnergy { .. }) => return Ok(()),
        (HungerModel::FoodEnergy, Nourishment::Unitless { .. }) => "nourishment.hunger",
        (HungerModel::Unitless, Nourishment::FoodEnergy { .. }) => "nourishment.energy_kcal",
    };
    let form = if field == "nourishment.hunger" {
        "unitless"
    } else {
        "food-energy"
    };
    Err(contradiction(
        read,
        event,
        item,
        MODEL_MISMATCH,
        field,
        mismatch(model, form),
    ))
}

/// The holder's needs after a meal, or [`None`] when the meal changes nothing: no needs, eating does
/// not restore hunger here, or the kind nourishes nothing (0, or no form of the holder's model).
fn fed(
    read: &WorldRead<'_>,
    consumed: &ItemsConsumed,
    at: WorldTime,
) -> Result<Option<Needs>, KernelError> {
    let holder = consumed.holder();
    let Some(needs) = read.component::<Needs>(holder) else {
        return Ok(None);
    };
    let found = parameters(read, holder);
    if !found.restore.eating() {
        return Ok(None);
    }
    let count = consumed.count();
    match read.component::<Nourishment>(consumed.item().entity_id()) {
        Some(Nourishment::Unitless { hunger }) if *hunger > 0 => {
            let by = u32::from(*hunger).saturating_mul(count);
            Ok(needs.fed(by, at, &rates(&found)))
        }
        Some(Nourishment::FoodEnergy { energy_kcal, .. }) if *energy_kcal > 0 => needs
            .energized(*energy_kcal, count, at, found.energy_kcal_per_day)
            .map(Some)
            .ok_or_else(|| backward(&ItemsConsumed::EVENT_TYPE)),
        _ => Ok(None),
    }
}

/// A fact earlier than the reserve it would change: world time never runs backwards, so this is a bug.
fn backward(event_type: &EventTypeId) -> KernelError {
    refused(
        event_type,
        RejectionCode::from_static("needs-reserve-read-before-stored"),
        "a reserve was read before the instant it was stored at".to_owned(),
    )
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

#[cfg(test)]
mod tests {
    use mineworld_contracts::{
        ActionId, EntityKey, EntityType, EventEnvelope, EventId, EventRecord, Provenance,
        Visibility,
    };
    use mineworld_kernel::World;

    use super::*;

    /// The needs owner calls a contradiction the author's only when the fact was caused by genesis; the
    /// same contradiction in any other fact is a stating system's (`FactRefusedByOwner`), so a reduction
    /// after genesis is never reported as authored input (`ARC-370`, design §3.5).
    #[test]
    fn a_contradiction_is_the_authors_only_at_genesis() {
        let mut world = World::new();
        let ana = world
            .create_entity(EntityKey::new("ana").expect("a key"), EntityType::Person)
            .expect("created");
        let envelope = |caused_by| {
            EventEnvelope::new(
                EventId::from_raw(1),
                WorldTime::EPOCH,
                EventRecord::new::<NeedsAssigned>(Vec::new()),
                caused_by,
                Visibility::SystemInternal,
                Provenance::new(NeedsSystem::ID),
            )
        };
        let read = world.read();
        let at_genesis = contradiction(
            &read,
            &envelope(Causation::WorldGenesis),
            ana,
            MODEL_MISMATCH,
            "hunger",
            "detail".to_owned(),
        );
        assert!(matches!(
            at_genesis,
            KernelError::InvalidGenesisContent { ref subject, .. } if subject.as_str() == "ana"
        ));
        for caused_by in [
            Causation::Action(ActionId::from_raw(7)),
            Causation::Event(EventId::from_raw(3)),
        ] {
            assert!(matches!(
                contradiction(
                    &read,
                    &envelope(caused_by),
                    ana,
                    MODEL_MISMATCH,
                    "hunger",
                    "detail".to_owned()
                ),
                KernelError::FactRefusedByOwner { .. }
            ));
        }
    }
}
