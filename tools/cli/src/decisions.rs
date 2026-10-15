//! The decision list, for the hosts: read the pack's file or `--decisions`, apply `--seat-model`, resolve
//! one model per seat (`docs/MODULE_SPEC.md` §4.1, §5.1; `pr-dl-a-decision-list.md` SD-DLa-11, SD-DLa-15).
//!
//! ```text
//! roster     the pack's seats in roster order, each seat's class (ARC-64), the classes of persons, the
//!            people — handed to mineworld-decision, which never reads a World Pack
//! read       --decisions FILE, else world.yaml's decisions:, else none
//! resolve    --seat-model over the file; each seat's in-process model, and its memory choice
//! ```
//!
//! This binary is the composition root that hands the decision-list reader this build's installed models
//! (`cognition/installed`); `run`, `server --town`, `validate` and `mineworld decisions` all come through
//! here, so they read one list the same way.

use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, EntityType};
use mineworld_decision::{
    ClassRef, DecisionList, Origin, ResolvedDecisions, Roster, SeatEntry, SeatOverride,
};
use mineworld_installed_models::Model;
use mineworld_worldpack::{ClassName, Classed, PackRoots, WorldPack};

use crate::described;

/// What `run`'s header and the server's `town:` statement say about an external model's seat.
pub const RUN_HOSTS_NONE: &str = "run hosts no out-of-process model";

/// What the server says about an external model's seat: it runs the fallback until the model joins.
pub const SERVER_FALLBACK: &str = "in-server until the model's process takes the seat over";

/// The operator's decision-list flags.
#[derive(Debug, Clone, Default)]
pub struct DecisionArgs {
    /// `--decisions FILE`: replaces the pack's file.
    pub file: Option<PathBuf>,
    /// `--seat-model SEAT=MODEL`, in the order given.
    pub seat_models: Vec<SeatOverride>,
}

impl DecisionArgs {
    /// Whether the operator gave either flag.
    pub fn given(&self) -> bool {
        self.file.is_some() || !self.seat_models.is_empty()
    }
}

/// A `--seat-model` value, checked as `SEAT=MODEL` with the seat a key; the model is checked against
/// the build's models when the list resolves, so its refusal can list them.
pub fn seat_model(text: &str) -> Result<SeatOverride, String> {
    let (seat, model) = text
        .split_once('=')
        .ok_or_else(|| format!("'{text}' is not SEAT=MODEL"))?;
    let seat =
        EntityKey::new(seat).map_err(|error| format!("'{seat}' is not a seat name: {error}"))?;
    if model.is_empty() {
        return Err(format!("'{text}' names no model after '='"));
    }
    Ok(SeatOverride {
        seat,
        model: model.to_owned(),
    })
}

/// The world as the decision list is checked against it.
pub fn roster(pack: &WorldPack) -> Roster {
    let class_ref = |name: &ClassName| {
        ClassRef::new(name.as_str()).expect("a class name is spelled as a class reference")
    };
    let seats = pack
        .seats()
        .iter()
        .map(|seat| {
            let tags = &pack.people()[seat].tags;
            let class = pack.classes().class_of(Classed {
                entity_type: EntityType::Person,
                tags,
            });
            SeatEntry {
                seat: seat.clone(),
                class: class_ref(&class),
            }
        })
        .collect();
    Roster {
        seats,
        classes: pack
            .classes()
            .definitions()
            .iter()
            .filter(|definition| definition.of == EntityType::Person)
            .map(|definition| class_ref(&definition.class))
            .collect(),
        implicit: [
            EntityType::Person,
            EntityType::Place,
            EntityType::Item,
            EntityType::Organization,
        ]
        .into_iter()
        .map(|entity_type| class_ref(&ClassName::implicit(entity_type)))
        .collect(),
        people: pack.people().keys().cloned().collect(),
    }
}

/// The list in effect for `pack`: `--decisions`, else the pack's file, else none — read and checked.
pub fn list(
    pack: &WorldPack,
    roster: &Roster,
    file: Option<&Path>,
) -> Result<(DecisionList<Model>, Origin), String> {
    if let Some(file) = file {
        let written = file.display().to_string();
        return DecisionList::read(file, &written, roster)
            .map(|list| (list, Origin::File))
            .map_err(|error| format!("[mineworld] --decisions: {error}"));
    }
    match pack.decisions() {
        None => Ok((DecisionList::none(), Origin::Implicit)),
        Some((written, path)) => DecisionList::read(&path, &written.to_string(), roster)
            .map(|list| (list, Origin::Pack))
            .map_err(|error| format!("[mineworld] {error}")),
    }
}

/// Every seat of `pack`, resolved under the operator's flags.
pub fn resolve(pack: &WorldPack, args: &DecisionArgs) -> Result<ResolvedDecisions, String> {
    let roster = roster(pack);
    let (list, origin) = list(pack, &roster, args.file.as_deref())?;
    list.resolve(&roster, origin, &args.seat_models)
        .map_err(|refusal| format!("[mineworld] {refusal}"))
}

/// What `mineworld decisions` was asked.
pub struct DecisionsRequest<'a> {
    pub world: &'a Path,
    pub args: DecisionArgs,
    pub json: bool,
    pub roots: &'a PackRoots,
}

/// `mineworld decisions <world>`: each seat's resolved model and memory choice, read exactly as `run`
/// reads them, with no world started and nothing written.
pub fn decisions(request: &DecisionsRequest<'_>) -> Result<(), String> {
    let pack = WorldPack::read_with(request.world, request.roots).map_err(described)?;
    let resolved = resolve(&pack, &request.args)?;
    let document = resolved.json(pack.id());
    if request.json {
        let text = serde_json::to_string_pretty(&document).map_err(|error| {
            format!("[mineworld] the decisions document did not serialize: {error}")
        })?;
        println!("{text}");
        return Ok(());
    }
    println!(
        "{} ({}): decisions {}",
        pack.name(),
        pack.id(),
        document.digest
    );
    for seat in &document.seats {
        let model = match (&seat.model.id, seat.model.chain.is_empty()) {
            (Some(id), true) => id.clone(),
            (_, false) => format!("{} ({RUN_HOSTS_NONE})", seat.model.chain.join(" → ")),
            (None, true) => "idle".to_owned(),
        };
        let memory = seat.memory.as_ref().map_or_else(
            || "memory: default".to_owned(),
            |memory| {
                let fallback = memory
                    .fallback
                    .as_ref()
                    .map_or_else(String::new, |fallback| format!(", fallback {fallback}"));
                format!(
                    "memory: {}{fallback} ({}:{}:{})",
                    memory.policy, memory.source.file, memory.source.line, memory.source.column
                )
            },
        );
        println!("  {:<10} {model}; {memory}", seat.seat);
    }
    Ok(())
}
