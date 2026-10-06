//! The authored shape: exactly what a `world.yaml`, a person file and a place file may say.
//!
//! These types are the pack format. They are deliberately *not* the contract types they become:
//! authored YAML is written by a person and read once, while a contract value is passed between
//! systems and persisted, and tying the two together would mean a change to either shape breaking
//! the other.
//!
//! Three properties are the whole design of this module.
//!
//! **Unknown fields are refused, not ignored.** Every struct carries `deny_unknown_fields`, so
//! `nmae: Alice` is an error at the line that wrote it rather than an entity with no name and no
//! reason why. `docs/MODULE_SPEC.md` §4's full model is wider than what MVP-0 implements, and §4.1
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

use mineworld_contracts::{EntityKey, Millimetres, SystemId, Tags};
use serde::Deserialize;

/// `world.yaml`: what this world is, what it is composed of, and who is in it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldManifest {
    /// The world's own identity.
    pub world: WorldIdentity,
    /// The systems this world enables, in installation order.
    ///
    /// Order is the pack's statement rather than something the loader sorts: registration order is
    /// reduction order, reduction order is observable in the event log, and a loader that sorted
    /// would be deciding a world's semantics on the author's behalf (`BD-4`).
    #[serde(default)]
    pub systems: Vec<SystemId>,
    /// The places this world has. Each key names a file in `places/`.
    #[serde(default)]
    pub places: Vec<EntityKey>,
    /// The people this world has. Each key names a file in `people/`.
    #[serde(default)]
    pub population: Vec<EntityKey>,
    /// The seats a client may occupy, each an existing person's key.
    ///
    /// The roster is the world's, never the client's: a client names a seat and the server resolves
    /// which entity it is, so there is no frame in which a client names an entity
    /// (`server/src/host.rs`).
    #[serde(default)]
    pub seats: Vec<EntityKey>,
}

/// A world's identity: the name a person reads, and the id everything else uses.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldIdentity {
    /// The pack's id, which must be its directory's name.
    ///
    /// Free text rather than a validated identifier, because that is what it becomes:
    /// `Metadata::source_pack` is documented as free text until "pack loading becomes a contract"
    /// (`contracts/src/entity.rs`). What is checked is that it agrees with the directory, so the pack
    /// has one identity rather than two.
    pub id: String,
    /// What the world is called, for a person. Not world state: no system reads it, and nothing in a
    /// simulation may branch on it.
    pub name: String,
}

/// A file in `people/`: one Person, as authored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPerson {
    /// The labels this person carries. An open vocabulary the contract validates, and the only thing
    /// in this file a system may read: a system decides whether an entity is of interest to it by
    /// reading tags and components, never by reading prose.
    #[serde(default)]
    pub tags: Tags,
    /// A note from whoever authored them, for a person debugging a world. Never gameplay state.
    #[serde(default)]
    pub note: Option<String>,
    /// Where this person starts.
    ///
    /// Optional: a person with no location is not "nowhere", they are somebody the presence system
    /// has not been told about, which `systems/presence` treats as knowing nothing rather than as an
    /// error.
    #[serde(default)]
    pub location: Option<AuthoredLocation>,
}

/// A file in `places/`: one Place, as authored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPlace {
    /// The labels this place carries. These reach a client through the observation, which is what
    /// lets it choose how to draw a café without the simulation knowing it is one.
    #[serde(default)]
    pub tags: Tags,
    /// A note from whoever authored it.
    #[serde(default)]
    pub note: Option<String>,
    /// The places this one opens onto, each through one doorway.
    ///
    /// A fact about where one can walk, which the `movement` system owns: each becomes a genesis
    /// fact that system reduces into both places, so a passage is stated once, in either of the two
    /// files, and holds both ways. It states no rule — how far from a doorway a person may pass is
    /// movement's decision, not this file's.
    #[serde(default)]
    pub passages: Vec<AuthoredPassage>,
}

/// One doorway from the place whose file states it to another place.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPassage {
    /// The place it leads to, by its authoring key.
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
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredLocation {
    /// Which place, by its authoring key.
    pub place: EntityKey,
    /// Where in it, in millimetres from the place's own origin.
    #[serde(default)]
    pub position: Option<AuthoredPosition>,
    /// Which way they face, in millidegrees clockwise. Validated and normalized by
    /// [`Orientation`](mineworld_contracts::Orientation).
    #[serde(default)]
    pub facing: Option<i32>,
}

/// A position inside a place: integer millimetres on the world's own axes.
///
/// Integers, not floats, and this is not a style choice: a position reaches the event log, and
/// floating-point arithmetic is not reproducible across platforms, so a world replayed elsewhere
/// would diverge (`AC-12`, `DD-5`).
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthoredPosition {
    /// Across the floor.
    #[serde(default)]
    pub x: Millimetres,
    /// Across the floor, at right angles to `x`.
    #[serde(default)]
    pub y: Millimetres,
    /// Height. Absent means the floor, which is what a world that does not model height means.
    #[serde(default)]
    pub z: Millimetres,
}
