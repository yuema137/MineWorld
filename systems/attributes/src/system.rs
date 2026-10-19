//! The installable system: what it declares, the facts it reduces, the proposal check, and what it
//! discloses.

use std::collections::BTreeSet;

use mineworld_attributes_model::section::SectionSchema;
use mineworld_attributes_model::value::admits;
use mineworld_attributes_model::{
    ATTRIBUTES, AttributeChangeProposed, AttributesComposed, AttributesHanded, CatalogueProcess,
    Mutability, RunningCatalogue, TraitChanged, Traits, TraitsStated, View, Visibility, derive,
    render,
};
use mineworld_contracts::{
    ComponentRecord, ContractError, EntityId, EntityType, Event, EventEnvelope, EventRecord,
    EventTypeId, Rejection, RejectionCode, SystemId, Visibility as Audience,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, OwnedBy, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, Presence};
use mineworld_sdk::SystemPack;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// A Person's declared attributes.
#[derive(Debug, Default)]
pub struct AttributesSystem;

impl SystemIdentity for AttributesSystem {
    const ID: SystemId = ATTRIBUTES;
}

/// Only this pack writes a person's `traits` (`INV-7`, I-S30-1).
impl OwnedBy<AttributesSystem> for Traits {}

/// The build's view of the pack (`ARC-33`): it owns `attributes/` and `vocabularies/`, the `attributes:`
/// section, decoded in its catalogue, and resolves every person before seeding (`ARC-121`).
impl SystemPack for AttributesSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_data!();
    mineworld_sdk::owns_section_in_context!();
    mineworld_sdk::resolves!();

    fn section_schema() -> Option<schemars::Schema> {
        Some(schemars::SchemaGenerator::default().into_root_schema_for::<SectionSchema>())
    }
}

/// Encodes a payload this pack declared; infallible for its structs of ids and values.
pub(crate) fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("this pack's payloads are JSON-representable by construction")
}

fn payload<E: Event + DeserializeOwned>(record: &EventRecord) -> Result<E, ContractError> {
    serde_json::from_slice(record.payload_for::<E>()?).map_err(|_| {
        ContractError::EventTypeMismatch {
            expected: E::EVENT_TYPE,
            actual: record.event_type().clone(),
        }
    })
}

/// A refusal of this pack's own, with its reason code.
fn refused(event_type: EventTypeId, code: &'static str, detail: Option<String>) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: ATTRIBUTES,
        event_type,
        reason: Rejection::System {
            code: RejectionCode::from_static(code),
            detail,
        },
    }
}

/// The running catalogue, or the refusal of a fact that needs one.
fn catalogue(
    world: &WorldRead<'_>,
    event_type: &EventTypeId,
) -> Result<RunningCatalogue, KernelError> {
    RunningCatalogue::in_world(world)
        .ok_or_else(|| refused(event_type.clone(), "attributes-uncomposed", None))
}

impl System for AttributesSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No dependency: a reader or proposer depends on this pack, and an owner subscribes to
    /// `attributes-handed` without one (`ARC-28` item 2).
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Traits>()
            .emitting::<AttributesComposed>()
            .emitting::<TraitsStated>()
            .emitting::<AttributesHanded>()
            .emitting::<TraitChanged>()
            .subscribing_to::<AttributesComposed>()
            .subscribing_to::<TraitsStated>()
            .subscribing_to::<AttributeChangeProposed>()
            .subscribing_to::<TraitChanged>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Traits>()
    }

    /// `attributes-composed` → the catalogue Process, once. `traits-stated` and `trait-changed` →
    /// `Traits`. `attribute-change-proposed` → checked (`ARC-120` item 3): accepted as `trait-changed`,
    /// or refused with `not-a-proposer`, `not-evolving`, `out-of-domain` or `unknown-cause`.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type().clone();
        if kind == AttributesComposed::EVENT_TYPE {
            let composed: AttributesComposed = payload(event.payload())?;
            if RunningCatalogue::in_world(&world.read()).is_some() {
                return Err(refused(kind, "attributes-composed-twice", None));
            }
            world.start_process(
                ProcessStart::<CatalogueProcess<Self>>::new(encode(&composed.catalogue))
                    .uninterruptible(),
            )?;
            return Ok(Vec::new());
        }
        if kind == TraitsStated::EVENT_TYPE {
            let stated: TraitsStated = payload(event.payload())?;
            let running = catalogue(&world.read(), &kind)?;
            let traits = Traits::stated(running.catalogue(), stated.values)
                .map_err(|why| refused(kind, "not-a-plain-trait", Some(why)))?;
            world.insert(stated.person.entity_id(), traits)?;
            return Ok(Vec::new());
        }
        if kind == TraitChanged::EVENT_TYPE {
            let changed: TraitChanged = payload(event.payload())?;
            let running = catalogue(&world.read(), &kind)?;
            let person = changed.person.entity_id();
            let mut traits = world
                .read()
                .component::<Traits>(person)
                .cloned()
                .unwrap_or_default();
            traits
                .set(running.catalogue(), changed.attribute, changed.to)
                .map_err(|why| refused(kind, "not-a-plain-trait", Some(why)))?;
            world.insert(person, traits)?;
            return Ok(Vec::new());
        }
        if kind == AttributeChangeProposed::EVENT_TYPE {
            let proposed: AttributeChangeProposed = payload(event.payload())?;
            return check(&world.read(), event, proposed).map(|changed| vec![changed]);
        }
        Ok(Vec::new())
    }
}

