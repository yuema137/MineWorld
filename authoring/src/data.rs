//! Data a System Pack owns, decoding a section in its owner's context, and resolving across subjects
//! before seeding (`DECISIONS.md` `ARC-121`, `MODULE_SPEC.md` §3.1 "A pack that owns data").
//!
//! A section (`ARC-31`) decodes by its type alone, and a configuration (`ARC-61`) is one file about no
//! entity. Neither lets a pack declare a schema in data and check a value against it at the value's own
//! line, or see every subject at once before anything is seeded. This module adds four capabilities beside
//! them, each optional and each named by no domain:
//!
//! ```text
//! OwnsData           directories of World Packs and Entity Packs the pack owns; the loader collects their
//!                    files in one order and the owner decodes each, then builds one context from all of
//!                    them and its own configuration
//! SectionInContext   a section decoded with that context in hand (a sibling of AuthoredSection, F-S30b-1)
//! ResolvesSubjects   once, before seeding: every subject of the kinds the section is carried by — key,
//!                    kind, tags and the decoded section if any, never a name, note or location — becomes at
//!                    most one seed per subject, seeded where the section would have been
//! ShippedData        data files a System Pack compiles in for an owner's directory (Q-S30b-1)
//! ```
//!
//! The loader holds what these produce behind [`OwnerContext`] and [`ResolvedSeeds`], as it holds a section
//! behind [`AuthoredContent`]: it never learns a type, a file's meaning or a seed's.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_contracts::{EntityId, EntityKey, Rejection, SystemId, Tags};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::de::{DeserializeSeed, Deserializer, Error as _};

use crate::configuration::AuthoredConfiguration;
use crate::content::AuthoredContent;
use crate::section::{ContentKind, Reference, SectionName, Seeding, is_valid};

/// The directories a World Pack's format already gives a meaning; no pack may own one.
const FORMAT_DIRECTORIES: [&str; 6] = [
    "people",
    "places",
    "items",
    "organizations",
    "configure",
    "data",
];

/// A directory of a World Pack or Entity Pack that a System Pack owns: one word, `1..=32` bytes of `a-z`,
/// `0-9` and `-`, and none of the format's own directories.
///
/// Stated as a constant, so a name that breaks the rule does not compile — as [`SectionName`]. Unique
/// among the packs of one build: the build's own guard holds that, since only it sees every pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataDirectory(&'static str);

impl DataDirectory {
    /// The format's own directories, which no pack may own.
    pub const FORMAT: &'static [&'static str] = &FORMAT_DIRECTORIES;

    /// A directory name, checked: a constant that breaks the rule does not compile.
    ///
    /// # Panics
    ///
    /// When `name` is empty, longer than 32 bytes, holds a byte other than `a-z`, `0-9` and `-`, or is
    /// one of [`DataDirectory::FORMAT`] — at compile time, for a constant.
    pub const fn from_static(name: &'static str) -> Self {
        assert!(
            is_valid(name),
            "a data directory is 1-32 bytes of a-z, 0-9 and -"
        );
        assert!(
            !is_format(name),
            "a data directory may not be one of the format's own directories"
        );
        Self(name)
    }

    /// The directory's name, as it is written in a pack.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl core::fmt::Display for DataDirectory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.0)
    }
}

const fn same(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut at = 0;
    while at < a.len() {
        if a[at] != b[at] {
            return false;
        }
        at += 1;
    }
    true
}

const fn is_format(name: &str) -> bool {
    let mut at = 0;
    while at < FORMAT_DIRECTORIES.len() {
        if same(name, FORMAT_DIRECTORIES[at]) {
            return true;
        }
        at += 1;
    }
    false
}

/// Which kind of pack a data file came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DataSource {
    /// Compiled into a System Pack ([`ShippedData`]).
    SystemPack,
    /// A required Entity Pack's directory.
    EntityPack,
    /// The world's own directory.
    World,
}

/// Where one data file came from, for every refusal about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataOrigin {
    /// The pack it belongs to: the shipping System Pack's id, the Entity Pack's id or the world's.
    pub pack: String,
    /// Which kind of pack that is.
    pub source: DataSource,
    /// The directory it was found in.
    pub directory: DataDirectory,
    /// The file: on disk for a World or Entity Pack, crate-relative for a shipped file.
    pub path: PathBuf,
}

