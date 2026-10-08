//! `mineworld packs list | show | validate` — every pack's package identity (`docs/DECISIONS.md`
//! `ARC-53`, `docs/MODULE_SPEC.md` §8.1).
//!
//! ```text
//! the build's code packs   every System Pack of the installed set, in its order, through
//!                          Capability::package(); then the controllers this binary composes
//! the data packs           for each --packs DIR, in the order given, every immediate subdirectory
//!                          holding world.yaml (read by the World Pack loader) or pack.yaml
//! ```
//!
//! Nothing is read from a directory that was not named. A package identity is never world state:
//! this module is the only reader of `PACKAGE`, and nothing it prints reaches a fact or a save.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use mineworld_contracts::{SystemId, WorldTime};
use mineworld_kernel::SystemVersion;
use mineworld_packages::{
    DataPack, Identity, PackType, Package, PackageError, check_style_manifest, distinct, packs_in,
    read_pack_file,
};
use mineworld_worldpack::catalog::AVAILABLE;
use mineworld_worldpack::{MANIFEST, WorldPack};

use crate::described;

/// The Controller Packs this binary composes (it is the composition root, `main.rs`).
const CONTROLLERS: [Package; 1] = [mineworld_rule_controller::PACKAGE];

/// Where a listed pack comes from.
enum Source {
    /// A System Pack of this build's installed set.
    System {
        id: SystemId,
        version: SystemVersion,
    },
    /// A controller this binary composes.
    Controller,
    /// A data pack in a directory given on the command line.
    Directory(PathBuf),
}

impl Source {
    fn describe(&self) -> String {
        match self {
            Self::System { id, .. } => format!("this build (system {id})"),
            Self::Controller => "this build (a controller)".to_owned(),
            Self::Directory(dir) => dir.display().to_string(),
        }
    }
}

/// One pack, identified, with where it came from.
struct Listed {
    identity: Identity,
    source: Source,
}

fn refused(error: PackageError) -> String {
    format!("[mineworld] {error}")
}

/// Every pack this build and these directories provide, each identity checked, no id twice.
fn gather(roots: &[PathBuf]) -> Result<Vec<Listed>, String> {
    let mut listed = Vec::new();
    for capability in AVAILABLE {
        listed.push(Listed {
            identity: Identity::of_code_pack(&capability.package(), PackType::SystemPack)
                .map_err(refused)?,
            source: Source::System {
                id: capability.id(),
                version: capability.version(),
            },
        });
    }
    for controller in &CONTROLLERS {
        listed.push(Listed {
            identity: Identity::of_code_pack(controller, PackType::ControllerPack)
                .map_err(refused)?,
            source: Source::Controller,
        });
    }
    for root in roots {
        for pack in packs_in(root).map_err(refused)? {
            listed.push(Listed {
                identity: identity_of(&pack)?,
                source: Source::Directory(pack.dir().to_path_buf()),
            });
        }
    }
    let origins: Vec<String> = listed.iter().map(|pack| pack.source.describe()).collect();
    distinct(
        listed
            .iter()
            .zip(&origins)
            .map(|(pack, origin)| (&pack.identity, origin.as_str())),
    )
    .map_err(refused)?;
    Ok(listed)
}

/// A data pack's identity: a world's through the World Pack loader, which owns `world.yaml`; any other
/// through its `pack.yaml`.
fn identity_of(pack: &DataPack) -> Result<Identity, String> {
    match pack {
        DataPack::World(dir) => world_identity(&WorldPack::read(dir).map_err(described)?),
        DataPack::PackFile(dir) => read_pack_file(dir).map_err(refused),
    }
}

/// A World Pack's identity, from the fields the loader read and checked, all required here.
fn world_identity(world: &WorldPack) -> Result<Identity, String> {
    let fields = world.package_fields();
    Identity::of_world(
        &world.root().join(MANIFEST),
        world.id(),
        fields.version.clone(),
        fields.license.clone(),
        fields.mineworld.clone(),
    )
    .map_err(refused)
}

