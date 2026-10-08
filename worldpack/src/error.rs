//! Every way a World Pack can be wrong, and nothing else.
//!
//! A pack is authored by a person, by hand, in a text editor. So the acceptance this crate is judged
//! on is not "it loads a good pack" — it is that **a bad pack is refused by name, saying what is
//! wrong and where**. A panic, a silently ignored field or a message that says only *invalid world*
//! all fail it equally.
//!
//! Three rules shape every variant below:
//!
//! 1. **It names the file.** A pack is a directory of files, and *which* file is half the answer.
//! 2. **It names the offending value, as a field rather than inside prose**, so a tool can react to
//!    it rather than parse the message.
//! 3. **Where the reader knows the legal alternatives, it lists them.** An author who typed a system
//!    name that does not exist wants to see the names that do.

use std::path::PathBuf;

use mineworld_authoring::SectionName;
use mineworld_contracts::{ContractError, EntityKey, EntityType, EventTypeId, Rejection, SystemId};
use mineworld_kernel::KernelError;
use thiserror::Error;

/// Which part of a pack a key was declared in, so a duplicate can say where both were.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declared {
    /// `world.yaml`'s `places` list.
    Places,
    /// `world.yaml`'s `population` list.
    Population,
    /// `world.yaml`'s `items` list (`ARC-36`).
    Items,
    /// `world.yaml`'s `organizations` list.
    Organizations,
    /// `world.yaml`'s `seats` list.
    Seats,
}

impl core::fmt::Display for Declared {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Places => "places",
            Self::Population => "population",
            Self::Items => "items",
            Self::Organizations => "organizations",
            Self::Seats => "seats",
        })
    }
}

/// What kind of content file a key names. Defined beside the section contract, because a System Pack
/// states which kinds of file may carry its section (`ARC-31`).
pub use mineworld_authoring::ContentKind;

/// One file of `kind` with its indefinite article, for a message: "a person", "an organization".
const fn one(kind: ContentKind) -> &'static str {
    match kind {
        ContentKind::Person => "a person",
        ContentKind::Place => "a place",
        ContentKind::Item => "an item",
        ContentKind::Organization => "an organization",
    }
}

/// Every way reading or loading a World Pack can refuse.
#[derive(Debug, Error)]
pub enum PackError {
    /// The path given is not a directory. A World Pack is a directory, not a file.
    #[error("{path} is not a World Pack directory")]
    NotAPackDirectory {
        /// The path that was offered.
        path: PathBuf,
    },

    /// A file the pack must have is absent.
    #[error("{path} does not exist, and a World Pack must have it")]
    FileMissing {
        /// The file that is not there.
        path: PathBuf,
    },

