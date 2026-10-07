//! `mineworld biography` — a Person's objective biography, derived from a save's fact log
//! (`docs/DECISIONS.md` `ARC-29`, `docs/CORE_CONCEPTS.md` §5.2).
//!
//! Reads the save's manifest and fact table and nothing else: it never resumes the world, never reads
//! the journal and never writes a row. The biography is not stored anywhere — every invocation
//! regenerates it from the log, which is what makes it a projection rather than a second account.
//!
//! ```text
//! selection   the Person is among the fact's subjects or participants
//!             and its event type is one the composition declares biographical
//! entry       { at, event id, event type, place, counterparts } — envelope fields only
//! ```
//!
//! It names no event type of its own: the biographical set is each installed pack's own declaration,
//! aggregated by the World Pack catalog, so a new pack adds a constant there and no code here.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mineworld_contracts::{EntityId, EntityKey, EntityType, EventEnvelope, EventTypeId, WorldTime};
use mineworld_persistence::{
    Durability, Manifest, PersistError, PersistenceBackend, SqliteBackend, format,
};
use mineworld_worldpack::{Capability, WorldPack};
use serde_json::json;

use crate::described;

const DAY: i64 = 86_400;

fn damaged(error: PersistError) -> String {
    format!("[mineworld] {error}")
}

/// What `biography` was asked.
pub struct BiographyRequest<'a> {
    pub world: &'a Path,
    pub save: &'a Path,
    pub person: &'a EntityKey,
    pub json: bool,
}

/// Prints `person`'s biography from the save.
pub fn biography(request: &BiographyRequest<'_>) -> Result<(), String> {
    let pack = WorldPack::read(request.world).map_err(described)?;
    if !SqliteBackend::exists(request.save) {
        return Err(format!(
            "[mineworld] {} holds no save (no {})",
            request.save.display(),
            SqliteBackend::file(request.save).display()
        ));
    }
    let backend = SqliteBackend::open(request.save, Durability::ProcessCrash).map_err(damaged)?;
    let manifest_row = backend.manifest().map_err(damaged)?;
    format::check_format(manifest_row.format).map_err(damaged)?;
    let manifest: Manifest = format::decode(&manifest_row.body, "manifest").map_err(damaged)?;
    if manifest.pack != pack.id() {
        return Err(format!(
            "[mineworld] the save in {} is a world of '{}', not of '{}'",
            request.save.display(),
            manifest.pack,
            pack.id()
        ));
    }
    let selected = biographical(&manifest)?;

    let loaded = pack.load(WorldTime::EPOCH).map_err(described)?;
    let keys: BTreeMap<EntityId, &EntityKey> =
        loaded.ids().iter().map(|(key, id)| (*id, key)).collect();
    let person = loaded
        .id(request.person)
        .filter(|id| {
            loaded
                .world()
                .read()
                .entity(*id)
                .is_some_and(|entity| entity.entity_type() == EntityType::Person)
        })
        .ok_or_else(|| {
            format!(
                "[mineworld] '{}' is not a person of {}",
                request.person,
                pack.id()
            )
        })?;

    let all = usize::try_from(i64::MAX).unwrap_or(usize::MAX);
    let facts: Vec<EventEnvelope> = backend
        .last_facts(all)
        .map_err(damaged)?
        .iter()
        .map(|fact| format::decode(&fact.bytes, "fact"))
        .collect::<Result<_, _>>()
        .map_err(damaged)?;
    let entries: Vec<&EventEnvelope> = facts
        .iter()
        .filter(|fact| selected.contains(fact.event_type()) && names(fact, person))
        .collect();

    // Display names, from the save's own `named` facts through naming's projection of them, so this
    // command still decodes no pack's payload itself (ARC-31). A world without naming names nobody.
    let names = mineworld_naming::names_in(&facts);
    let display = |id: EntityId| names.get(&id).map(|name| name.as_str().to_owned());
    let name = |id: EntityId| {
        keys.get(&id)
            .map_or_else(|| format!("entity {}", id.raw()), ToString::to_string)
    };
    let named = |id: EntityId| match display(id) {
        Some(display) => format!("{} \"{display}\"", name(id)),
        None => name(id),
    };
    if !request.json {
        let types: Vec<&str> = selected.iter().map(EventTypeId::as_str).collect();
        println!(
            "biography  {} (entity {}) in {}: {} entries from {} facts; biographical: {}",
            named(person),
            person.raw(),
            pack.id(),
            entries.len(),
            facts.len(),
            types.join(", "),
        );
    }
    for fact in entries {
        let place = fact.place().map(|place| name(place.entity_id()));
        let met = counterparts(fact, person);
        if request.json {
            let others: Vec<String> = met.iter().copied().map(name).collect();
            let their_names: Vec<Option<String>> = met.iter().copied().map(display).collect();
            println!(
                "{}",
                json!({
                    "at": fact.at().seconds(),
                    "event_id": fact.id().raw(),
                    "event_type": fact.event_type().as_str(),
                    "place": place,
                    "counterparts": others,
                    "name": display(person),
                    "counterpart_names": their_names,
                })
            );
        } else {
            let others: Vec<String> = met.iter().copied().map(named).collect();
            let at = fact.at().seconds();
            println!(
                "  t{at:<10} day {:<4} {}  #{:<8} {:<24} at {:<10} with {}",
                at / DAY + 1,
                clock(at),
                fact.id().raw(),
                fact.event_type().as_str(),
                place.unwrap_or_else(|| "-".to_owned()),
                if others.is_empty() {
                    "-".to_owned()
                } else {
                    others.join(", ")
                },
            );
        }
    }
    Ok(())
}

/// The event types the save's composition declares biographical; a system this build does not
/// provide is refused by name rather than skipped, since skipping it would drop its part of a life.
fn biographical(manifest: &Manifest) -> Result<BTreeSet<EventTypeId>, String> {
    let mut selected = BTreeSet::new();
    for installed in &manifest.composition {
        let system = installed.declaration.system();
        let capability = Capability::resolve(system).ok_or_else(|| {
            format!("[mineworld] the save's world installs '{system}', which this build does not provide")
        })?;
        selected.extend(capability.biographical().iter().cloned());
    }
    Ok(selected)
}

/// Whether the fact is about, or involves, this person.
fn names(fact: &EventEnvelope, person: EntityId) -> bool {
    fact.subjects().contains(&person) || fact.participants().contains(&person)
}

/// Everybody else the fact names, subjects first, each once.
fn counterparts(fact: &EventEnvelope, person: EntityId) -> Vec<EntityId> {
    let mut others: Vec<EntityId> = Vec::new();
    for entity in fact.subjects().iter().chain(fact.participants()) {
        if *entity != person && !others.contains(entity) {
            others.push(*entity);
        }
    }
    others
}

/// `hh:mm:ss` of a day.
fn clock(seconds: i64) -> String {
    let of_day = seconds.rem_euclid(DAY);
    format!(
        "{:02}:{:02}:{:02}",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60
    )
}