fn authors(identity: &Identity) -> String {
    if identity.authors.is_empty() {
        "—".to_owned()
    } else {
        identity.authors.join(", ")
    }
}

/// `mineworld packs list`: one line per pack, then a count.
pub fn list(roots: &[PathBuf]) -> Result<(), String> {
    let listed = gather(roots)?;
    let mut out = String::new();
    for pack in &listed {
        let identity = &pack.identity;
        let tail = match &pack.source {
            Source::System { id, .. } => format!("system {id}"),
            Source::Controller => String::new(),
            Source::Directory(dir) => dir.display().to_string(),
        };
        let line = format!(
            "{:<17}  {:<26}  {:<6}  {}  {}  {tail}",
            identity.kind.as_str(),
            identity.id.as_str(),
            identity.version.to_string(),
            identity.license,
            authors(identity),
        );
        let _ = writeln!(out, "{}", line.trim_end());
    }
    let _ = writeln!(out, "{} packs", listed.len());
    print!("{out}");
    Ok(())
}

/// `mineworld packs show <id>`: every field of one pack.
pub fn show(id: &str, roots: &[PathBuf]) -> Result<(), String> {
    let listed = gather(roots)?;
    let Some(pack) = listed.iter().find(|pack| pack.identity.id.as_str() == id) else {
        let known: Vec<&str> = listed
            .iter()
            .map(|pack| pack.identity.id.as_str())
            .collect();
        return Err(format!(
            "[mineworld] no pack has the id '{id}' in this build or the directories given \
             (there are: {})",
            known.join(", ")
        ));
    };
    print!("{}", described_identity(&pack.identity, &pack.source));
    Ok(())
}

fn described_identity(identity: &Identity, source: &Source) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}", identity.id);
    let _ = writeln!(out, "  type        {}", identity.kind);
    let _ = writeln!(out, "  version     {}", identity.version);
    let _ = writeln!(out, "  license     {}", identity.license);
    let _ = writeln!(out, "  authors     {}", authors(identity));
    let _ = writeln!(
        out,
        "  repository  {}",
        identity.repository.as_deref().unwrap_or("—")
    );
    let range = identity
        .mineworld
        .as_ref()
        .map_or_else(|| "—".to_owned(), ToString::to_string);
    let _ = writeln!(out, "  mineworld   {range}");
    match source {
        Source::System { id, version } => {
            let _ = writeln!(out, "  system      {id} (SystemVersion {})", version.get());
            let _ = writeln!(out, "  source      this build");
        }
        Source::Controller => {
            let _ = writeln!(out, "  source      this build");
        }
        Source::Directory(dir) => {
            let _ = writeln!(out, "  source      {}", dir.display());
        }
    }
    out
}

/// `mineworld packs validate <directory>`: one data pack, its package fields all required, then its
/// content — a world read and loaded as `validate` does; a presentation pack's style manifest.
pub fn validate(dir: &Path) -> Result<(), String> {
    let pack = DataPack::of(dir).map_err(refused)?.ok_or_else(|| {
        refused(PackageError::NotAPack {
            dir: dir.to_path_buf(),
        })
    })?;
    let identity = match &pack {
        DataPack::World(dir) => {
            let world = WorldPack::read(dir).map_err(described)?;
            let identity = world_identity(&world)?;
            world.load(WorldTime::EPOCH).map_err(described)?;
            identity
        }
        DataPack::PackFile(dir) => {
            let identity = read_pack_file(dir).map_err(refused)?;
            check_style_manifest(dir).map_err(refused)?;
            identity
        }
    };
    print!(
        "{}",
        described_identity(&identity, &Source::Directory(dir.to_path_buf()))
    );
    println!("{} is a valid {}.", dir.display(), identity.kind);
    Ok(())
}
