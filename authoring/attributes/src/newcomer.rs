//! People who arrive later (`DECISIONS.md` `ARC-128`; `pr-s30j-runtime-sampling.md` §§5.2, 5.5): the
//! newcomer context a running world draws from, the fact that requests an
//! admission, the `attributes-newcomers` Process, and the checked constructor an admitter runs.
//!
//! | Fact | Audience | Reduced into |
//! | --- | --- | --- |
//! | `newcomer-admitted` | `SystemInternal` | checked, drawn: `traits-stated` and `attributes-handed` in the same dispatch |
//!
//! Refusal codes, each naming the person: `attributes-uncomposed`, `no-newcomers`, `not-an-admitter`,
//! `not-a-newcomer`, `already-arrived`, `newcomer-unsatisfiable`; `not-arrived` for a proposal about a person
//! who has not arrived; `unknown-cause` in `attributes`' own re-check.

use core::marker::PhantomData;
use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{
    EntityId, EntityKey, EntityType, Event, EventId, EventSchemaVersion, EventTypeId, PersonId,
    ProcessId, ProcessTypeId, Rejection, SystemId, Visibility as Audience,
};
use mineworld_kernel::{Emission, ProcessKind, SystemIdentity, WorldRead};
use serde::{Deserialize, Serialize};

use crate::ATTRIBUTES;
use crate::facts::{AttributesId, RunningCatalogue, refused};
use crate::profile::Profile;
use crate::sample::{Arrival, Fixed, Newcomer, Resident, Words, sample_newcomer};
use crate::traits::Traits;

/// What a running world draws newcomers from, carried by `attributes-composed` (§5.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewcomerContext {
    /// The profile's runtime form; its digest is unchanged by the round trip.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<Profile>,
    /// The population seed.
    pub seed: String,
    /// The salt.
    pub salt: u32,
    /// Every deferred person, including an explicit empty fixed set when nothing was authored.
    pub deferred: BTreeMap<EntityKey, Fixed>,
}

/// A System's request that `attributes` resolve a deferred person now, citing its cause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewcomerAdmitted {
    /// Who arrives.
    pub person: PersonId,
    /// The fact that is the reason (INV-15).
    pub because: EventId,
}

impl Event for NewcomerAdmitted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("newcomer-admitted");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The process type of the running newcomer context.
pub const NEWCOMERS_PROCESS: ProcessTypeId = ProcessTypeId::from_static("attributes-newcomers");

/// The newcomer Process's kind, generic over its owner type, as [`crate::CatalogueProcess`] is.
pub struct NewcomersProcess<O>(PhantomData<fn() -> O>);

impl<O: SystemIdentity + 'static> ProcessKind for NewcomersProcess<O> {
    const PROCESS_TYPE: ProcessTypeId = NEWCOMERS_PROCESS;
    type Owner = O;
}

/// The newcomer Process's state: the context, each deferred person's fixed set, who has arrived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Newcomers {
    /// What newcomers are drawn from.
    pub context: NewcomerContext,
    /// Who has arrived.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub arrived: BTreeSet<EntityKey>,
}

impl Newcomers {
    /// A fresh state from the context: nobody has arrived.
    pub const fn new(context: NewcomerContext) -> Self {
        Self {
            context,
            arrived: BTreeSet::new(),
        }
    }

    /// The world's newcomer Process and its state; `None` without `newcomers:`.
    pub fn in_world(world: &WorldRead<'_>) -> Option<(ProcessId, Self)> {
        let process = world.processes().find(|process| {
            *process.owner() == ATTRIBUTES && *process.process_type() == NEWCOMERS_PROCESS
        })?;
        let bytes = process.state_for::<NewcomersProcess<AttributesId>>().ok()?;
        let state = serde_json::from_slice(bytes).ok()?;
        Some((process.id(), state))
    }

    /// Whether `key` arrives later and has not yet.
    pub fn is_waiting(&self, key: &EntityKey) -> bool {
        self.context.deferred.contains_key(key) && !self.arrived.contains(key)
    }
}

/// A person's key, when `person` is a Person of this world.
fn key_of(world: &WorldRead<'_>, person: PersonId) -> Option<EntityKey> {
    world
        .entity(person.entity_id())
        .filter(|entity| entity.entity_type() == EntityType::Person)
        .map(|entity| entity.key().clone())
}

/// `not-arrived` for a person who arrives later and has not yet; otherwise nothing.
///
/// # Errors
///
/// `not-arrived`, naming the person.
pub fn check_arrived(world: &WorldRead<'_>, person: PersonId) -> Result<(), Rejection> {
    let Some((_, newcomers)) = Newcomers::in_world(world) else {
        return Ok(());
    };
    match key_of(world, person) {
        Some(key) if newcomers.is_waiting(&key) => Err(refused(
            "not-arrived",
            format!("'{key}' arrives later and has not arrived"),
        )),
        _ => Ok(()),
    }
}