/// An owner's refusal of its data as a whole, in its own words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRefusal {
    /// The file it is about, when it is about one.
    pub file: Option<PathBuf>,
    /// Why, in the owner's words.
    pub reason: String,
}

/// An owner's context — every data file of the world's composition and the owner's configuration,
/// decoded and checked — with its type erased, for the loader to hold.
pub trait OwnerContext: core::fmt::Debug + Send + Sync {
    /// The System Pack it belongs to.
    fn owner(&self) -> SystemId;

    /// The owner's own [`OwnsData::Context`], for the owner (or its tools) to downcast.
    fn as_any(&self) -> &dyn Any;

    /// The context, shared: an `Arc` of the owner's own [`OwnsData::Context`] behind `Any`. What a decoder
    /// or a resolution keeps; not for the loader.
    #[doc(hidden)]
    fn shared(&self) -> Arc<dyn Any + Send + Sync>;
}

/// A pack that owns data directories in World Packs and Entity Packs (`ARC-121` item 1).
pub trait OwnsData: SystemIdentity {
    /// The directories it owns, in the order their files are handed over within one source.
    const DIRECTORIES: &'static [DataDirectory];

    /// One decoded file. Decoding is validating: a refusal keeps the file's line and column.
    type File: core::fmt::Debug + Send + Sync + 'static;

    /// What every file of the world's composition, and the owner's configuration, become.
    type Context: core::fmt::Debug + Send + Sync + 'static;

    /// Decodes one file of `directory`, straight from its YAML stream (`DEP-10`).
    ///
    /// # Errors
    ///
    /// The deserializer's own error, which keeps the position.
    fn decode_file<'de, D: Deserializer<'de>>(
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<Self::File, D::Error>;

    /// The context, once, from every file in collection order and the owner's own decoded configuration
    /// (its [`PackConfiguration::Configuration`](crate::PackConfiguration::Configuration), `None` when the
    /// world lists none). Where what spans files is checked: duplicates, namespaces.
    ///
    /// # Errors
    ///
    /// The owner's refusal, naming the file when it is about one.
    fn context(
        files: Vec<(DataOrigin, Self::File)>,
        configuration: Option<&dyn Any>,
    ) -> Result<Self::Context, DataRefusal>;
}

/// One owner's context.
struct HeldContext<S: OwnsData>(Arc<S::Context>);

impl<S: OwnsData> core::fmt::Debug for HeldContext<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(S::ID.as_str()).field(&self.0).finish()
    }
}

impl<S: OwnsData + 'static> OwnerContext for HeldContext<S> {
    fn owner(&self) -> SystemId {
        S::ID
    }

    fn as_any(&self) -> &dyn Any {
        &*self.0
    }

    fn shared(&self) -> Arc<dyn Any + Send + Sync> {
        self.0.clone()
    }
}

/// `S`'s own context, shared, if `context` is `S`'s.
fn context_of<S: OwnsData + 'static>(context: &dyn OwnerContext) -> Option<Arc<S::Context>> {
    if context.owner() != S::ID {
        return None;
    }
    context.shared().downcast::<S::Context>().ok()
}

/// `S`'s own configuration, as its owner's type behind `Any`; refused when it is another pack's.
fn configuration_of<S: SystemIdentity>(
    configuration: Option<&dyn AuthoredConfiguration>,
) -> Result<Option<&dyn Any>, String> {
    match configuration {
        None => Ok(None),
        Some(configuration) if configuration.owner() == S::ID => Ok(Some(configuration.as_any())),
        Some(configuration) => Err(format!(
            "the '{}' system was handed the '{}' system's configuration",
            S::ID.as_str(),
            configuration.owner().as_str()
        )),
    }
}

/// Decodes one of `S`'s data files and erases its type: what `owns_data!()` defines
/// `SystemPack::decode_data_file` as.
///
/// # Errors
///
/// The deserializer's error for an invalid file, or a refusal naming the system when `directory` is not
/// one of its own.
pub fn decode_data_file<'de, S: OwnsData, D: Deserializer<'de>>(
    directory: DataDirectory,
    deserializer: D,
) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
    if !S::DIRECTORIES.contains(&directory) {
        return Err(D::Error::custom(format!(
            "the '{}' system does not own the directory '{directory}'",
            S::ID.as_str()
        )));
    }
    Ok(Box::new(S::decode_file(directory, deserializer)?))
}

