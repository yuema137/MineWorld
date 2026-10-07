//! `mineworld create` — a new, minimal World Pack in a directory that does not exist yet.
//!
//! The template is embedded in the binary, so the command works away from this repository. The only
//! text substituted is the world's id, and it is checked as a key first — lowercase letters, digits,
//! `-` and `_` — so nothing can be injected into the YAML. The pack written is read and loaded before
//! success is reported: a template that had drifted from the pack format would fail here, not in the
//! author's hands (step-08 HD-11, I-8).

use std::path::Path;

use mineworld_contracts::{EntityKey, WorldTime};
use mineworld_worldpack::WorldPack;

use crate::described;

/// The template's files: where each goes, and what it says.
const TEMPLATE: [(&str, &str); 4] = [
    (
        "world.yaml",
        include_str!("../templates/new-world/world.yaml"),
    ),
    (
        "places/home.yaml",
        include_str!("../templates/new-world/places/home.yaml"),
    ),
    (
        "people/first.yaml",
        include_str!("../templates/new-world/people/first.yaml"),
    ),
    (
        "people/second.yaml",
        include_str!("../templates/new-world/people/second.yaml"),
    ),
];

/// Writes a new World Pack into `directory`, which must not exist.
pub fn create(directory: &Path) -> Result<(), String> {
    if directory.exists() {
        return Err(format!(
            "[mineworld] {} already exists; create writes a new World Pack and never overwrites one",
            directory.display()
        ));
    }
    let id = directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            format!(
                "[mineworld] {} has no name to use as an id",
                directory.display()
            )
        })?;
    EntityKey::new(id).map_err(|error| {
        format!(
            "[mineworld] '{id}' cannot be a world's id, because a World Pack's id is its directory's \
             name: {error}"
        )
    })?;

    for (file, text) in TEMPLATE {
        let path = directory.join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!("[mineworld] cannot create {}: {error}", parent.display())
            })?;
        }
        std::fs::write(&path, text.replace("{id}", id))
            .map_err(|error| format!("[mineworld] cannot write {}: {error}", path.display()))?;
    }

    let pack = WorldPack::read(directory).map_err(described)?;
    let loaded = pack.load(WorldTime::EPOCH).map_err(described)?;
    println!(
        "created {}: World Pack '{}' — {} place(s), {} person(s), seats {}; {} genesis fact(s)",
        directory.display(),
        pack.id(),
        pack.places().len(),
        pack.people().len(),
        pack.seats()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", "),
        loaded.genesis().len(),
    );
    println!(
        "next: mineworld run {} --headless --seed 1 --days 7",
        directory.display()
    );
    Ok(())
}
