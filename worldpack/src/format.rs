//! The authored shape: exactly what a `world.yaml` and a person, place, item or organization file may
//! say.
//!
//! These types are the pack format. They are deliberately *not* the contract types they become:
//! authored YAML is written by a person and read once, while a contract value is passed between
//! systems and persisted, and tying the two together would mean a change to either shape breaking
//! the other.
//!
//! Three properties are the whole design of this module.
//!
//! **Unknown fields are refused, not ignored.** Every struct carries `deny_unknown_fields`, and a
//! content file — whose legal keys also include the sections System Packs own
//! (`DECISIONS.md` `ARC-31`) — is read by [`crate::content`], which refuses any other key the same
//! way. So `nmae: Alice` is an error at the line that wrote it rather than an entity with no name and
//! no reason why. `docs/MODULE_SPEC.md` §4's full model is wider than what MVP-0 implements, and §4.1
//! names the implemented subset — a field outside it is refused rather than silently accepted,
//! because an accepted field an author believes in is worse than a refused one.
//!
//! **Validated contract types are used wherever one exists.** [`EntityKey`], [`SystemId`],
//! [`Tags`] and [`Millimetres`] all validate as they deserialize, so an illegal name is refused by
//! the contract's own rule at the exact position in the file that wrote it. The alternative — plain
//! `String` fields checked afterwards — would be a second implementation of a rule
//! `contracts/src/ids.rs` is explicit about keeping in one place.
//!
//! **A key is stated once.** A person file does not name the person: `world.yaml`'s `population`
//! lists the keys, and each key names its file. Two copies of one fact in a pack are two chances for
//! them to disagree, which is the reason `EntityRegistry` derives its key index rather than storing
//! it.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_authoring::{Attached, AuthoredConfiguration, AuthoredContent, SectionName};
use mineworld_contracts::{EntityKey, Millimetres, SystemId, Tags};
use mineworld_packages::{Compatibility, License, PackId, Version};
use serde::Deserialize;

use crate::catalog::Capability;

/// `world.yaml`: what this world is, what it is composed of, and who is in it.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example_world())]
pub struct WorldManifest {
    /// The world's own identity: its id and its name.
    pub world: WorldIdentity,
    /// The System Packs this world enables, in installation order.
    ///
    /// Order is the pack's statement rather than something the loader sorts: registration order is
    /// reduction order, reduction order is observable in the event log, and a loader that sorted
    /// would be deciding a world's semantics on the author's behalf (`BD-4`).
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::SystemName>")]
    pub systems: Vec<SystemId>,
    /// The places this world has. Each key names a file in `places/`.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    pub places: Vec<EntityKey>,
    /// The people this world has. Each key names a file in `people/`.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    pub population: Vec<EntityKey>,
    /// The item kinds this world has. Each key names a file in `items/`: a kind of thing, not one
    /// object.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    pub items: Vec<EntityKey>,
    /// The organizations this world has. Each key names a file in `organizations/`.
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    pub organizations: Vec<EntityKey>,
    /// The seats a client may occupy, each the key of a person in `population`.
    ///
    /// The roster is the world's, never the client's: a client names a seat and the server resolves
    /// which entity it is, so there is no frame in which a client names an entity
    /// (`server/src/host.rs`).
    #[serde(default)]
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    pub seats: Vec<EntityKey>,
    /// The framework versions this world is authored for, such as "^0.1"; when stated, a framework
    /// outside it is refused by name.
    ///
    /// Optional to the loader (`DECISIONS.md` `ARC-53`). Never world state.
    #[serde(default)]
    pub mineworld: Option<Compatibility>,
    /// Every pack this world uses that is not bundled, with the versions it accepts, such as
    /// `modern-goods: "^0.1"`; each is found in a directory given with --packs or MINEWORLD_PACKS.
    ///
    /// (`DECISIONS.md` `ARC-54`.) Packs and versions only: which systems are enabled is `systems`, and
    /// how a pack is configured is not this key's. Resolved in the build and the pack roots; never
    /// world state.
    #[serde(default)]
    pub requires: BTreeMap<PackId, Compatibility>,
    /// The files in `configure/` this world has, in seeding order: each key names
    /// `configure/<key>.yaml` — an enabled System Pack's id, or `classes` or `packages`.
    ///
    /// (`DECISIONS.md` `ARC-61`.) Each file's owner decodes and seeds it. Absent means none.
    #[serde(default)]
    pub configure: Vec<ConfigurationKey>,
    /// How a host paces this world when it serves it: `time_scale`, the world seconds that pass per
    /// wall second unless the host says otherwise.
    ///
    /// (`DECISIONS.md` `ARC-69` item 9.) Host pacing content: never seeded, never read by a pack or a
    /// controller, ignored by `run`.
    #[serde(default)]
    pub hosting: Option<Hosting>,
    /// The decision list: a file of this pack, such as decisions.yaml, saying which decision model
    /// drives each seat. Absent, every seat is driven by the rule model.
    ///
    /// (`DECISIONS.md` `ARC-84`, `MODULE_SPEC.md` §4.1.) A file of this pack that this crate checks
    /// exists and lies inside the pack, and never parses: its format belongs to `mineworld-decision`,
    /// read by hosts. Never world state.
    #[serde(default)]
    pub decisions: Option<DecisionsFile>,
}

