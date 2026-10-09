//! The run-time half: the configured fact, the component it becomes on every Place, and the three
//! lookups a pack makes (`ARC-63` items 6 … 8, `ARC-65`).
//!
//! With no component on the place — the world configures nothing for the pack — every lookup returns
//! the compiled default and reads nothing else, which is what keeps an unconfigured world
//! byte-identical (IL-I1).

use std::marker::PhantomData;

use mineworld_authoring::EntityClasses;
use mineworld_contracts::{
    ActionTypeId, Component, ComponentSchemaVersion, ComponentTypeId, EntityId, EntityType, Event,
    EventEnvelope, EventSchemaVersion, EventTypeId, PlaceId, Rejection, SystemId, Visibility,
};
use mineworld_kernel::{
    Declarations, KernelError, OwnedBy, SystemDeclaration, SystemIdentity, WorldRead, WorldView,
};
use serde::{Deserialize, Serialize};

use super::InteractionSection;
use super::decl::{Audience, Role};
use super::resolve::Resolution;
use super::selector::{RoleSubjects, Roles};

/// `<pack>-interactions-configured`: a configured section, resolved, stated once at genesis
/// (`SystemInternal`, no subjects). Its payload is the pack's own encoding of [`Resolved`]; this type
/// only names the event.
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct SectionConfigured<S> {
    #[serde(skip)]
    _pack: PhantomData<fn() -> S>,
}

impl<S: InteractionSection> Event for SectionConfigured<S> {
    const EVENT_TYPE: EventTypeId = S::CONFIGURED;
    const OWNER: SystemId = <S as SystemIdentity>::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// `<pack>-interactions`: what a configured section says at one Place — the base and that place's
/// region, with the classes they are decided by. Owned by the section's pack, written only by its
/// reduction of [`SectionConfigured`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Interactions<S: InteractionSection> {
    classes: EntityClasses,
    base: Resolution<S>,
    region: Option<Resolution<S>>,
}

impl<S: InteractionSection> Component for Interactions<S> {
    const COMPONENT_TYPE: ComponentTypeId = S::COMPONENT;
    const OWNER: SystemId = <S as SystemIdentity>::ID;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl<S: InteractionSection> OwnedBy<S> for Interactions<S> {}

impl<S: InteractionSection> Interactions<S> {
    /// The resolution that applies here.
    fn here(&self) -> &Resolution<S> {
        self.region.as_ref().unwrap_or(&self.base)
    }
}

/// Adds the section's component, configured fact and its subscription to a pack's declaration.
pub fn declare<S: InteractionSection>(declaration: SystemDeclaration) -> SystemDeclaration {
    declaration
        .owning::<Interactions<S>>()
        .emitting::<SectionConfigured<S>>()
        .subscribing_to::<SectionConfigured<S>>()
}

/// Installs the section's component table; a pack's `install` calls it.
///
/// # Errors
///
/// The kernel's refusal.
pub fn install<S: InteractionSection>(tables: &mut Declarations<'_, S>) -> Result<(), KernelError> {
    tables.component::<Interactions<S>>()
}

/// Reduces the pack's configured fact into its component on every Place; `Ok(false)` for any other
/// fact. A pack's `react` calls it first.
///
/// # Errors
///
/// The kernel's refusal, or a payload the pack's own codec cannot read.
pub fn reduce<S: InteractionSection>(
    world: &mut WorldView<'_, S>,
    event: &EventEnvelope,
) -> Result<bool, KernelError> {
    if *event.event_type() != S::CONFIGURED {
        return Ok(false);
    }
    let resolved = S::decode(event.payload().payload()).map_err(|_| {
        KernelError::Contract(mineworld_contracts::ContractError::EventTypeMismatch {
            expected: S::CONFIGURED,
            actual: event.event_type().clone(),
        })
    })?;
    let places: Vec<(EntityId, mineworld_contracts::EntityKey)> = world
        .read()
        .entities()
        .filter(|entity| entity.entity_type() == EntityType::Place)
        .map(|entity| (entity.id(), entity.key().clone()))
        .collect();
    for (place, key) in places {
        let region = resolved
            .regions
            .iter()
            .find(|(region, _)| *region == key)
            .map(|(_, resolution)| resolution.clone());
        world.insert(
            place,
            Interactions::<S> {
                classes: resolved.classes.clone(),
                base: resolved.base.clone(),
                region,
            },
        )?;
    }
    Ok(true)
}

/// The type and tags of every entity filling a role, the `place` role filled by `at` when unset.
fn subjects<'w>(read: &WorldRead<'w>, roles: &Roles, at: Option<PlaceId>) -> RoleSubjects<'w> {
    let mut subjects = RoleSubjects::none();
    for role in Role::ALL {
        let entity = roles.get(role).or_else(|| {
            (role == Role::Place)
                .then(|| at.map(PlaceId::entity_id))
                .flatten()
        });
        if let Some(record) = entity.and_then(|entity| read.entity(entity)) {
            subjects = subjects.with(role, record.entity_type(), record.tags());
        }
    }
    subjects
}

