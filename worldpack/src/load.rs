//! Turning a read pack into a running world: install, create, resolve, state.
//!
//! ```text
//! install   every capability the pack enables, in the order the pack states
//! create    every place, then every person — each in key order, each with its provenance
//! resolve   every authored key to the identity the runtime uses (DD-1, KD-3)
//! state     what is true of the world as it begins, as facts its systems reduce
//! ```
//!
//! # Determinism, which is the whole point of the order above
//!
//! `AC-12` requires that the same inputs reproduce the same run, and at this layer the inputs are a
//! directory of YAML. Identity is allocated by a monotonic counter in creation order, so **creation
//! order is what the ids are**: get it from something that varies and two loads of one pack produce
//! two different worlds whose event logs cannot be compared.
//!
//! So creation order is stated here and comes from nowhere else:
//!
//! ```text
//! places in EntityKey order, then people in EntityKey order
//! ```
//!
//! Not the order the author listed them in — reordering `population` then changes nothing. Not the
//! order the directory was read in — a filesystem's order is not a property of the pack. Not a hash
//! map's iteration order — which `clippy.toml` bans outright, because this is exactly the failure
//! that stays invisible until a replay disagrees. [`WorldPack`] holds its content in `BTreeMap`s, so
//! the order is the key order and there is nothing here to get wrong.
//!
//! Genesis facts follow the same order, so the event identities are fixed too.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_contracts::{
    EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location, Metadata,
    Millidegrees, Orientation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::{Emission, World, WorldRead};
use mineworld_presence::PerceptionProvider;

use crate::catalog::{self, Capability};
use crate::error::{ContentKind, PackError};
use crate::format::AuthoredLocation;
use crate::read::WorldPack;

/// A world built from a pack: the world itself, what its keys resolved to, and what it began with.
pub struct LoadedWorld {
    world: World,
    ids: BTreeMap<EntityKey, EntityId>,
    genesis: Vec<EventEnvelope>,
    providers: Vec<Box<dyn PerceptionProvider>>,
}

/// A loaded world handed over to whatever will run it.
///
/// Two things, because a host needs both and they have different owners: the world is the kernel's,
/// and the providers are the System Packs' answer to "what may this observer attempt", which a
/// perception implementation needs and cannot produce
/// (`systems/presence/src/interaction.rs`).
pub struct RunningWorld {
    /// The world, ready to be hosted or driven headless.
    pub world: World,
    /// The packs that answer for their own actions, in the order the pack composed them.
    pub providers: Vec<Box<dyn PerceptionProvider>>,
}

impl LoadedWorld {
    /// The world, read-only. Every accessor a caller needs while the world is still being set up.
    pub const fn world(&self) -> &World {
        &self.world
    }

    /// The world, mutably: for a caller that drives it headless rather than hosting it.
    ///
    /// A `&mut World` grants no write access to component state — every store mutation goes through a
    /// system holding its own write token (`INV-7`) — so what this permits is dispatching requests and
    /// enabling or disabling systems, which is what a headless run and a test both need.
    pub const fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// What each authoring key resolved to, in key order.
    ///
    /// The observable half of `DD-1`/`KD-3`: authored content refers to `alice`, the runtime refers to
    /// entity 2, and this map is the one place the two meet.
    pub const fn ids(&self) -> &BTreeMap<EntityKey, EntityId> {
        &self.ids
    }

    /// The identity one key resolved to.
    pub fn id(&self, key: &EntityKey) -> Option<EntityId> {
        self.ids.get(key).copied()
    }

    /// The facts this world began with, in the order they were recorded.
    ///
    /// Returned rather than kept: the durable event log is S5's, and a loader that appended to one
    /// would be the loader owning persistence.
    pub fn genesis(&self) -> &[EventEnvelope] {
        &self.genesis
    }

    /// Hands the world over to whatever will run it.
    pub fn into_running(self) -> RunningWorld {
        RunningWorld {
            world: self.world,
            providers: self.providers,
        }
    }
}

/// A world with this pack's systems installed and nothing else: what a saved world is resumed into.
///
/// Composition is the part of a pack a save cannot carry — systems are code — so a host resuming a save
/// composes the world from the pack and restores everything else from the save
/// (`docs/DECISIONS.md` `ARC-25`).
pub struct ComposedWorld {
    /// The world: systems installed, in the order the pack states, and no entity.
    pub world: World,
    /// The packs that answer for their own actions, in the order the pack composed them.
    pub providers: Vec<Box<dyn PerceptionProvider>>,
}

/// A world assembled from this pack — systems installed, entities created — that has **not** begun:
/// its genesis facts are returned rather than applied, so that a persisted world can run genesis
/// itself and journal exactly what it applied (step-06 PD-9).
pub struct AssembledWorld {
    /// The world, systems and entities in place, no state yet.
    pub world: World,
    /// What each authoring key resolved to.
    pub ids: BTreeMap<EntityKey, EntityId>,
    /// What is true of the world as it begins, in the order genesis must state it.
    pub facts: Vec<Emission>,
    /// The packs that answer for their own actions.
    pub providers: Vec<Box<dyn PerceptionProvider>>,
}