/// How a host paces the world it serves: a time scale from 1 to 24, such as 12 for a two-hour day.
///
/// `world.yaml`'s `hosting:` block (`ARC-69`, step-19 QTW-4).
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Hosting {
    /// World seconds per wall second when the host does not say otherwise, a whole number from 1 to
    /// 24, so a day lasts 24 h divided by it. Absent, the host's own default, 1.
    ///
    /// A fast-forward is never a pack's to state (`ARC-69` item 5), so a larger value is refused at
    /// load, naming this key.
    #[serde(default)]
    pub time_scale: Option<u32>,
}

/// A file of the pack, relative to its root, written with / on every platform, such as decisions.yaml:
/// no \, no leading /, no empty, . or .. part, no :.
///
/// `world.yaml`'s `decisions:` (`MODULE_SPEC.md` §4.1). Each refusal happens as it decodes, so the same
/// text names the same file on macOS, Linux and Windows and cannot climb out of the pack; that its real
/// path, links followed, stays inside is checked at read.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String")]
#[schemars(with = "String")]
pub struct DecisionsFile(Vec<String>);

impl TryFrom<String> for DecisionsFile {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused = |why: &str| format!("decisions: '{text}' {why}");
        if text.contains('\\') {
            return Err(refused("uses '\\'; write paths with '/' on every platform"));
        }
        if text.starts_with('/') {
            return Err(refused(
                "is absolute; name a file of this pack, relative to its root",
            ));
        }
        let components: Vec<String> = text.split('/').map(str::to_owned).collect();
        for component in &components {
            if component.is_empty() || component == "." || component == ".." {
                return Err(refused("has an empty, '.' or '..' component"));
            }
            if component.contains(':') {
                return Err(refused("names a drive or a stream (':')"));
            }
        }
        Ok(Self(components))
    }
}

impl DecisionsFile {
    /// The file under a pack's root, in the platform's own form.
    pub fn path_under(&self, root: &std::path::Path) -> PathBuf {
        self.0
            .iter()
            .fold(root.to_path_buf(), |path, component| path.join(component))
    }

    /// The directory of the pack's root the file lies in, when it lies in one: the first component of a
    /// path of more than one. A data directory rule admits it (`ARC-121` item 1).
    pub(crate) fn directory(&self) -> Option<&str> {
        (self.0.len() > 1).then(|| self.0[0].as_str())
    }
}

impl core::fmt::Display for DecisionsFile {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0.join("/"))
    }
}

/// The example the generated reference prints for `world.yaml`.
fn example_world() -> serde_json::Value {
    serde_json::json!({
        "world": { "id": "corner-shop", "name": "Corner Shop" },
        "systems": ["presence", "movement", "conversation"],
        "places": ["home", "shop"],
        "population": ["ana", "ben"],
        "seats": ["ben"],
    })
}

/// A key of `world.yaml`'s `configure:` list, naming `configure/<key>.yaml` (`DECISIONS.md`
/// `ARC-61`): an enabled System Pack's id, or a framework key.
///
/// Its own type rather than a [`SystemId`], because not every key names a system: `classes` is the
/// world's entity classes (`ARC-64`) and `packages` its licence policy (`ARC-55` note), each owned by a
/// framework crate. It is spelled as a system id is and validates at its line in `world.yaml`; whether
/// it names a system is decided after.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(transparent)]
pub struct ConfigurationKey(SystemId);

