//! `mineworld interactions <world> [--place KEY] [--json]` — what a World Pack's Interaction List
//! resolves to (`docs/DECISIONS.md` `ARC-63` item 10, `ARC-64`).
//!
//! Reads the World Pack only: it assembles the world in memory, as `validate` does, takes each
//! configured section's genesis fact — the section already resolved by its owner — and each entity's
//! class, and prints them. It writes nothing and resumes nothing.
//!
//! ```text
//! sections   each enabled pack with a section: its resolved value (base, then each region), or
//!            "default (compiled)" when the world does not configure it; with --place, the
//!            resolution that applies at that place
//! classes    each entity, by key: its class (ARC-64)
//! ```
//!
//! It names no pack: the sections are each capability's `interaction_section()`, and the values are
//! printed as their owners encoded them.

use std::collections::BTreeMap;
use std::path::Path;

use mineworld_contracts::EntityKey;
use mineworld_worldpack::{Classed, PackRoots, WorldPack};
use serde_json::{Value, json};

use crate::described;

/// What a world with a section left unconfigured shows for it.
pub const COMPILED: &str = "default (compiled)";

/// What `interactions` was asked.
pub struct InteractionsRequest<'a> {
    pub world: &'a Path,
    pub place: Option<&'a EntityKey>,
    pub json: bool,
    /// Where the world's requirements are resolved (`ARC-54`).
    pub roots: &'a PackRoots,
}

/// The resolution that applies at `place` in a resolved section: its region's, or the base.
fn at_place(resolved: &Value, place: &EntityKey) -> Value {
    let region = resolved["regions"].as_array().and_then(|regions| {
        regions
            .iter()
            .find(|region| region[0].as_str() == Some(place.as_str()))
            .map(|region| region[1].clone())
    });
    match region {
        Some(resolution) => json!({ "place": place.as_str(), "region": resolution }),
        None => json!({ "place": place.as_str(), "base": resolved["base"].clone() }),
    }
}

/// Prints the world's resolved Interaction List.
pub fn interactions(request: &InteractionsRequest<'_>) -> Result<(), String> {
    let pack = WorldPack::read_with(request.world, request.roots).map_err(described)?;
    let assembled = pack.assemble().map_err(described)?;
    if let Some(place) = request.place
        && !pack.places().contains_key(place)
    {
        return Err(format!(
            "[mineworld] '{place}' is not a place of {}",
            pack.id()
        ));
    }

    let mut sections = BTreeMap::new();
    for capability in pack.systems() {
        let Some(section) = capability.interaction_section() else {
            continue;
        };
        let configured = assembled
            .facts
            .iter()
            .find(|fact| *fact.event_type() == section.configured);
        let value = match configured {
            None => Value::String(COMPILED.to_owned()),
            Some(fact) => {
                let resolved: Value = serde_json::from_slice(fact.record().payload())
                    .map_err(|error| format!("[mineworld] {capability}'s section: {error}"))?;
                match request.place {
                    Some(place) => at_place(&resolved, place),
                    None => resolved,
                }
            }
        };
        sections.insert(capability.id().to_string(), value);
    }

    let read = assembled.world.read();
    let mut classes = BTreeMap::new();
    for entity in read.entities() {
        let class = pack.classes().class_of(Classed {
            entity_type: entity.entity_type(),
            tags: entity.tags(),
        });
        classes.insert(entity.key().to_string(), Value::String(class.to_string()));
    }

    if request.json {
        let document = json!({ "world": pack.id(), "sections": sections, "classes": classes });
        println!("{document}");
        return Ok(());
    }
    println!("interactions  {} ({})", pack.name(), pack.id());
    if sections.is_empty() {
        println!("  no enabled pack has a section");
    }
    for (system, value) in &sections {
        match value {
            Value::String(text) => println!("section  {system}: {text}"),
            _ => println!(
                "section  {system}:\n{}",
                serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
            ),
        }
    }
    for (entity, class) in &classes {
        println!(
            "class    {entity:<16} {}",
            class.as_str().unwrap_or_default()
        );
    }
    Ok(())
}