/// Builds `S`'s context from its erased files and configuration: what `owns_data!()` defines
/// `SystemPack::data_context` as.
///
/// # Errors
///
/// The owner's refusal; or one naming the file when a file was decoded by another owner, or the
/// configuration is another pack's — a defect of the caller, refused rather than trusted.
pub fn data_context<S: OwnsData + 'static>(
    files: Vec<(DataOrigin, Box<dyn Any + Send + Sync>)>,
    configuration: Option<&dyn AuthoredConfiguration>,
) -> Result<Arc<dyn OwnerContext>, DataRefusal> {
    let mut typed = Vec::with_capacity(files.len());
    for (origin, file) in files {
        let file = file.downcast::<S::File>().map_err(|_| DataRefusal {
            file: Some(origin.path.clone()),
            reason: format!(
                "this file was not decoded by the '{}' system",
                S::ID.as_str()
            ),
        })?;
        typed.push((origin, *file));
    }
    let configuration = configuration_of::<S>(configuration)
        .map_err(|reason| DataRefusal { file: None, reason })?;
    let context = S::context(typed, configuration)?;
    Ok(Arc::new(HeldContext::<S>(Arc::new(context))))
}

/// A section of authored content decoded in its owner's context (`ARC-121` item 2).
///
/// The sibling of [`AuthoredSection`](crate::AuthoredSection), not an extension of it: a pack implements
/// one or the other, and owns one section either way (`ARC-31`). Its seeding obligations are the same —
/// its own vocabulary, its own codec.
pub trait SectionInContext: OwnsData {
    /// The key this pack owns in a content file.
    const SECTION: SectionName;

    /// The content files that may carry it.
    const CARRIED_BY: &'static [ContentKind];

    /// The section, as authored and checked against the context.
    type Authored: core::fmt::Debug + Send + Sync + 'static;

    /// Decodes the section with the context in hand, straight from the file's stream, so a value outside
    /// what the data declares is refused at its own line and column (`DEP-10`).
    ///
    /// # Errors
    ///
    /// The deserializer's own error.
    fn decode<'de, D: Deserializer<'de>>(
        context: &Self::Context,
        deserializer: D,
    ) -> Result<Self::Authored, D::Error>;

    /// The other entities the section names by key, each with the entity type it must be.
    fn references(authored: &Self::Authored) -> Vec<Reference<'_>> {
        let _ = authored;
        Vec::new()
    }

    /// The genesis facts the section becomes, about `subject`, in this pack's own vocabulary. Not called
    /// for a pack that resolves ([`ResolvesSubjects`]): its resolution seeds instead.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] when the world as assembled cannot take the value.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        context: &Self::Context,
        authored: &Self::Authored,
    ) -> Result<Vec<Emission>, Rejection>;
}

/// One section decoded in its owner's context.
struct HeldIn<S: SectionInContext> {
    context: Arc<S::Context>,
    authored: S::Authored,
}

impl<S: SectionInContext> core::fmt::Debug for HeldIn<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(S::SECTION.as_str())
            .field(&self.authored)
            .finish()
    }
}

impl<S: SectionInContext + 'static> AuthoredContent for HeldIn<S> {
    fn section(&self) -> SectionName {
        S::SECTION
    }

    fn owner(&self) -> SystemId {
        S::ID
    }

    fn references(&self) -> Vec<Reference<'_>> {
        S::references(&self.authored)
    }

    fn seed(
        &self,
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
    ) -> Result<Vec<Emission>, Rejection> {
        S::seed(seeding, subject, &self.context, &self.authored)
    }

    fn as_any(&self) -> &dyn Any {
        &self.authored
    }
}

/// Decodes `S`'s section with its context, straight from a deserializer; produces the same
/// [`AuthoredContent`] the loader holds for any section.
pub struct DecodeIn<S: SectionInContext> {
    context: Arc<S::Context>,
}

impl<S: SectionInContext> core::fmt::Debug for DecodeIn<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DecodeIn")
            .field(&S::SECTION.as_str())
            .field(&self.context)
            .finish()
    }
}

impl<S: SectionInContext + 'static> DecodeIn<S> {
    /// A decoder with this context.
    pub const fn new(context: Arc<S::Context>) -> Self {
        Self { context }
    }

    /// A decoder with `context`, if it is `S`'s.
    pub fn from_erased(context: &dyn OwnerContext) -> Option<Self> {
        context_of::<S>(context).map(Self::new)
    }
}