/// The section's component at `at`, or — without a place — on any Place (each holds the base).
fn configured<'w, S: InteractionSection>(
    read: &WorldRead<'w>,
    at: Option<PlaceId>,
) -> Option<(&'w Interactions<S>, bool)> {
    match at {
        Some(place) => read
            .component::<Interactions<S>>(place.entity_id())
            .map(|found| (found, true)),
        None => read
            .components::<Interactions<S>>()
            .next()
            .map(|(_, found)| (found, false)),
    }
}

/// Whether the world's list permits `action` for these roles at `at` (`ARC-63` item 8): `Ok`, or
/// `Err(PermissionDenied)`. With nothing configured, `Ok` without reading anything else.
///
/// # Errors
///
/// [`Rejection::PermissionDenied`] when forbidden.
pub fn permits<S: InteractionSection>(
    read: &WorldRead<'_>,
    at: PlaceId,
    action: &ActionTypeId,
    roles: &Roles,
) -> Result<(), Rejection> {
    let Some((found, _)) = configured::<S>(read, Some(at)) else {
        return Ok(());
    };
    if found
        .here()
        .permits(&found.classes, action, &subjects(read, roles, Some(at)))
    {
        Ok(())
    } else {
        Err(Rejection::PermissionDenied)
    }
}

/// The pack's parameters for these roles at `at`. With nothing configured, the compiled defaults.
pub fn parameters<S: InteractionSection>(
    read: &WorldRead<'_>,
    at: PlaceId,
    roles: &Roles,
) -> S::Parameters {
    match configured::<S>(read, Some(at)) {
        None => S::Parameters::default(),
        Some((found, _)) => found
            .here()
            .parameters(&found.classes, &subjects(read, roles, Some(at))),
    }
}

/// What a fact's owner states it with, as the world's list routes it (`ARC-65`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consequence<K> {
    /// The audience, as a [`Visibility`]: the owner's default, or the list's narrowing of it.
    pub visibility: Visibility,
    /// Whether it enters a biography.
    pub biographical: bool,
    /// The pack's knobs.
    pub knobs: K,
}

/// The consequence of `fact` with these roles, at `at` when the fact has a place (without one, the
/// base). With nothing configured, `owner_default`, the compiled biographical flag, default knobs.
pub fn consequence<S: InteractionSection>(
    read: &WorldRead<'_>,
    at: Option<PlaceId>,
    fact: &EventTypeId,
    roles: &Roles,
    owner_default: Visibility,
) -> Consequence<S::Knobs> {
    let compiled = <S as crate::SystemPack>::BIOGRAPHICAL.contains(fact);
    let Some((found, at_place)) = configured::<S>(read, at) else {
        return Consequence {
            visibility: owner_default,
            biographical: compiled,
            knobs: S::Knobs::default(),
        };
    };
    let resolution = if at_place { found.here() } else { &found.base };
    let chosen = resolution.consequence(&found.classes, fact, &subjects(read, roles, at));
    Consequence {
        visibility: narrowed(chosen.audience, owner_default, at),
        biographical: chosen.biography.unwrap_or(compiled),
        knobs: chosen.knobs,
    }
}

/// The visibility an audience gives. The section was refused at load if it widened its owner's
/// default, so this only narrows; an audience of a place with no place to name keeps the default.
fn narrowed(
    audience: Option<Audience>,
    owner_default: Visibility,
    at: Option<PlaceId>,
) -> Visibility {
    match audience {
        None => owner_default,
        Some(Audience::Public) => Visibility::Public,
        Some(Audience::Place) => match (&owner_default, at) {
            (Visibility::Place(place), _) => Visibility::Place(*place),
            (_, Some(place)) => Visibility::Place(place),
            (_, None) => owner_default,
        },
        Some(Audience::Participants) => Visibility::Participants,
    }
}