    /// A file the pack declares is absent. Distinct from [`PackError::FileMissing`] because the
    /// author can see both halves of the mistake: the list that names the key, and the file it
    /// expects.
    #[error(
        "world.yaml lists the {kind} '{key}', but {path} does not exist \
         ({one}'s key names its file)",
        one = one(*kind)
    )]
    ContentFileMissing {
        /// The key that was listed.
        key: EntityKey,
        /// Which kind of content it is.
        kind: ContentKind,
        /// The file that would have described it.
        path: PathBuf,
    },

    /// A content file exists that `world.yaml` does not list.
    ///
    /// Refused rather than ignored, and this is the refusal that catches the commonest authoring
    /// mistake there is: writing `people/carol.yaml` and forgetting to add `carol` to `population`.
    /// Ignoring the file would leave the author with a person who does not exist and no reason why.
    #[error(
        "{path} describes {one} that world.yaml does not list; \
         add '{key}' to {list}, or remove the file",
        one = one(*kind)
    )]
    ContentFileNotDeclared {
        /// The key the file's name states.
        key: String,
        /// Which kind of content the directory says it is.
        kind: ContentKind,
        /// Which list of `world.yaml` would have to name it.
        list: Declared,
        /// The file nothing refers to.
        path: PathBuf,
    },

    /// A file could not be read at all: permissions, a broken link, a directory where a file
    /// belongs.
    #[error("{path} could not be read: {source}")]
    Unreadable {
        /// The file that could not be read.
        path: PathBuf,
        /// What the operating system said.
        #[source]
        source: std::io::Error,
    },

    /// A file is not the YAML this pack format expects.
    ///
    /// The detail is the parser's own report, which carries the line, the column and an excerpt of
    /// the offending text — so an unknown field, a misspelled key, a string where a number belongs
    /// and a duplicate mapping key all arrive here already located.
    #[error("{path} is not a valid {kind} file:\n{detail}")]
    Malformed {
        /// The file that does not parse.
        path: PathBuf,
        /// What kind of file it was read as.
        kind: &'static str,
        /// The parser's report, including the position in the file.
        detail: String,
    },

    /// `world.yaml`'s `world.id` and the pack's directory name disagree.
    ///
    /// Refused for the reason the kernel refuses a persisted entity filed under another identity: a
    /// thing with two identities has none, and every later reference — the authoring provenance on
    /// every entity, a dependency naming this pack — would have to choose one.
    #[error(
        "world.yaml calls this pack '{declared}', but its directory is named '{directory}'; \
         a pack's id is its directory name"
    )]
    PackIdIsNotItsDirectory {
        /// The id `world.yaml` states.
        declared: String,
        /// The name of the directory the pack is in.
        directory: String,
    },

    /// `world.yaml`'s `mineworld:` range does not admit the framework this build is
    /// (`DECISIONS.md` `ARC-53`). Refused rather than run: the author said which frameworks the world
    /// is for, and this is not one of them.
    #[error("{path}: {refusal}")]
    FrameworkNotSupported {
        /// The `world.yaml` that states the range.
        path: PathBuf,
        /// The refusal, naming the range and the framework's version.
        refusal: mineworld_packages::PackageError,
    },

    /// `world.yaml` enables a system this build does not provide.
    ///
    /// The available names are listed, because the author's next question is which ones exist. In
    /// MVP-0 the set is fixed at compile time; when System Packs are installed rather than linked
    /// (`ARC-8`), this becomes "not installed" rather than "does not exist".
    #[error(
        "world.yaml enables the system '{system}', which this build does not provide (it has: {available})"
    )]
    UnknownSystem {
        /// The name that was asked for.
        system: SystemId,
        /// The names that exist, in the order the catalogue lists them.
        available: String,
    },

    /// `world.yaml` names the same system twice. Not merged silently: a world's system list is also
    /// its installation order, so a repeated name is an author who believes one of the two positions
    /// means something.
    #[error("world.yaml enables the system '{system}' twice")]
    SystemDeclaredTwice {
        /// The repeated name.
        system: SystemId,
    },

    /// One authoring key is declared twice in the pack.
    ///
    /// Keys are unique within a world — they are how authored content refers to entities, and the
    /// kernel refuses the second entity claiming one. Caught here so the report names the two lists
    /// rather than the internal failure.
    #[error("'{key}' is declared twice in world.yaml: once in {first}, again in {second}")]
    KeyDeclaredTwice {
        /// The repeated key.
        key: EntityKey,
        /// Where it was first declared.
        first: Declared,
        /// Where it was declared again.
        second: Declared,
    },

    /// A person is placed in a place this pack does not have.
    #[error(
        "{path} puts '{person}' in the place '{place}', which this pack does not declare \
         (it has: {known})"
    )]
    PersonInUnknownPlace {
        /// The person whose location does not resolve.
        person: EntityKey,
        /// The place they were put in.
        place: EntityKey,
        /// The places the pack does declare, in key order.
        known: String,
        /// The file that says so.
        path: PathBuf,
    },

    /// A place opens onto a place this pack does not have.
    #[error(
        "{path} gives '{place}' a passage to '{to}', which this pack does not declare \
         (it has: {known})"
    )]
    PassageToUnknownPlace {
        /// The place whose file states the passage.
        place: EntityKey,
        /// The place it was said to lead to.
        to: EntityKey,
        /// The places the pack does declare, in key order.
        known: String,
        /// The file that says so.
        path: PathBuf,
    },

    /// A place opens onto itself. A doorway joins two places; one that leads back into the room it
    /// is in is a position, not a passage.
    #[error("{path} gives '{place}' a passage to itself")]
    PassageToItself {
        /// The place.
        place: EntityKey,
        /// The file that says so.
        path: PathBuf,
    },

    /// Two places are joined twice — in both of their files, or twice in one.
    ///
    /// A passage holds both ways, so it is stated once. Two statements could disagree about where
    /// the doorway is, and a loader that picked one would be choosing for the author.
    #[error(
        "'{first}' and '{second}' are joined by more than one passage (stated again in {path}); \
         a passage holds both ways, so state it once"
    )]
    PassageStatedTwice {
        /// One of the two places, the earlier in key order.
        first: EntityKey,
        /// The other.
        second: EntityKey,
        /// The file that states it again.
        path: PathBuf,
    },

    /// A seat names something that is not a person in this pack.
    ///
    /// A seat is an existing Person a client may occupy, so a seat naming a place, or naming nobody,
    /// would be a world offering a connection into something that cannot act.
    #[error("world.yaml offers the seat '{seat}', which is not one of its people")]
    SeatIsNotOneOfThePeople {
        /// The seat that does not resolve.
        seat: EntityKey,
    },

    /// Authored content needs a capability the pack did not enable.
    ///
    /// A `location` is state the `presence` system owns, so a pack that places its people without
    /// enabling `presence` has asked for something no installed system can do. Reported as the
    /// author's mistake — a missing line in `world.yaml` — rather than as a world that quietly
    /// starts with everybody nowhere.
    #[error(
        "{path} gives '{subject}' a {content}, which the '{system}' system owns, \
         but world.yaml does not enable it"
    )]
    ContentNeedsASystem {
        /// The entity the content belongs to.
        subject: EntityKey,
        /// What the content is, in the pack format's own words.
        content: &'static str,
        /// The system that owns the state it would become.
        system: SystemId,
        /// The file that authored it.
        path: PathBuf,
    },

    /// A section that a System Pack owns appears in a kind of file that pack does not let carry it —
    /// a `routine:` in a place's file.
    #[error(
        "{path} gives the {kind} '{subject}' a '{section}' section, which only {carried_by} files \
         may carry"
    )]
    SectionNotCarriedHere {
        /// The entity the file describes.
        subject: EntityKey,
        /// The section.
        section: SectionName,
        /// The kind of file it was found in.
        kind: ContentKind,
        /// The kinds that may carry it, for the author to read.
        carried_by: String,
        /// The file.
        path: PathBuf,
    },

    /// A section names, by key, an entity the pack does not declare — or one of the wrong kind, a
    /// person where its owner needs a place.
    #[error(
        "{path} gives '{subject}' a '{section}' section naming '{key}', which is not a {expected} \
         this pack declares"
    )]
    SectionNamesUnknownEntity {
        /// The entity the file describes.
        subject: EntityKey,
        /// The section.
        section: SectionName,
        /// The key it names.
        key: EntityKey,
        /// What the section's owner needs that key to be.
        expected: EntityType,
        /// The file.
        path: PathBuf,
    },

    /// The System Pack that owns a section refused the value as the assembled world stands. The
    /// owner's own refusal, unaltered.
    #[error("{path}: the '{system}' system refused '{subject}''s '{section}' section: {reason:?}")]
    SectionRefusedByOwner {
        /// The entity the file describes.
        subject: EntityKey,
        /// The section.
        section: SectionName,
        /// Its owner.
        system: SystemId,
        /// What the owner said. Boxed: a rejection may carry a system's own detail, and every other
        /// refusal should not pay for its size.
        reason: Box<Rejection>,
        /// The file.
        path: PathBuf,
    },

    /// A section's owner seeded a fact in another System Pack's vocabulary. World genesis attributes a
    /// fact to its type's owner and checks no dependency, so this is refused here: stating another
    /// pack's fact needs a dependency on it (`ARC-26`), and a section is not a way around that.
    #[error(
        "{path}: the '{system}' system's '{section}' section stated a '{event_type}' fact, which is \
         the '{owner}' system's vocabulary"
    )]
    SectionStatedAnotherPacksFact {
        /// The section. (The entity is the file's: `path` names it.)
        section: SectionName,
        /// The section's owner.
        system: SystemId,
        /// The fact it stated.
        event_type: EventTypeId,
        /// Whose vocabulary that fact is.
        owner: SystemId,
        /// The file.
        path: PathBuf,
    },

    /// A value in the pack is not a legal contract value — a key with a capital letter, a tag with a
    /// space, an over-long name.
    ///
    /// The contract layer's own complaint, unaltered: the identifier rule lives in one place
    /// (`contracts/src/ids.rs`), and a second implementation here would eventually judge a name
    /// differently from the code that has to use it.
    #[error("{path}: {source}")]
    Value {
        /// The file that carries the value.
        path: PathBuf,
        /// What the contract layer said about it.
        #[source]
        source: ContractError,
    },

    /// The world the pack describes could not be composed.
    ///
    /// The kernel's own refusal, which is the one that matters: a system whose dependency is absent,
    /// two systems claiming one component type, a key already used. A loader that paraphrased these
    /// would be inventing a second account of what a world may be.
    #[error("the world this pack describes cannot be composed: {source}")]
    Composition {
        /// What the kernel said.
        #[source]
        source: KernelError,
    },

    // ---- Configuration (`DECISIONS.md` `ARC-61`): world.yaml's `configure:` and `configure/`. ----
    /// A `configure:` key names no system this build provides.
    #[error(
        "{path}: configure: names '{key}', which is not a system this build provides (available: \
         {available})"
    )]
    ConfigurationOfUnknownSystem {
        /// The key.
        key: String,
        /// The systems this build provides, for the author to pick from.
        available: String,
        /// `world.yaml`.
        path: PathBuf,
    },

    /// A `configure:` key names a system the world does not enable.
    #[error(
        "{path}: configure: names '{system}', which this world does not enable: add it to systems:"
    )]
    ConfigurationOwnerNotEnabled {
        /// The system.
        system: SystemId,
        /// `world.yaml`.
        path: PathBuf,
    },

    /// A `configure:` key names an enabled system that takes no configuration.
    #[error("{path}: configure: names '{system}', which takes no configuration")]
    NotConfigurable {
        /// The system.
        system: SystemId,
        /// `world.yaml`.
        path: PathBuf,
    },

    /// A `configure:` key is reserved for something a later build configures.
    #[error(
        "{path}: configure: names '{key}', which is reserved for {reserved_for}; not configurable in this build"
    )]
    ConfigurationReserved {
        /// The key.
        key: String,
        /// What it is reserved for.
        reserved_for: &'static str,
        /// `world.yaml`.
        path: PathBuf,
    },

    /// A `configure:` key is listed twice.
    #[error("{path}: configure: lists '{key}' twice")]
    ConfigurationListedTwice {
        /// The key.
        key: String,
        /// `world.yaml`.
        path: PathBuf,
    },

    /// A `configure:` key's file is absent.
    #[error("configure: names '{system}', but {path} does not exist")]
    ConfigurationFileMissing {
        /// The system.
        system: SystemId,
        /// The file that is not there.
        path: PathBuf,
    },

    /// A file in `configure/` that `configure:` does not list — the configuration an author believes
    /// they wrote and the world would never see.
    #[error(
        "{path} is not listed in world.yaml's configure: — list '{key}' there, or remove the file"
    )]
    ConfigurationFileNotDeclared {
        /// The file's stem.
        key: String,
        /// The file.
        path: PathBuf,
    },

    /// A configuration needs a system the world does not enable.
    #[error(
        "{path}: the '{by}' system's configuration needs '{requires}', which this world does not enable"
    )]
    ConfigurationRequiresSystem {
        /// The system it needs.
        requires: SystemId,
        /// The configured system.
        by: SystemId,
        /// The configuration's file.
        path: PathBuf,
    },

    /// A configuration names an entity this pack does not declare, or one of the wrong type.
    #[error(
        "{path}: the '{system}' system's configuration names '{key}', which is not a declared \
         {expected:?}"
    )]
    ConfigurationNamesUnknownEntity {
        /// The configured system.
        system: SystemId,
        /// The key it names.
        key: EntityKey,
        /// What its owner needs that key to be.
        expected: EntityType,
        /// The configuration's file.
        path: PathBuf,
    },

    /// The configured System Pack refused its configuration as the assembled world stands. The owner's
    /// own refusal, unaltered.
    #[error("{path}: the '{system}' system refused its configuration: {reason:?}")]
    ConfigurationRefusedByOwner {
        /// The configured system.
        system: SystemId,
        /// What it said. Boxed, as for a section.
        reason: Box<Rejection>,
        /// The configuration's file.
        path: PathBuf,
    },

    /// A configuration seeded a fact in another System Pack's vocabulary.
    #[error(
        "{path}: the '{system}' system's configuration stated a '{event_type}' fact, which is the \
         '{owner}' system's vocabulary"
    )]
    ConfigurationStatedAnotherPacksFact {
        /// The configured system.
        system: SystemId,
        /// The fact it stated.
        event_type: EventTypeId,
        /// Whose vocabulary that fact is.
        owner: SystemId,
        /// The configuration's file.
        path: PathBuf,
    },

    /// A configuration seeded a fact of its owner's that the owner did not declare among its
    /// configuration facts — which the drift check at resume would never compare.
    #[error(
        "{path}: the '{system}' system's configuration stated a '{event_type}' fact, which it does \
         not declare as a configuration fact"
    )]
    ConfigurationStatedUndeclaredFact {
        /// The configured system.
        system: SystemId,
        /// The fact it stated.
        event_type: EventTypeId,
        /// The configuration's file.
        path: PathBuf,
    },

    /// A save is being resumed or replayed against a World Pack whose configuration differs from the
    /// one the save was created with (`ARC-61` item 7, QPL-12).
    #[error(
        "the world's configuration differs from the save's: system '{system}' (save: {saved}; this \
         pack: {here}) — a save resumes only against the configuration it was created with"
    )]
    ConfigurationDrift {
        /// The first system whose configuration differs.
        system: SystemId,
        /// What the save holds at the first difference.
        saved: String,
        /// What this pack seeds there.
        here: String,
    },
}

impl PackError {
    /// The contract layer's complaint about a value in `path`.
    pub(crate) fn value(path: impl Into<PathBuf>, source: ContractError) -> Self {
        Self::Value {
            path: path.into(),
            source,
        }
    }
}

impl From<KernelError> for PackError {
    fn from(source: KernelError) -> Self {
        Self::Composition { source }
    }
}