impl ConfigurationKey {
    /// The key, as the system id it names.
    pub fn system(&self) -> &SystemId {
        &self.0
    }

    /// The key as written.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Spelled as a system id is (`ARC-90`).
impl schemars::JsonSchema for ConfigurationKey {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ConfigurationKey")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": mineworld_authoring::schema::IDENTIFIER_PATTERN,
            "minLength": 1,
            "maxLength": 64,
            "description": "An enabled System Pack's id (such as calendar), or classes (the world's \
                            entity classes) or packages (its licence policy). Each names \
                            configure/<key>.yaml and is listed once.",
        })
    }
}

impl core::fmt::Display for ConfigurationKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A world's identity: the name a person reads, and the id everything else uses.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WorldIdentity {
    /// The World Pack's id, which must be its directory's name.
    ///
    /// Free text rather than a validated identifier, because that is what it becomes:
    /// `Metadata::source_pack` is documented as free text until "pack loading becomes a contract"
    /// (`contracts/src/entity.rs`). What is checked is that it agrees with the directory, so the pack
    /// has one identity rather than two.
    pub id: String,
    /// What the world is called, for a person. Not world state: no system reads it, and nothing in a
    /// simulation may branch on it.
    pub name: String,
    /// The pack's release version, semver (`DECISIONS.md` `ARC-53`). Optional to the loader, checked
    /// when stated; required by `mineworld packs validate`. Never world state.
    #[serde(default)]
    pub version: Option<Version>,
    /// The pack's licence, an SPDX expression (`ARC-53`). Optional to the loader, checked when
    /// stated; required by `mineworld packs validate`. Never world state.
    #[serde(default)]
    pub license: Option<License>,
}

/// One section a content file carries: a top-level key a System Pack owns (`DECISIONS.md` `ARC-31`).
///
/// The loader holds it without knowing what it means. Its fate is decided while the file is read,
/// because only then is it known whether the owner is enabled and whether this kind of file may carry
/// it — and a section that will be refused is not decoded, so the author hears the refusal that
/// matters rather than a complaint about the inside of a section that could never be used.
#[derive(Debug, Clone)]
pub struct FoundSection {
    /// The capability that owns it.
    pub owner: Capability,
    /// Its key.
    pub name: SectionName,
    /// What became of it.
    pub state: SectionState,
}

/// What became of a section as its file was read.
#[derive(Debug, Clone)]
pub enum SectionState {
    /// Decoded, and so validated, by its owner's own type.
    Decoded(Arc<dyn AuthoredContent>),
    /// Its owner is a system this build provides that the world does not enable.
    OwnerNotEnabled,
    /// Its owner does not let this kind of file carry it.
    NotCarriedHere,
}

/// One System Pack's world-level configuration, read from `configure/<id>.yaml` and decoded — and so
/// validated — by its owner's own type (`DECISIONS.md` `ARC-61`). The loader holds it without knowing
/// what it means.
#[derive(Debug, Clone)]
pub struct FoundConfiguration {
    /// The capability that owns it.
    pub owner: Capability,
    /// The file it was read from.
    pub path: PathBuf,
    /// The decoded value.
    pub configuration: Arc<dyn AuthoredConfiguration>,
    /// The bytes of the `data/` files it names (`ARC-61` note), read by the loader.
    pub attached: Attached,
}

/// A file in `people/`: one Person, as authored.
///
/// Read through [`crate::content`] rather than a derived `Deserialize`, because which top-level keys
/// are legal depends on the sections this build's System Packs own — and an unknown key is still
/// refused rather than ignored.
#[derive(Debug, Clone, Default)]
pub struct AuthoredPerson {
    /// The labels this person carries. An open vocabulary the contract validates, and the only thing
    /// in this file a system may read: a system decides whether an entity is of interest to it by
    /// reading tags and components, never by reading prose.
    pub tags: Tags,
    /// A note from whoever authored them, for a person debugging a world. Never gameplay state.
    pub note: Option<String>,
    /// Where this person starts.
    ///
    /// Optional: a person with no location is not "nowhere", they are somebody the presence system
    /// has not been told about, which `systems/presence` treats as knowing nothing rather than as an
    /// error.
    pub location: Option<AuthoredLocation>,
    /// The sections this file carries, in the order it states them.
    pub sections: Vec<FoundSection>,
}