impl<'de, S: SectionInContext + 'static> DeserializeSeed<'de> for DecodeIn<S> {
    type Value = Arc<dyn AuthoredContent>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let authored = S::decode(&self.context, deserializer)?;
        Ok(Arc::new(HeldIn::<S> {
            context: self.context,
            authored,
        }))
    }
}

/// One subject a resolution is handed: a file of a kind the owner's section is carried by.
///
/// Exactly these four fields, which is the privacy boundary (step-30 §8.5 item 5): no display name, no
/// `note`, no location reaches a resolution.
#[derive(Debug)]
pub struct Subject<'a, A> {
    /// The subject's authoring key.
    pub key: &'a EntityKey,
    /// Which kind of file it is.
    pub kind: ContentKind,
    /// Its tags.
    pub tags: &'a Tags,
    /// Its decoded section, if its file carries one.
    pub authored: Option<&'a A>,
}

/// A [`Subject`] as the loader hands it over, its section's type erased.
#[derive(Debug, Clone, Copy)]
pub struct ErasedSubject<'a> {
    /// The subject's authoring key.
    pub key: &'a EntityKey,
    /// Which kind of file it is.
    pub kind: ContentKind,
    /// Its tags.
    pub tags: &'a Tags,
    /// Its decoded section, if its file carries the owner's.
    pub authored: Option<&'a dyn AuthoredContent>,
}

/// A resolution's refusal, in the owner's words.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolveRefusal {
    /// The subject it is about, when it is about one.
    pub subject: Option<EntityKey>,
    /// Why.
    pub reason: String,
}

/// An owner that resolves across subjects once, before any seeding (`ARC-121` item 3).
pub trait ResolvesSubjects: SectionInContext {
    /// What one subject's resolution is.
    type Seed: core::fmt::Debug + Send + Sync + 'static;

    /// Once, after every section is decoded and every reference checked, before any seeding: every
    /// subject of the kinds the section is carried by, in the loader's file order, becomes at most one seed.
    /// A key that is not one of `subjects` is refused by the seam.
    ///
    /// # Errors
    ///
    /// The owner's refusal.
    fn resolve(
        context: &Self::Context,
        configuration: Option<&dyn Any>,
        subjects: &[Subject<'_, Self::Authored>],
    ) -> Result<BTreeMap<EntityKey, Self::Seed>, ResolveRefusal>;

    /// The genesis facts one subject's seed becomes, in this pack's own vocabulary.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] when the world as assembled cannot take the value.
    fn seed_resolved(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        context: &Self::Context,
        seed: &Self::Seed,
    ) -> Result<Vec<Emission>, Rejection>;
}

/// A resolution, with its type erased, for the loader to seed from.
pub trait ResolvedSeeds: core::fmt::Debug + Send + Sync {
    /// The System Pack it belongs to.
    fn owner(&self) -> SystemId;

    /// The owner's section, whose place in seeding the seeds take.
    fn section(&self) -> SectionName;

    /// The genesis facts of `key`'s seed, about `subject`; [`None`] when the resolution gave `key` none.
    fn seed(
        &self,
        seeding: &Seeding<'_, '_>,
        key: &EntityKey,
        subject: EntityId,
    ) -> Option<Result<Vec<Emission>, Rejection>>;

    /// The seeds themselves — the owner's `BTreeMap<EntityKey, Seed>` — for the owner's tools to downcast.
    fn as_any(&self) -> &dyn Any;
}

/// One owner's resolution.
struct HeldSeeds<S: ResolvesSubjects> {
    context: Arc<S::Context>,
    seeds: BTreeMap<EntityKey, S::Seed>,
}

impl<S: ResolvesSubjects> core::fmt::Debug for HeldSeeds<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(S::ID.as_str()).field(&self.seeds).finish()
    }
}

impl<S: ResolvesSubjects + 'static> ResolvedSeeds for HeldSeeds<S> {
    fn owner(&self) -> SystemId {
        S::ID
    }

    fn section(&self) -> SectionName {
        S::SECTION
    }

    fn seed(
        &self,
        seeding: &Seeding<'_, '_>,
        key: &EntityKey,
        subject: EntityId,
    ) -> Option<Result<Vec<Emission>, Rejection>> {
        let seed = self.seeds.get(key)?;
        Some(S::seed_resolved(seeding, subject, &self.context, seed))
    }

    fn as_any(&self) -> &dyn Any {
        &self.seeds
    }
}