/// The proposal check: the proposer listed the attribute; it is `attributes`' and `evolving`; `to` is
/// in its domain; `because` is an earlier fact.
fn check(
    world: &WorldRead<'_>,
    event: &EventEnvelope,
    proposed: AttributeChangeProposed,
) -> Result<Emission, KernelError> {
    let kind = event.event_type().clone();
    let running = catalogue(world, &kind)?;
    let catalogue = running.catalogue();
    let proposer = event.provenance().emitted_by();
    let id = &proposed.attribute;
    let listed = catalogue
        .uses()
        .get(proposer)
        .is_some_and(|uses| uses.proposes.contains(id));
    if !listed {
        return Err(refused(
            kind,
            "not-a-proposer",
            Some(format!("'{proposer}' did not list '{id}'")),
        ));
    }
    let declaration = catalogue
        .get(id)
        .filter(|declaration| {
            declaration.owner == ATTRIBUTES && declaration.mutability == Mutability::Evolving
        })
        .ok_or_else(|| refused(kind.clone(), "not-evolving", Some(id.to_string())))?;
    if !admits(&declaration.kind, &proposed.to) {
        return Err(refused(kind, "out-of-domain", Some(proposed.to.display())));
    }
    // Event ids are allocated in order, so a cause is a fact recorded before this one.
    if proposed.because >= event.id() {
        return Err(refused(kind, "unknown-cause", None));
    }
    let person = proposed.person.entity_id();
    let from = world
        .component::<Traits>(person)
        .and_then(|traits| traits.value(id))
        .cloned();
    let changed = TraitChanged {
        person: proposed.person,
        attribute: proposed.attribute.clone(),
        from,
        to: proposed.to,
        because: proposed.because,
    };
    let place = world
        .component::<Presence>(person)
        .map(|presence| presence.location().place());
    let audience = match place {
        Some(place) if declaration.visibility == Visibility::Perceivable => Audience::Place(place),
        _ => Audience::Entities(BTreeSet::from([person])),
    };
    Ok(Emission::new::<TraitChanged>(encode(&changed), audience).about(vec![person]))
}

impl PerceptionProvider for AttributesSystem {
    /// One `traits` record about a perceived person: the holder's view of their own, or another
    /// observer's, through the model crate's filter and renderer; derived values computed at the read.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let is_person = world
            .entity(subject)
            .is_some_and(|entity| entity.entity_type() == EntityType::Person);
        if !is_person {
            return Vec::new();
        }
        let Some(running) = RunningCatalogue::in_world(world) else {
            return Vec::new();
        };
        let catalogue = running.catalogue();
        let traits = world
            .component::<Traits>(subject)
            .cloned()
            .unwrap_or_default();
        let at = world.at();
        let disclosed = render::disclose(
            catalogue,
            &ATTRIBUTES,
            |declaration| match declaration.mutability {
                Mutability::Derived => derive::value(catalogue, declaration, &traits, at),
                Mutability::Fixed | Mutability::Evolving => traits.value(&declaration.id).cloned(),
            },
            View::of(observer, subject),
        );
        disclosed
            .and_then(|disclosed| serde_json::to_value(disclosed).ok())
            .map(|value| vec![ComponentRecord::new::<Traits>(subject, value)])
            .unwrap_or_default()
    }
}
