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

use mineworld_contracts::{ContractError, EntityKey, SystemId};
use mineworld_kernel::KernelError;
use thiserror::Error;

/// Which part of a pack a key was declared in, so a duplicate can say where both were.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Declared {
    /// `world.yaml`'s `places` list.
    Places,
    /// `world.yaml`'s `population` list.
    Population,
    /// `world.yaml`'s `seats` list.
    Seats,
}

impl core::fmt::Display for Declared {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Places => "places",
            Self::Population => "population",
            Self::Seats => "seats",
        })
    }
}

/// What kind of content file a key names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentKind {
    /// `people/<key>.yaml`.
    Person,
    /// `places/<key>.yaml`.
    Place,
}

impl ContentKind {
    /// The directory this kind of content lives in.
    pub const fn directory(self) -> &'static str {
        match self {
            Self::Person => "people",
            Self::Place => "places",
        }
    }

    /// What one file of this kind describes, for a message to name it.
    pub const fn describes(self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Place => "place",
        }
    }
}

impl core::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Person => "person",
            Self::Place => "place",
        })
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
         (a {kind}'s key names its file)"
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
        "{path} describes a {kind} that world.yaml does not list; \
         add '{key}' to {list}, or remove the file"
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