/// Resolves `S` across its erased subjects: what `resolves!()` defines `SystemPack::resolve` as.
///
/// # Errors
///
/// The owner's refusal; a seed for a key that is not one of `subjects`, naming it; or a context, section or
/// configuration that is not `S`'s — a defect of the caller, refused rather than trusted.
pub fn resolve_erased<S: ResolvesSubjects + 'static>(
    context: &dyn OwnerContext,
    configuration: Option<&dyn AuthoredConfiguration>,
    subjects: &[ErasedSubject<'_>],
) -> Result<Arc<dyn ResolvedSeeds>, ResolveRefusal> {
    let refused = |subject: Option<&EntityKey>, reason: String| ResolveRefusal {
        subject: subject.cloned(),
        reason,
    };
    let context = context_of::<S>(context).ok_or_else(|| {
        refused(
            None,
            format!("the '{}' system was handed another context", S::ID.as_str()),
        )
    })?;
    let mut typed = Vec::with_capacity(subjects.len());
    for subject in subjects {
        let authored = match subject.authored {
            None => None,
            Some(content) => Some(
                (content.owner() == S::ID)
                    .then(|| content.as_any().downcast_ref::<S::Authored>())
                    .flatten()
                    .ok_or_else(|| {
                        refused(
                            Some(subject.key),
                            format!(
                                "this section was not decoded by the '{}' system",
                                S::ID.as_str()
                            ),
                        )
                    })?,
            ),
        };
        typed.push(Subject {
            key: subject.key,
            kind: subject.kind,
            tags: subject.tags,
            authored,
        });
    }
    let configuration = configuration_of::<S>(configuration).map_err(|why| refused(None, why))?;
    let seeds = S::resolve(&context, configuration, &typed)?;
    let given: BTreeSet<&EntityKey> = subjects.iter().map(|subject| subject.key).collect();
    if let Some(stranger) = seeds.keys().find(|key| !given.contains(key)) {
        return Err(refused(
            Some(stranger),
            format!(
                "'{stranger}' is not one of the subjects the '{}' system was given",
                S::ID.as_str()
            ),
        ));
    }
    Ok(Arc::new(HeldSeeds::<S> { context, seeds }))
}

/// A data file a System Pack ships, compiled in, for an owner's directory (`ARC-121` item 4,
/// Q-S30b-1): its crate-relative path, `<directory>/<file>.yaml`, and its text.
///
/// Declared through `mineworld_sdk::ships_data!`, which reads the file with `include_str!`; the path is
/// checked when the pack compiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShippedData {
    path: &'static str,
    directory: DataDirectory,
    text: &'static str,
}

impl ShippedData {
    /// A shipped file at `path` with `text`.
    ///
    /// # Panics
    ///
    /// When `path` is not exactly `<directory>/<file>.yaml` with a valid [`DataDirectory`] and a
    /// non-empty file name — at compile time, for a constant.
    pub const fn new(path: &'static str, text: &'static str) -> Self {
        let bytes = path.as_bytes();
        let mut slash = 0;
        while slash < bytes.len() && bytes[slash] != b'/' {
            slash += 1;
        }
        assert!(
            slash < bytes.len(),
            "a shipped file is <directory>/<file>.yaml"
        );
        let (directory, file) = path.split_at(slash);
        let file = file.as_bytes();
        let mut at = 1;
        while at < file.len() {
            assert!(
                file[at] != b'/',
                "a shipped file lies directly in its directory"
            );
            at += 1;
        }
        let suffix = b".yaml";
        assert!(
            file.len() > suffix.len() + 1,
            "a shipped file is <directory>/<file>.yaml"
        );
        let mut at = 0;
        while at < suffix.len() {
            assert!(
                file[file.len() - suffix.len() + at] == suffix[at],
                "a shipped file's name ends in .yaml"
            );
            at += 1;
        }
        Self {
            path,
            directory: DataDirectory::from_static(directory),
            text,
        }
    }

    /// The crate-relative path, as declared.
    pub const fn path(&self) -> &'static str {
        self.path
    }

    /// The owner's directory it is shipped for.
    pub const fn directory(&self) -> DataDirectory {
        self.directory
    }

    /// The file's name within that directory.
    pub fn file(&self) -> &'static str {
        &self.path[self.directory.as_str().len() + 1..]
    }

    /// The file's text.
    pub const fn text(&self) -> &'static str {
        self.text
    }
}

#[cfg(test)]
mod tests;
