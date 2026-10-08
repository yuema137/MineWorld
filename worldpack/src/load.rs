//! Turning a read pack into a running world: install, create, resolve, state.
//!
//! ```text
//! install   every capability the pack enables, in the order the pack states
//! create    every place, then every person, then every item kind, then every organization — each
//!           in key order, each with its provenance
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
//! places, then people, then items, then organizations, each in EntityKey order
//! ```
//!
//! Items and organizations come last (`DECISIONS.md` `ARC-36`) so that a world which declares none
//! allocates exactly the ids it did before those kinds existed. Not the order the author listed them in — reordering `population` then changes nothing. Not the
//! order the directory was read in — a filesystem's order is not a property of the pack. Not a hash
//! map's iteration order — which `clippy.toml` bans outright, because this is exactly the failure
//! that stays invisible until a replay disagrees. [`WorldPack`] holds its content in `BTreeMap`s, so
//! the order is the key order and there is nothing here to get wrong.
//!
//! Genesis facts are fixed too: passages, then locations, then sections — items', organizations',
//! places', people's, each in key order — so what a person's or a place's section may name is stated
//! before it, and a world with no items or organizations keeps every event id it had.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use mineworld_authoring::{AuthoredContent, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, EventEnvelope, LocalPosition, Location, Metadata,
    Millidegrees, Orientation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::{Emission, World, WorldRead};
use mineworld_presence::PerceptionProvider;

use crate::catalog::{self, Capability};
use crate::error::{ContentKind, PackError};
use crate::format::{AuthoredLocation, AuthoredPosition, FoundSection, SectionState};

use crate::read::WorldPack;

/// An authored position as the contract's own [`LocalPosition`].
fn position(authored: AuthoredPosition) -> LocalPosition {
    LocalPosition::new(authored.x, authored.y, authored.z)
}