impl WorldPack {
    /// Installs this pack's systems into an empty world, in the order the pack states, and nothing
    /// else.
    pub fn compose(&self) -> Result<ComposedWorld, PackError> {
        let mut world = World::new();
        for capability in self.systems() {
            capability.install(&mut world)?;
        }
        Ok(ComposedWorld {
            world,
            providers: self.providers(),
        })
    }

    /// Composes the world and creates its entities, returning the genesis facts unapplied.
    pub fn assemble(&self) -> Result<AssembledWorld, PackError> {
        let ComposedWorld {
            mut world,
            providers,
        } = self.compose()?;

        let mut ids = BTreeMap::new();
        for (key, place) in self.places() {
            let id = world.create_authored_entity(
                key.clone(),
                EntityType::Place,
                place.tags.clone(),
                Some(self.provenance(ContentKind::Place, key, place.note.clone())),
            )?;
            ids.insert(key.clone(), id);
        }
        for (key, person) in self.people() {
            let id = world.create_authored_entity(
                key.clone(),
                EntityType::Person,
                person.tags.clone(),
                Some(self.provenance(ContentKind::Person, key, person.note.clone())),
            )?;
            ids.insert(key.clone(), id);
        }

        let facts = self.initial_facts(&world.read(), &ids)?;
        Ok(AssembledWorld {
            world,
            ids,
            facts,
            providers,
        })
    }

    /// Builds the world this pack describes, beginning at `at`: [`WorldPack::assemble`], then genesis.
    ///
    /// The instant is supplied rather than assumed, for the reason dispatch takes one: a world's clock
    /// is the host's, and a loader that read one would produce a world whose first facts are stamped
    /// with the moment it happened to be loaded.
    pub fn load(&self, at: WorldTime) -> Result<LoadedWorld, PackError> {
        let AssembledWorld {
            mut world,
            ids,
            facts,
            providers,
        } = self.assemble()?;
        let genesis = world.genesis(at, facts)?;

        Ok(LoadedWorld {
            world,
            ids,
            genesis,
            providers,
        })
    }

    /// The packs that answer for their own actions, in the order the pack composed them.
    fn providers(&self) -> Vec<Box<dyn PerceptionProvider>> {
        self.systems()
            .iter()
            .map(|capability| Capability::provider(*capability))
            .collect()
    }

    /// What is true of this world as it begins, as facts the systems that own that state will reduce.
    ///
    /// One per authored location today. A pack that could state a fact of its own choosing would be a
    /// World Pack defining a rule, so what it may state is bounded by what its capabilities own:
    /// `location` is presence's, and [`crate::catalog`] is where that is said.
    fn initial_facts(
        &self,
        world: &WorldRead<'_>,
        ids: &BTreeMap<EntityKey, EntityId>,
    ) -> Result<Vec<Emission>, PackError> {
        let mut facts = Vec::new();
        for (key, person) in self.people() {
            let Some(authored) = &person.location else {
                continue;
            };
            let path = self.content_file(ContentKind::Person, key);
            let person_id = PersonId::new(ids[key], EntityType::Person)
                .map_err(|error| PackError::value(path.clone(), error))?;
            facts.push(catalog::located(
                world,
                person_id,
                self.location(key, authored, ids, &path)?,
            )?);
        }
        Ok(facts)
    }

    /// One authored location as the contract's own [`Location`].
    fn location(
        &self,
        person: &EntityKey,
        authored: &AuthoredLocation,
        ids: &BTreeMap<EntityKey, EntityId>,
        path: &PathBuf,
    ) -> Result<Location, PackError> {
        // `read` has already refused a location naming a place this pack does not declare, so the key
        // resolves; the `ok_or` is the honest report for the two disagreeing rather than a panic.
        let place =
            ids.get(&authored.place)
                .copied()
                .ok_or_else(|| PackError::PersonInUnknownPlace {
                    person: person.clone(),
                    place: authored.place.clone(),
                    known: String::new(),
                    path: path.clone(),
                })?;
        let place = PlaceId::new(place, EntityType::Place)
            .map_err(|error| PackError::value(path, error))?;

        let mut location = Location::in_place(place);
        if let Some(position) = authored.position {
            location = location.with_local(LocalPosition::new(position.x, position.y, position.z));
        }
        if let Some(facing) = authored.facing {
            let orientation = Orientation::new(Millidegrees::new(facing), None)
                .map_err(|error| PackError::value(path, error))?;
            location = location.with_facing(orientation);
        }
        Ok(location)
    }

    /// Where an entity was authored, for a person debugging a world.
    ///
    /// Provenance, never gameplay state: `contracts/src/entity.rs` is explicit that nothing in the
    /// simulation may branch on these values. It is filled in because the alternative is a world whose
    /// entities cannot be traced back to the files that wrote them, which is the first question anyone
    /// asks of a world that came out wrong.
    fn provenance(&self, kind: ContentKind, key: &EntityKey, note: Option<String>) -> Metadata {
        Metadata {
            source_pack: self.id().to_owned(),
            source_path: format!("{}/{key}.yaml", kind.directory()),
            authoring_note: note,
        }
    }

    /// The file a key's content was read from, for an error to name.
    fn content_file(&self, kind: ContentKind, key: &EntityKey) -> PathBuf {
        self.root()
            .join(kind.directory())
            .join(format!("{key}.yaml"))
    }
}
