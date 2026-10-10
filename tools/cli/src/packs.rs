//! `mineworld packs list | show | validate | resolve` — every pack's package identity, and a world's
//! composition (`docs/DECISIONS.md` `ARC-53`, `ARC-54`, `ARC-55`; `docs/MODULE_SPEC.md` §8.1).
//!
//! ```text
//! the build's code packs   every System Pack of the installed set, in its order, through
//!                          Capability::package(); then the controllers this binary composes
//! the data packs           for each pack root (--packs DIR, then MINEWORLD_PACKS), in order, every
//!                          immediate subdirectory holding world.yaml (read by the World Pack loader)
//!                          or pack.yaml
//! ```
//!
//! Nothing is read from a directory that was not named. A package identity is never world state:
//! this module is the only reader of `PACKAGE` and of a world's composition, and nothing it prints
//! reaches a fact or a save.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, SystemId, WorldTime};
use mineworld_kernel::SystemVersion;
use mineworld_packages::{
    DataPack, Identity, LicencePolicy, PackType, Package, PackageError, Source,
    check_style_manifest, distinct, read_pack_file,
};
use mineworld_worldpack::catalog::AVAILABLE;
use mineworld_worldpack::{MANIFEST, PackRoots, WorldPack, validate_entity_pack};

use crate::described;

/// The Controller Packs this binary composes (it is the composition root, `main.rs`).
const CONTROLLERS: [Package; 1] = [mineworld_rule_controller::PACKAGE];

/// Where a listed pack comes from.
enum Origin {
    /// A System Pack of this build's installed set, bundled or third-party (`ARC-54` point 2).
    System {
        id: SystemId,
        version: SystemVersion,
        bundled: bool,
    },
    /// A controller this binary composes, bundled or third-party.
    Controller { bundled: bool },
    /// A data pack in a pack root.
    Directory(PathBuf),
}

/// A code pack's origin in one word (`ARC-66` point 4): compiled from this workspace, or not.
const fn origin_word(bundled: bool) -> &'static str {
    if bundled { "bundled" } else { "third-party" }
}

impl Origin {
    fn describe(&self) -> String {
        match self {
            Self::System { id, .. } => format!("this build (system {id})"),
            Self::Controller { .. } => "this build (a controller)".to_owned(),
            Self::Directory(dir) => dir.display().to_string(),
        }
    }
}

/// One pack, identified, with where it came from.
struct Listed {
    identity: Identity,
    origin: Origin,
}

fn refused(error: PackageError) -> String {
    format!("[mineworld] {error}")
}

/// Where a requirement was met, as `validate` and `resolve` print it.
///
/// A directory is written with `/` on every OS, so the printed composition is the same text on every
/// platform (AC-8 hashes it; `source_path` follows the same rule). On Windows `\` and `/` are both
/// separators and neither may appear in a name, so the rewrite loses nothing; elsewhere `\` is an
/// ordinary character in a name and is left alone.
pub fn source(source: &Source) -> String {
    match source {
        Source::Build { bundled: true } => "this build, bundled".to_owned(),
        Source::Build { bundled: false } => "this build, third-party".to_owned(),
        Source::Directory(dir) if cfg!(windows) => dir.display().to_string().replace('\\', "/"),
        Source::Directory(dir) => dir.display().to_string(),
    }
}

/// Every pack this build and the pack roots provide, each identity checked, no id twice.
fn gather(roots: &PackRoots) -> Result<Vec<Listed>, String> {
    let mut listed = Vec::new();
    for capability in AVAILABLE {
        listed.push(Listed {
            identity: Identity::of_code_pack(&capability.package(), PackType::SystemPack)
                .map_err(refused)?,
            origin: Origin::System {
                id: capability.id(),
                version: capability.version(),
                bundled: capability.package().bundled(),
            },
        });
    }
    for controller in &CONTROLLERS {
        listed.push(Listed {
            identity: Identity::of_code_pack(controller, PackType::ControllerPack)
                .map_err(refused)?,
            origin: Origin::Controller {
                bundled: controller.bundled(),
            },
        });
    }
    for pack in roots.packs().map_err(refused)? {
        listed.push(Listed {
            identity: identity_of(&pack, roots)?,
            origin: Origin::Directory(pack.dir().to_path_buf()),
        });
    }
    let origins: Vec<String> = listed.iter().map(|pack| pack.origin.describe()).collect();
    distinct(
        listed
            .iter()
            .zip(&origins)
            .map(|(pack, origin)| (&pack.identity, origin.as_str())),
    )
    .map_err(refused)?;
    Ok(listed)
}