/// The genesis facts one section becomes, from its owner — refused if the owner refuses the value,
/// and refused if any of them is another pack's vocabulary (`ARC-31`, `ARC-26`).
fn seeded(
    world: &WorldRead<'_>,
    ids: &BTreeMap<EntityKey, EntityId>,
    subject: &EntityKey,
    section: &FoundSection,
    content: &Arc<dyn AuthoredContent>,
    path: &Path,
) -> Result<Vec<Emission>, PackError> {
    let owner = content.owner();
    let emissions = content
        .seed(&Seeding::new(world, ids), ids[subject])
        .map_err(|reason| PackError::SectionRefusedByOwner {
            subject: subject.clone(),
            section: section.name,
            system: owner.clone(),
            reason: Box::new(reason),
            path: path.to_path_buf(),
        })?;
    if let Some(foreign) = emissions.iter().find(|emission| *emission.owner() != owner) {
        return Err(PackError::SectionStatedAnotherPacksFact {
            section: section.name,
            system: owner,
            event_type: foreign.event_type().clone(),
            owner: foreign.owner().clone(),
            path: path.to_path_buf(),
        });
    }
    Ok(emissions)
}

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
    ///
    /// First it registers the build's extension catalogs — the installed set's `extension` lines,
    /// presence's arrival resolvers among them — each with its owner (`docs/DECISIONS.md` `ARC-62`,
    /// `ARC-39`). Every host composes through here (`assemble` and `load` call this), so no host can
    /// run a world without registering them, and an implementing pack installed below finds itself
    /// registered. Registering the same list again is a no-op, so a process may compose any number of
    /// worlds; a different list registered earlier in the process is a defect of the host and panics,
    /// naming both.
    pub fn compose(&self) -> Result<ComposedWorld, PackError> {
        Capability::register_extensions();
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
        // Item kinds and organizations after every person (`ARC-36`), so declaring them moves no id
        // a world already had.
        for (key, item) in self.items() {
            let id = world.create_authored_entity(
                key.clone(),
                EntityType::Item,
                item.tags.clone(),
                Some(self.provenance(ContentKind::Item, key, item.note.clone())),
            )?;
            ids.insert(key.clone(), id);
        }
        for (key, organization) in self.organizations() {
            let id = world.create_authored_entity(
                key.clone(),
                EntityType::Organization,
                organization.tags.clone(),
                Some(self.provenance(ContentKind::Organization, key, organization.note.clone())),
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
        // Passages first: they are facts about the places, which exist before anybody is in them.
        for (key, place) in self.places() {
            let path = self.content_file(ContentKind::Place, key);
            for passage in &place.passages {
                let (a, b) = (ids[key], ids[&passage.to]);
                let a = PlaceId::new(a, EntityType::Place)
                    .map_err(|error| PackError::value(path.clone(), error))?;
                let b = PlaceId::new(b, EntityType::Place)
                    .map_err(|error| PackError::value(path.clone(), error))?;
                facts.push(catalog::opened(
                    a,
                    passage.here.map(position),
                    b,
                    passage.there.map(position),
                ));
            }
        }
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
        // Then sections (ARC-31): after every passage and location, so those keep the event ids they
        // had before sections existed; items', organizations', places', people's, each in key order
        // (ARC-36) — the one order `read` refuses in, so what a section names is seeded before it.
        for (kind, key, sections) in self.sectioned_files() {
            let path = self.content_file(kind, key);
            for (section, content) in self.in_composition_order(sections) {
                facts.extend(seeded(world, ids, key, section, content, &path)?);
            }
        }
        Ok(facts)
    }

    /// A file's decoded sections, in the order the world's `systems` lists their owners: the order
    /// is the pack's statement wherever it reaches the log (MODULE_SPEC §4.1 rule 2), and the order
    /// keys happen to appear in a file is not.
    fn in_composition_order<'a>(
        &self,
        sections: &'a [FoundSection],
    ) -> Vec<(&'a FoundSection, &'a Arc<dyn AuthoredContent>)> {
        let mut decoded: Vec<(usize, &FoundSection, &Arc<dyn AuthoredContent>)> = sections
            .iter()
            .filter_map(|section| match &section.state {
                SectionState::Decoded(content) => {
                    let rank = self
                        .systems()
                        .iter()
                        .position(|system| *system == section.owner)
                        .unwrap_or(usize::MAX);
                    Some((rank, section, content))
                }
                SectionState::OwnerNotEnabled | SectionState::NotCarriedHere => None,
            })
            .collect();
        decoded.sort_by_key(|(rank, _, _)| *rank);
        decoded
            .into_iter()
            .map(|(_, section, content)| (section, content))
            .collect()
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
        if let Some(authored) = authored.position {
            location = location.with_local(position(authored));
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

#[cfg(test)]
mod tests {
    use mineworld_authoring::{AuthoredSection, ContentKind, Decode, SectionName};
    use mineworld_contracts::{Rejection, SystemId};
    use mineworld_kernel::SystemIdentity;
    use mineworld_presence::{PresenceSystem, arrival};
    use serde::de::DeserializeSeed;

    use super::*;

    /// A section owner that does what `ARC-31` forbids: it seeds presence's `arrived`.
    struct Trespasser;

    impl SystemIdentity for Trespasser {
        const ID: SystemId = SystemId::from_static("trespasser");
    }

    impl AuthoredSection for Trespasser {
        const SECTION: SectionName = SectionName::from_static("trespass");
        const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
        type Authored = ();

        fn seed(
            seeding: &Seeding<'_, '_>,
            subject: EntityId,
            (): &(),
        ) -> Result<Vec<Emission>, Rejection> {
            let world = seeding.world();
            let place = seeding
                .resolve(&EntityKey::new("cafe").expect("a key"), EntityType::Place)
                .ok_or(Rejection::PreconditionFailed)?;
            let person = PersonId::new(subject, EntityType::Person)
                .map_err(|_| Rejection::PreconditionFailed)?;
            let place = PlaceId::new(place, EntityType::Place)
                .map_err(|_| Rejection::PreconditionFailed)?;
            Ok(vec![arrival(world, person, Location::in_place(place))?])
        }
    }

    #[test]
    fn a_section_that_seeds_another_packs_fact_is_refused_by_name() {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        let mut ids = BTreeMap::new();
        let cafe = world
            .create_entity(EntityKey::new("cafe").expect("a key"), EntityType::Place)
            .expect("a place");
        let alice = world
            .create_entity(EntityKey::new("alice").expect("a key"), EntityType::Person)
            .expect("a person");
        ids.insert(EntityKey::new("cafe").expect("a key"), cafe);
        ids.insert(EntityKey::new("alice").expect("a key"), alice);

        let content = Decode::<Trespasser>::new()
            .deserialize(serde_json::Value::Null)
            .expect("an empty section decodes");
        let section = FoundSection {
            owner: Capability::Naming,
            name: Trespasser::SECTION,
            state: SectionState::Decoded(content.clone()),
        };
        let refusal = seeded(
            &world.read(),
            &ids,
            &EntityKey::new("alice").expect("a key"),
            &section,
            &content,
            Path::new("people/alice.yaml"),
        )
        .expect_err("another pack's vocabulary");

        let PackError::SectionStatedAnotherPacksFact {
            system,
            event_type,
            owner,
            ..
        } = &refusal
        else {
            panic!("got: {refusal}");
        };
        assert_eq!(*system, Trespasser::ID);
        assert_eq!(event_type.as_str(), "arrived");
        assert_eq!(*owner, PresenceSystem::ID);
        assert!(refusal.to_string().contains("people/alice.yaml"));
    }

    // -----------------------------------------------------------------------------------------
    // Sections on item and organization files (`ARC-36`). No installed pack owns one before the
    // packs that need them exist (F-22), so a probe owner that exists only here stands in. It is
    // decoded with authoring's own `Decode` and attributed to an installed capability for ranking
    // only, as `Trespasser` is above; it is never installed.
    // -----------------------------------------------------------------------------------------

    /// A section owner carried by every kind of file, naming whatever entities its value lists.
    struct Probe;

    impl SystemIdentity for Probe {
        const ID: SystemId = SystemId::from_static("probe");
    }

    /// The probe's one fact: that it was seeded about `subject`.
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct Probed {
        subject: u64,
    }

    impl mineworld_contracts::Event for Probed {
        const EVENT_TYPE: mineworld_contracts::EventTypeId =
            mineworld_contracts::EventTypeId::from_static("probed");
        const OWNER: SystemId = Probe::ID;
        const SCHEMA_VERSION: mineworld_contracts::EventSchemaVersion =
            mineworld_contracts::EventSchemaVersion::new(1);
    }

    /// One entity the probe's value names, and the type it names it as.
    #[derive(Debug, serde::Deserialize)]
    struct Names {
        key: EntityKey,
        entity_type: EntityType,
    }

    impl AuthoredSection for Probe {
        const SECTION: SectionName = SectionName::from_static("probe");
        const CARRIED_BY: &'static [ContentKind] = &ContentKind::ALL;
        type Authored = Vec<Names>;

        fn references(authored: &Vec<Names>) -> Vec<mineworld_authoring::Reference<'_>> {
            authored
                .iter()
                .map(|names| mineworld_authoring::Reference {
                    key: &names.key,
                    entity_type: names.entity_type,
                })
                .collect()
        }

        fn seed(
            _: &Seeding<'_, '_>,
            subject: EntityId,
            _: &Vec<Names>,
        ) -> Result<Vec<Emission>, Rejection> {
            Ok(vec![probed(subject)])
        }
    }

    fn probed(subject: EntityId) -> Emission {
        let payload = serde_json::to_vec(&Probed {
            subject: subject.raw(),
        })
        .expect("a probe fact encodes");
        Emission::new::<Probed>(payload, mineworld_contracts::Visibility::Public)
    }

    fn key(value: &str) -> EntityKey {
        EntityKey::new(value).expect("a key")
    }

    /// One probe section whose value is `value`, as the loader would have found it.
    fn probe(value: serde_json::Value) -> Vec<FoundSection> {
        let content = Decode::<Probe>::new()
            .deserialize(value)
            .expect("a probe section decodes");
        vec![FoundSection {
            owner: Capability::Naming,
            name: Probe::SECTION,
            state: SectionState::Decoded(content),
        }]
    }

    /// A pack holding a probe section on two item kinds, one organization, one place and one person;
    /// the item kinds' sections carry `on_lantern` and `on_pebble`.
    fn probed_pack(on_lantern: serde_json::Value, on_pebble: serde_json::Value) -> WorldPack {
        use crate::format::{AuthoredItem, AuthoredOrganization, AuthoredPerson, AuthoredPlace};

        let place = AuthoredPlace {
            sections: probe(serde_json::json!([])),
            ..AuthoredPlace::default()
        };
        let person = AuthoredPerson {
            sections: probe(serde_json::json!([])),
            ..AuthoredPerson::default()
        };
        let organization = AuthoredOrganization {
            sections: probe(serde_json::json!([])),
            ..AuthoredOrganization::default()
        };
        let lantern = AuthoredItem {
            sections: probe(on_lantern),
            ..AuthoredItem::default()
        };
        let pebble = AuthoredItem {
            sections: probe(on_pebble),
            ..AuthoredItem::default()
        };
        WorldPack::in_memory(
            vec![Capability::Presence, Capability::Naming],
            BTreeMap::from([(key("cafe"), place)]),
            BTreeMap::from([(key("alice"), person)]),
            BTreeMap::from([(key("pebble"), pebble), (key("lantern"), lantern)]),
            BTreeMap::from([(key("chess-club"), organization)]),
        )
    }

    #[test]
    fn sections_on_items_and_organizations_are_seeded_before_places_and_people() {
        let pack = probed_pack(serde_json::json!([]), serde_json::json!([]));

        let assembled = pack.assemble().expect("assembles");

        let id = |name: &str| assembled.ids[&key(name)];
        // Entities: places, people, items, organizations, each in key order.
        let order: Vec<u64> = ["cafe", "alice", "lantern", "pebble", "chess-club"]
            .into_iter()
            .map(|name| id(name).raw())
            .collect();
        assert!(
            order.windows(2).all(|pair| pair[0] < pair[1]),
            "ids are allocated places, people, items, organizations: {order:?}"
        );
        // Sections: items', organizations', places', people's, each in key order.
        let expected: Vec<Emission> = ["lantern", "pebble", "chess-club", "cafe", "alice"]
            .into_iter()
            .map(|name| probed(id(name)))
            .collect();
        assert_eq!(assembled.facts, expected);
    }

    #[test]
    fn a_section_may_name_an_item_as_an_item_and_not_as_a_place() {
        let as_item = probed_pack(
            serde_json::json!([]),
            serde_json::json!([{ "key": "lantern", "entity_type": "item" }]),
        );
        crate::read::check_sections(&as_item).expect("lantern is declared as an item");

        let as_place = probed_pack(
            serde_json::json!([]),
            serde_json::json!([{ "key": "lantern", "entity_type": "place" }]),
        );
        let refusal = crate::read::check_sections(&as_place).expect_err("lantern is no place");
        let PackError::SectionNamesUnknownEntity {
            subject,
            key: named,
            expected,
            path,
            ..
        } = &refusal
        else {
            panic!("got: {refusal}");
        };
        assert_eq!(*subject, key("pebble"));
        assert_eq!(*named, key("lantern"));
        assert_eq!(*expected, EntityType::Place);
        assert!(path.ends_with("items/pebble.yaml"));
    }
}