/// A newcomer's draw: whose, and the values.
#[derive(Debug, Clone)]
pub struct Drawn {
    /// The newcomer's key.
    pub key: EntityKey,
    /// The newcomer Process and its state before the arrival.
    pub process: (ProcessId, Newcomers),
    /// The values, with their origins.
    pub arrival: Arrival,
}

/// Every check of §5.5 but the cause, then the draw at `world.at()` against every Person already there
/// (plain stored traits only, in entity-id order; people still waiting are not counted).
///
/// # Errors
///
/// The [`Rejection`] naming the code and the person.
pub fn draw(
    world: &WorldRead<'_>,
    admitter: &SystemId,
    person: PersonId,
    because: EventId,
) -> Result<Drawn, Rejection> {
    let running = RunningCatalogue::in_world(world).ok_or_else(|| {
        refused(
            "attributes-uncomposed",
            format!("the world has no catalogue to admit {person:?}"),
        )
    })?;
    let catalogue = running.catalogue();
    let (process, newcomers) = Newcomers::in_world(world).ok_or_else(|| {
        refused(
            "no-newcomers",
            format!("the world configures no newcomers:, so {person:?} cannot be admitted"),
        )
    })?;
    if !catalogue
        .uses()
        .get(admitter)
        .is_some_and(|uses| uses.admits)
    {
        return Err(refused(
            "not-an-admitter",
            format!("'{admitter}' did not declare uses: {{ admits: true }} to admit {person:?}"),
        ));
    }
    let key = key_of(world, person)
        .filter(|key| newcomers.context.deferred.contains_key(key))
        .ok_or_else(|| {
            refused(
                "not-a-newcomer",
                format!("{person:?} is not listed in newcomers.later"),
            )
        })?;
    if newcomers.arrived.contains(&key) {
        return Err(refused(
            "already-arrived",
            format!("'{key}' has already arrived"),
        ));
    }
    let tags = world
        .entity(person.entity_id())
        .map(|entity| entity.tags().clone())
        .unwrap_or_default();
    // In entity-id order, so the population is the same whatever order the store iterates in.
    let residents: BTreeMap<EntityId, (EntityKey, &Traits)> = world
        .components::<Traits>()
        .filter_map(|(entity, traits)| {
            let resident = world.entity(entity)?;
            if resident.entity_type() != EntityType::Person {
                return None;
            }
            let resident = resident.key().clone();
            (resident != key && !newcomers.is_waiting(&resident))
                .then_some((entity, (resident, traits)))
        })
        .collect();
    let population: Vec<Resident<'_>> = residents
        .values()
        .map(|(key, traits)| Resident {
            key: key.as_str(),
            traits,
        })
        .collect();
    let fixed = newcomers.context.deferred.get(&key).ok_or_else(|| {
        refused(
            "not-a-newcomer",
            format!("'{key}' has no fixed set in the composed context"),
        )
    })?;
    let context = &newcomers.context;
    let arrival = sample_newcomer(
        catalogue,
        context.profile.as_ref(),
        Words::new(&context.seed, context.salt),
        Newcomer {
            key: key.as_str(),
            tags: &tags,
            fixed,
        },
        &population,
        world.at(),
        because,
    )
    .map_err(|error| refused("newcomer-unsatisfiable", format!("'{key}': {error}")))?;
    Ok(Drawn {
        key,
        process: (process, newcomers),
        arrival,
    })
}

/// The check every admitter runs in `validate`, so a refused admission is an ordinary rejection of its own
/// request (`ARC-26`: the owner's checked constructor): every code of §5.5 but `unknown-cause`.
///
/// # Errors
///
/// The [`Rejection`] naming the code and the person.
pub fn check_newcomer(
    world: &WorldRead<'_>,
    admitter: &SystemId,
    person: PersonId,
) -> Result<(), Rejection> {
    draw(world, admitter, person, EventId::from_raw(0)).map(|_| ())
}

/// The same check, then the emission the admitter returns, citing its cause.
///
/// # Errors
///
/// As [`check_newcomer`].
pub fn admit_newcomer(
    world: &WorldRead<'_>,
    admitter: &SystemId,
    person: PersonId,
    because: EventId,
) -> Result<Emission, Rejection> {
    check_newcomer(world, admitter, person)?;
    let admitted = NewcomerAdmitted { person, because };
    let payload = serde_json::to_vec(&admitted)
        .map_err(|error| refused("newcomer-unsatisfiable", format!("{person:?}: {error}")))?;
    Ok(
        Emission::new::<NewcomerAdmitted>(payload, Audience::SystemInternal)
            .about(vec![person.entity_id()]),
    )
}
