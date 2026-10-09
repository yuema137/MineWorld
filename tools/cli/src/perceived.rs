//! `mineworld perceived` — the facts a Person learned of, judged from a save's fact log by the one
//! audience rule (`docs/DECISIONS.md` `ARC-43`, `docs/MODULE_SPEC.md` §8.1).
//!
//! The offline half of what the server's `perceived` stream delivers live (`server/PROTOCOL.md` §5.8):
//! the same function, [`perceived_by`], over the same log, so a client's recorded stream and this
//! export agree fact for fact. It reads the save's manifest and fact table and the World Pack for the
//! authoring keys, and nothing else: it never resumes the world and never writes a row. The save is
//! opened, read and closed inside [`perceived`], so no handle outlives the command — on Windows an open
//! handle would keep the save directory from being removed or renamed.
//!
//! ```text
//! default   <event id> <at> <event type> place=<place key or -> caused_by=<cause>
//! --json    one PerceivedEvent per line, exactly as the server's perceived frame carries it
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_contracts::{Causation, EntityId, EntityKey, EntityType, EventEnvelope, EventId};
use mineworld_persistence::{
    Durability, Manifest, PersistError, PersistenceBackend, SqliteBackend, format,
};
use mineworld_presence::audience::perceived_by;
use mineworld_server::protocol::{PayloadForm, wire_fact};
use mineworld_worldpack::{PackRoots, WorldPack};

use crate::described;

/// The arguments of `mineworld perceived`, parsed by `clap` (`DEP-11`) and declared here rather than
/// in `main.rs`, which holds only the variant that flattens them.
#[derive(Debug, clap::Args)]
pub struct PerceivedArgs {
    /// The World Pack the save was created from (it names people by their keys).
    world: PathBuf,
    /// The save to read.
    #[arg(long, value_name = "DIR")]
    save: PathBuf,
    /// The Person, by authoring key, such as alice.
    #[arg(long, value_name = "KEY", value_parser = crate::seat)]
    person: EntityKey,
    /// Only the facts after this event id: the cursor of the server's perceived stream.
    #[arg(long, value_name = "ID")]
    since: Option<u64>,
    /// One PerceivedEvent per line, in the server's wire form, instead of lines for reading.
    #[arg(long)]
    json: bool,
}

fn damaged(error: PersistError) -> String {
    format!("[mineworld] {error}")
}

/// Prints what the Person perceived, from the save.
pub fn perceived(args: &PerceivedArgs, roots: &PackRoots) -> Result<(), String> {
    let pack = WorldPack::read_with(&args.world, roots).map_err(described)?;
    let facts = read_facts(&args.save, pack.id())?;

    let loaded = pack
        .load(mineworld_contracts::WorldTime::EPOCH)
        .map_err(described)?;
    let keys: BTreeMap<EntityId, &EntityKey> =
        loaded.ids().iter().map(|(key, id)| (*id, key)).collect();
    let person = loaded
        .id(&args.person)
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
                args.person,
                pack.id()
            )
        })?;

    let since = args.since.map(EventId::from_raw);
    let mut not_json = Vec::new();
    for fact in perceived_by(&facts, person, since) {
        if args.json {
            let (event, form) = wire_fact(fact).map_err(|error| {
                format!(
                    "[mineworld] fact {} cannot be written: {error}",
                    fact.id().raw()
                )
            })?;
            if form == PayloadForm::NotJson && !not_json.contains(fact.event_type()) {
                not_json.push(fact.event_type().clone());
                eprintln!(
                    "[mineworld] event type {} has a payload that is not JSON; written as null",
                    fact.event_type()
                );
            }
            let line = serde_json::to_string(&event).map_err(|error| {
                format!(
                    "[mineworld] fact {} cannot be written: {error}",
                    fact.id().raw()
                )
            })?;
            println!("{line}");
        } else {
            let place = fact
                .place()
                .map_or_else(|| "-".to_owned(), |place| name(&keys, place.entity_id()));
            println!(
                "{} {} {} place={place} caused_by={}",
                fact.id().raw(),
                fact.at().seconds(),
                fact.event_type(),
                cause(fact.caused_by()),
            );
        }
    }
    Ok(())
}

/// Every fact of the save, oldest first, after checking it is a save of `pack`. The backend is
/// dropped before this returns.
fn read_facts(save: &Path, pack: &str) -> Result<Vec<EventEnvelope>, String> {
    if !SqliteBackend::exists(save) {
        return Err(format!(
            "[mineworld] {} holds no save (no {})",
            save.display(),
            SqliteBackend::file(save).display()
        ));
    }
    let backend = SqliteBackend::open(save, Durability::ProcessCrash).map_err(damaged)?;
    let manifest_row = backend.manifest().map_err(damaged)?;
    format::check_format(manifest_row.format).map_err(damaged)?;
    let manifest: Manifest = format::decode(&manifest_row.body, "manifest").map_err(damaged)?;
    if manifest.pack != pack {
        return Err(format!(
            "[mineworld] the save in {} is a world of '{}', not of '{pack}'",
            save.display(),
            manifest.pack,
        ));
    }
    let all = usize::try_from(i64::MAX).unwrap_or(usize::MAX);
    let facts = backend
        .last_facts(all)
        .map_err(damaged)?
        .iter()
        .map(|fact| format::decode(&fact.bytes, "fact"))
        .collect::<Result<Vec<EventEnvelope>, _>>()
        .map_err(damaged)?;
    drop(backend);
    Ok(facts)
}

fn name(keys: &BTreeMap<EntityId, &EntityKey>, id: EntityId) -> String {
    keys.get(&id)
        .map_or_else(|| format!("entity-{}", id.raw()), ToString::to_string)
}

/// A cause in one word, so a line splits on spaces.
fn cause(causation: &Causation) -> String {
    match causation {
        Causation::Action(action) => format!("action:{}", action.raw()),
        Causation::Process(process) => format!("process:{}", process.raw()),
        Causation::Event(event) => format!("event:{}", event.raw()),
        Causation::SystemTick { system } => format!("tick:{system}"),
        Causation::WorldGenesis => "genesis".to_owned(),
    }
}