/// A data pack's identity: a world's through the World Pack loader, which owns `world.yaml` and
/// resolves the world's own requirements in the same roots; any other through its `pack.yaml`.
fn identity_of(pack: &DataPack, roots: &PackRoots) -> Result<Identity, String> {
    match pack {
        DataPack::World(dir) => {
            world_identity(&WorldPack::read_with(dir, roots).map_err(described)?)
        }
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
pub fn list(roots: &PackRoots) -> Result<(), String> {
    let listed = gather(roots)?;
    let mut out = String::new();
    for pack in &listed {
        let identity = &pack.identity;
        let tail = match &pack.origin {
            Origin::System { id, bundled, .. } => format!("{}  system {id}", origin_word(*bundled)),
            Origin::Controller { bundled } => origin_word(*bundled).to_owned(),
            Origin::Directory(dir) => dir.display().to_string(),
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
pub fn show(id: &str, roots: &PackRoots) -> Result<(), String> {
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
    print!("{}", described_identity(&pack.identity, &pack.origin));
    Ok(())
}

fn described_identity(identity: &Identity, origin: &Origin) -> String {
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
    match origin {
        Origin::System {
            id,
            version,
            bundled,
        } => {
            let _ = writeln!(out, "  system      {id} (SystemVersion {})", version.get());
            let _ = writeln!(out, "  source      this build ({})", origin_word(*bundled));
        }
        Origin::Controller { bundled } => {
            let _ = writeln!(out, "  source      this build ({})", origin_word(*bundled));
        }
        Origin::Directory(dir) => {
            let _ = writeln!(out, "  source      {}", dir.display());
        }
    }
    out
}

/// `mineworld packs validate <directory>`: one data pack, its package fields all required, its licence
/// judged by the default policy (a World Pack's by its own, ARC-55 note), then its content — a world read (its requirements resolved in the
/// roots) and loaded as `validate` does; a presentation pack's style manifest.
pub fn validate(dir: &Path, roots: &PackRoots) -> Result<(), String> {
    let pack = DataPack::of(dir).map_err(refused)?.ok_or_else(|| {
        refused(PackageError::NotAPack {
            dir: dir.to_path_buf(),
        })
    })?;
    // A World Pack is judged by its own policy (`configure/packages.yaml`, ARC-55 note), after it is
    // read and loaded; any other pack by the default, before its content — then its own framework range
    // (ARC-54 note, F-Ed1), then its content.
    let (identity, kinds) = match &pack {
        DataPack::World(dir) => {
            let world = WorldPack::read_with(dir, roots).map_err(described)?;
            let identity = world_identity(&world)?;
            world.load(WorldTime::EPOCH).map_err(described)?;
            world
                .licence_policy()
                .judge(identity.id.as_str(), &identity.license)
                .map_err(refused)?;
            (identity, None)
        }
        DataPack::PackFile(dir) => {
            let identity = read_pack_file(dir).map_err(refused)?;
            LicencePolicy::default()
                .judge(identity.id.as_str(), &identity.license)
                .map_err(refused)?;
            identity.require_framework().map_err(refused)?;
            let kinds = match identity.kind {
                // An Entity Pack's kinds, read against this build's whole installed set (ARC-71).
                PackType::EntityPack => Some(validate_entity_pack(dir).map_err(described)?),
                _ => {
                    check_style_manifest(dir).map_err(refused)?;
                    None
                }
            };
            (identity, kinds)
        }
    };
    print!(
        "{}",
        described_identity(&identity, &Origin::Directory(dir.to_path_buf()))
    );
    if let Some(kinds) = kinds {
        let kinds: Vec<&str> = kinds.iter().map(EntityKey::as_str).collect();
        println!("  items       {}", kinds.join(", "));
    }
    println!("{} is a valid {}.", dir.display(), identity.kind);
    Ok(())
}

/// `mineworld packs resolve <world>`: the world's composition, or its first refusal.
pub fn resolve(world: &Path, roots: &PackRoots) -> Result<(), String> {
    let pack = WorldPack::read_with(world, roots).map_err(described)?;
    let composition = pack.composition();
    let mut out = String::new();
    let _ = writeln!(out, "{} ({})", pack.name(), pack.id());
    let range = composition
        .mineworld
        .as_ref()
        .map_or_else(|| "not stated".to_owned(), |r| format!("\"{r}\""));
    let _ = writeln!(
        out,
        "  framework  {} (mineworld: {range})",
        composition.framework
    );
    for required in &composition.required {
        let _ = writeln!(
            out,
            "  requires   {} \"{}\" → {} {} ({})",
            required.identity.id,
            required.range,
            required.identity.kind,
            required.identity.version,
            source(&required.source),
        );
    }
    for system in &composition.systems {
        let kind = if system.bundled {
            "bundled"
        } else {
            "third-party"
        };
        let _ = writeln!(
            out,
            "  system     {} → {} {} ({kind})",
            system.system, system.identity.id, system.identity.version,
        );
    }
    let _ = writeln!(out, "{} resolves.", world.display());
    print!("{out}");
    Ok(())
}