/// A file in `places/`: one Place, as authored.
#[derive(Debug, Clone, Default)]
pub struct AuthoredPlace {
    /// The labels this place carries. These reach a client through the observation, which is what
    /// lets it choose how to draw a café without the simulation knowing it is one.
    pub tags: Tags,
    /// A note from whoever authored it.
    pub note: Option<String>,
    /// The places this one opens onto, each through one doorway.
    ///
    /// A fact about where one can walk, which the `movement` system owns: each becomes a genesis
    /// fact that system reduces into both places, so a passage is stated once, in either of the two
    /// files, and holds both ways. It states no rule — how far from a doorway a person may pass is
    /// movement's decision, not this file's.
    pub passages: Vec<AuthoredPassage>,
    /// The sections this file carries, in the order it states them.
    pub sections: Vec<FoundSection>,
}

/// A file in `items/`: one item kind, as authored (`ARC-36`). What anybody holds of it is a count of
/// this kind, kept by whichever System Pack owns holdings; the file declares no single object.
///
/// Its own struct rather than one shared with [`AuthoredOrganization`]: the two are distinct concepts
/// that happen to have the same fields today.
#[derive(Debug, Clone, Default)]
pub struct AuthoredItem {
    /// The labels this kind carries.
    pub tags: Tags,
    /// A note from whoever authored it.
    pub note: Option<String>,
    /// The sections this file carries, in the order it states them.
    pub sections: Vec<FoundSection>,
}

/// A file in `organizations/`: one Organization, as authored.
#[derive(Debug, Clone, Default)]
pub struct AuthoredOrganization {
    /// The labels this organization carries.
    pub tags: Tags,
    /// A note from whoever authored it.
    pub note: Option<String>,
    /// The sections this file carries, in the order it states them.
    pub sections: Vec<FoundSection>,
}

/// One doorway from the place whose file states it to another place.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPassage {
    /// The place it leads to, by its authoring key. A passage is stated once, in either place's file,
    /// and goes both ways.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    pub to: EntityKey,
    /// Where the doorway is in this place, if the world models positions.
    #[serde(default)]
    pub here: Option<AuthoredPosition>,
    /// Where the same doorway is in `to`, if the world models positions.
    #[serde(default)]
    pub there: Option<AuthoredPosition>,
}

/// Where somebody starts: a place, and optionally where in it and which way they face.
///
/// The same two scales [`Location`](mineworld_contracts::Location) has, for the same reason: a
/// headless world and a 2D client need only the place, and a 3D client needs the position inside it.
/// A pack that omits `position` is not approximating — the refinement is absent
/// (`ENGINEERING_RULES.md` §5).
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredLocation {
    /// Which place, by its authoring key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    pub place: EntityKey,
    /// Where in it, in millimetres from the place's own origin.
    #[serde(default)]
    pub position: Option<AuthoredPosition>,
    /// Which way they face, in millidegrees clockwise (90000 is a quarter turn; 360000 is a whole
    /// turn).
    ///
    /// Validated and normalized by [`Orientation`](mineworld_contracts::Orientation).
    #[serde(default)]
    pub facing: Option<i32>,
}

/// A position inside a place: integer millimetres on the world's own axes.
///
/// Integers, not floats, and this is not a style choice: a position reaches the event log, and
/// floating-point arithmetic is not reproducible across platforms, so a world replayed elsewhere
/// would diverge (`AC-12`, `DD-5`).
#[derive(Debug, Clone, Copy, Default, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPosition {
    /// Millimetres across the floor from the place's origin.
    #[serde(default)]
    #[schemars(with = "mineworld_authoring::schema::Length")]
    pub x: Millimetres,
    /// Millimetres across the floor from the place's origin, at right angles to `x`.
    #[serde(default)]
    #[schemars(with = "mineworld_authoring::schema::Length")]
    pub y: Millimetres,
    /// Millimetres of height. Absent means the floor, which is what a world that does not model height
    /// means.
    #[serde(default)]
    #[schemars(with = "mineworld_authoring::schema::Length")]
    pub z: Millimetres,
}
