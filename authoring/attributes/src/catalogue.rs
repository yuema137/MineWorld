//! The catalogue (`DECISIONS.md` `ARC-122`; `MODULE_SPEC.md` §4.1): every declaration of a world's
//! composition, built once from the files in collection order, then frozen.
//!
//! ```text
//! 1  namespaces   a file's namespace is its pack's id; `core` is person-attributes' and nobody else's
//! 2  vocabularies collected first, as <namespace>.<name>
//! 3  declarations in the file's namespace (ForeignNamespace), each id once (DuplicateAttribute), each
//!                 owner `attributes` or the shipping System Pack itself (F-S30c-7)
//! 4  derived      sources and thresholds checked against every declaration
//! 5  overrides    in collection order, by MODULE_SPEC §4.1's table (OverrideNotAllowed)
//! 6  uses         what each shipping System Pack reads and proposes, checked last (S30-c C5)
//! ```
//!
//! Catalogue order is qualified-id order. Every refusal is one [`AttributeError`] named as §4.1's refusal
//! table names it, with the file and, where it has one, the line.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mineworld_authoring::{DataOrigin, DataSource};
use mineworld_contracts::SystemId;
use serde::{Deserialize, Serialize};
use serde_saphyr::Spanned;

use crate::ATTRIBUTES;
use crate::declaration::{
    AttributesFile, DataFile, Declaration, DeclarationFault, Derive, Mutability, RawDeclaration,
    RawRead, RawUses, Read, Threshold, Visibility, band_source, thresholds,
};
use crate::id::{QualifiedId, Slug, ValueSlug, is_namespace, nearest};
use crate::kind::{Kind, KindName};
use crate::value::{Loose, ValueError, decode};

/// The pack whose namespace is `core` (S30-e ships it).
pub const CORE_PACK: &str = "person-attributes";

/// The reserved namespace of the built-in catalogue.
pub const CORE: &str = "core";

/// A place in a file, for a refusal: the file, and the line and column where the YAML parser knew them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct At {
    /// The file.
    pub file: PathBuf,
    /// 1-indexed line and column; `None` where the source carried no position.
    pub position: Option<(u64, u64)>,
}

impl At {
    fn file(file: &Path) -> Self {
        Self {
            file: file.to_path_buf(),
            position: None,
        }
    }

    fn spanned<T>(file: &Path, spanned: &Spanned<T>) -> Self {
        let (line, column) = (spanned.referenced.line(), spanned.referenced.column());
        Self {
            file: file.to_path_buf(),
            position: (line > 0).then_some((line, column)),
        }
    }
}

impl fmt::Display for At {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = self.file.display().to_string().replace('\\', "/");
        match self.position {
            Some((line, column)) => write!(f, "{file} line {line} column {column}"),
            None => f.write_str(&file),
        }
    }
}

/// Every refusal of the attribute model, named as `MODULE_SPEC.md` §4.1's table names it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AttributeError {
    /// A declaration breaks a field rule.
    #[error("{at}: BadDeclaration: '{attribute}' field '{field}': {reason}")]
    BadDeclaration {
        /// Where.
        at: At,
        /// The attribute (or vocabulary, or `uses`).
        attribute: String,
        /// The field.
        field: String,
        /// The rule.
        reason: String,
    },
    /// An id no catalogue declares.
    #[error("{}UnknownAttribute: '{id}' is declared by no catalogue of this world (nearest: {})", .at.as_ref().map(|at| format!("{at}: ")).unwrap_or_default(), .nearest.join(", "))]
    UnknownAttribute {
        /// Where it was named, when a file named it.
        at: Option<At>,
        /// The id as written.
        id: String,
        /// The nearest declared ids.
        nearest: Vec<String>,
    },
    /// Two declarations of one id.
    #[error("DuplicateAttribute: '{id}' is declared in {} and again in {}", .first.display(), .second.display())]
    DuplicateAttribute {
        /// The id.
        id: QualifiedId,
        /// The first file.
        first: PathBuf,
        /// The second file.
        second: PathBuf,
    },
    /// A declaration in another pack's namespace.
    #[error(
        "{at}: ForeignNamespace: '{id}' is not in the namespace of the pack '{pack}' that declares it"
    )]
    ForeignNamespace {
        /// Where.
        at: At,
        /// The id as written.
        id: String,
        /// The declaring pack.
        pack: String,
    },
    /// A short slug two catalogues declare.
    #[error("AmbiguousSlug: '{slug}' is declared as {}; write the qualified id", .candidates.iter().map(ToString::to_string).collect::<Vec<_>>().join(" and "))]
    AmbiguousSlug {
        /// The slug.
        slug: String,
        /// The qualified candidates.
        candidates: Vec<QualifiedId>,
    },
    /// A value that does not fit its declaration.
    #[error("{}'{attribute}': {error}", .at.as_ref().map(|at| format!("{at}: ")).unwrap_or_default())]
    Value {
        /// Where, when a file wrote it.
        at: Option<At>,
        /// The attribute.
        attribute: QualifiedId,
        /// What is wrong with it.
        error: ValueError,
    },
    /// A vocabulary no pack declares.
    #[error("{at}: UnknownTopic: '{attribute}' names the vocabulary '{vocabulary}', which no pack declares (nearest: {})", .nearest.join(", "))]
    UnknownVocabulary {
        /// Where.
        at: At,
        /// The attribute.
        attribute: QualifiedId,
        /// The vocabulary.
        vocabulary: QualifiedId,
        /// The nearest declared vocabularies.
        nearest: Vec<String>,
    },
    /// An override of a field the table forbids.
    #[error("{at}: OverrideNotAllowed: '{attribute}' field '{field}' may not be overridden{}", .why.as_ref().map(|why| format!(" ({why})")).unwrap_or_default())]
    OverrideNotAllowed {
        /// Where.
        at: At,
        /// The attribute.
        attribute: QualifiedId,
        /// The field.
        field: String,
        /// Why, when more than the table.
        why: Option<String>,
    },
    /// An attribute whose owner the world does not enable.
    #[error(
        "{at}: OwnerNotInstalled: '{attribute}' is owned by '{owner}', which no enabled System Pack of this world is"
    )]
    OwnerNotInstalled {
        /// Where.
        at: At,
        /// The attribute.
        attribute: QualifiedId,
        /// The owner it names.
        owner: String,
    },
    /// A read of another kind than the declaration's.
    #[error("WrongKind: the '{pack}' system reads '{attribute}' as {requested}, but it is declared {declared}{}", if *.requested == KindName::Text { "; no System reads prose" } else { "" })]
    WrongRead {
        /// The reading pack.
        pack: SystemId,
        /// The attribute.
        attribute: QualifiedId,
        /// Its declared kind.
        declared: KindName,
        /// The kind the pack reads.
        requested: KindName,
    },
    /// A read of an attribute the catalogue does not hold.
    #[error(
        "Missing: the '{pack}' system reads '{attribute}', which this world's catalogue does not hold"
    )]
    Missing {
        /// The reading pack.
        pack: SystemId,
        /// The attribute as written.
        attribute: String,
    },
}

impl AttributeError {
    /// The file the refusal is about, when it is about one.
    pub fn file(&self) -> Option<&Path> {
        match self {
            Self::BadDeclaration { at, .. }
            | Self::ForeignNamespace { at, .. }
            | Self::UnknownVocabulary { at, .. }
            | Self::OverrideNotAllowed { at, .. }
            | Self::OwnerNotInstalled { at, .. } => Some(&at.file),
            Self::UnknownAttribute { at, .. } | Self::Value { at, .. } => {
                at.as_ref().map(|at| at.file.as_path())
            }
            Self::DuplicateAttribute { second, .. } => Some(second),
            Self::AmbiguousSlug { .. } | Self::WrongRead { .. } | Self::Missing { .. } => None,
        }
    }
}

/// What one System Pack declares it reads and proposes (`MODULE_SPEC.md` §3.1, F-S30c-2).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uses {
    /// Each attribute read, and how.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub reads: BTreeMap<QualifiedId, Read>,
    /// Each attribute it may propose changes to.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub proposes: BTreeSet<QualifiedId>,
}

/// One world's frozen catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalogue {
    attributes: BTreeMap<QualifiedId, Declaration>,
    origins: BTreeMap<QualifiedId, PathBuf>,
    vocabularies: BTreeMap<QualifiedId, Vec<ValueSlug>>,
    short: BTreeMap<String, Vec<QualifiedId>>,
    uses: BTreeMap<SystemId, Uses>,
}

/// The catalogue as `attributes-composed` states it: what the running world needs, and no file path,
/// so moving a file is not drift. Canonical: every map sorted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    /// Every declaration, in catalogue order.
    pub attributes: Vec<Declaration>,
    /// Every vocabulary.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vocabularies: BTreeMap<QualifiedId, Vec<ValueSlug>>,
    /// What each reading or proposing pack declared.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub uses: BTreeMap<SystemId, Uses>,
}

/// The namespace a file declares in: its pack's id, `core` for person-attributes.
fn namespace_of(origin: &DataOrigin, at: &At) -> Result<String, AttributeError> {
    if origin.pack == CORE_PACK {
        return Ok(CORE.to_owned());
    }
    if origin.pack == CORE || !is_namespace(&origin.pack) {
        return Err(AttributeError::ForeignNamespace {
            at: at.clone(),
            id: origin.pack.clone(),
            pack: origin.pack.clone(),
        });
    }
    Ok(origin.pack.clone())
}

fn bad(
    at: At,
    attribute: impl fmt::Display,
    field: &str,
    reason: impl Into<String>,
) -> AttributeError {
    AttributeError::BadDeclaration {
        at,
        attribute: attribute.to_string(),
        field: field.to_owned(),
        reason: reason.into(),
    }
}

/// One declaration waiting for the derived check: its raw form, file and position.
struct Pending<'f> {
    raw: &'f Spanned<RawDeclaration>,
    file: &'f Path,
}

impl Catalogue {
    /// The catalogue of a composition, from every file in collection order (shipped, Entity Packs by id,
    /// the world; S30-b I-b-7).
    ///
    /// # Errors
    ///
    /// The first refusal, in the order of the module's table.
    pub fn build(files: &[(DataOrigin, DataFile)]) -> Result<Self, AttributeError> {
        let mut namespaces = BTreeSet::from([CORE.to_owned()]);
        for (origin, _) in files {
            if let Ok(namespace) = namespace_of(origin, &At::file(&origin.path)) {
                namespaces.insert(namespace);
            }
        }
        let vocabularies = vocabularies(files)?;
        let shippers: BTreeSet<&str> = files
            .iter()
            .filter(|(origin, _)| origin.source == DataSource::SystemPack)
            .map(|(origin, _)| origin.pack.as_str())
            .collect();

        let mut catalogue = Self {
            attributes: BTreeMap::new(),
            origins: BTreeMap::new(),
            vocabularies,
            short: BTreeMap::new(),
            uses: BTreeMap::new(),
        };
        let mut pending = BTreeMap::new();
        for (origin, file) in files {
            let DataFile::Attributes(file) = file else {
                continue;
            };
            let namespace = namespace_of(origin, &At::file(&origin.path))?;
            for (slug, raw) in &file.attributes {
                let at = At::spanned(&origin.path, raw);
                let id = qualified(&namespace, slug, &namespaces, origin, &at)?;
                if let Some(first) = catalogue.origins.get(&id) {
                    return Err(AttributeError::DuplicateAttribute {
                        id,
                        first: first.clone(),
                        second: origin.path.clone(),
                    });
                }
                let owner = owner(&raw.value, &id, origin, &shippers, &at)?;
                if raw.value.since().is_some_and(str::is_empty) {
                    return Err(bad(at, &id, "since", "a version, when stated"));
                }
                let declaration = raw
                    .value
                    .resolve(id.clone(), owner, &|vocabulary| {
                        catalogue.vocabulary(vocabulary).cloned()
                    })
                    .map_err(|fault| refusal(fault, &at, &id))?;
                catalogue.origins.insert(id.clone(), origin.path.clone());
                catalogue.attributes.insert(id.clone(), declaration);
                pending.insert(
                    id,
                    Pending {
                        raw,
                        file: &origin.path,
                    },
                );
            }
        }
        catalogue.derived(&pending)?;
        for (origin, file) in files {
            if let DataFile::Attributes(file) = file {
                catalogue.overrides(origin, file)?;
            }
        }
        for (origin, file) in files {
            if let DataFile::Attributes(file) = file {
                catalogue.collect_uses(origin, file)?;
            }
        }
        catalogue.short = short_slugs(&catalogue.attributes);
        catalogue.check_uses()?;
        Ok(catalogue)
    }

    /// The vocabulary `id`'s values, or the refusal naming the nearest.
    fn vocabulary(&self, id: &QualifiedId) -> Result<&Vec<ValueSlug>, DeclarationFault> {
        self.vocabularies
            .get(id)
            .ok_or_else(|| DeclarationFault::UnknownVocabulary {
                vocabulary: id.clone(),
                nearest: nearest(
                    id.as_str(),
                    self.vocabularies.keys().map(QualifiedId::as_str),
                ),
            })
    }

    /// Every derived declaration's source and thresholds, against the whole catalogue.
    fn derived(
        &mut self,
        pending: &BTreeMap<QualifiedId, Pending<'_>>,
    ) -> Result<(), AttributeError> {
        for (id, entry) in pending {
            let at = At::spanned(entry.file, entry.raw);
            let declaration = &self.attributes[id];
            let Some(derive) = declaration.derive.clone() else {
                continue;
            };
            let source_kind = |from: &QualifiedId| {
                self.attributes
                    .get(from)
                    .map(|source| source.kind.clone())
                    .ok_or_else(|| {
                        bad(
                            at.clone(),
                            id,
                            "derive.from",
                            format!("'{from}' is declared by no catalogue"),
                        )
                    })
            };
            match derive {
                Derive::Age { from } => {
                    if source_kind(&from)? != Kind::Instant {
                        return Err(bad(
                            at,
                            id,
                            "derive.from",
                            format!("an age is derived from an instant, and '{from}' is not one"),
                        ));
                    }
                    if !matches!(declaration.kind, Kind::Quantity(quantity) if quantity.scale == 0)
                    {
                        return Err(bad(at, id, "kind", "an age is a quantity at scale 0"));
                    }
                }
                Derive::Band { from, .. } => {
                    let source = source_kind(&from)?;
                    let below = band_source(&entry.raw.value).unwrap_or_default();
                    let thresholds = thresholds(&source, below, "derive.below")
                        .map_err(|fault| refusal(fault, &at, id))?;
                    let Kind::Level { values } = &declaration.kind else {
                        return Err(bad(at, id, "kind", "a band is a level"));
                    };
                    if values.len() != thresholds.len() + 1 {
                        return Err(bad(
                            at,
                            id,
                            "derive.below",
                            format!(
                                "{} values need {} thresholds",
                                values.len(),
                                values.len() - 1
                            ),
                        ));
                    }
                    if let Some(declaration) = self.attributes.get_mut(id) {
                        declaration.derive = Some(Derive::Band {
                            from,
                            below: thresholds,
                        });
                    }
                }
                Derive::Projection => {
                    if declaration.owner == ATTRIBUTES {
                        return Err(bad(
                            at,
                            id,
                            "derive",
                            "a projection is an owner's; `attributes` projects nothing",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// One file's overrides, by `MODULE_SPEC.md` §4.1's table.
    fn overrides(
        &mut self,
        origin: &DataOrigin,
        file: &AttributesFile,
    ) -> Result<(), AttributeError> {
        for (written, entry) in &file.overrides {
            let at = At::spanned(&origin.path, entry);
            let id = self.resolve_id(written).map_err(|error| match error {
                AttributeError::UnknownAttribute { id, nearest, .. } => {
                    AttributeError::UnknownAttribute {
                        at: Some(at.clone()),
                        id,
                        nearest,
                    }
                }
                other => other,
            })?;
            if self.origins.get(&id) == Some(&origin.path) {
                return Err(bad(
                    at,
                    &id,
                    "overrides",
                    "a file does not override its own declaration",
                ));
            }
            let mut declaration = self.attributes[&id].clone();
            for (field, value) in &entry.value {
                apply(&mut declaration, field, value).map_err(|why| match why {
                    Overridden::NotAllowed(why) => AttributeError::OverrideNotAllowed {
                        at: at.clone(),
                        attribute: id.clone(),
                        field: field.clone(),
                        why,
                    },
                    Overridden::Bad(reason) => bad(at.clone(), &id, field, reason),
                    Overridden::Value(error) => AttributeError::Value {
                        at: Some(at.clone()),
                        attribute: id.clone(),
                        error,
                    },
                })?;
            }
            self.attributes.insert(id, declaration);
        }
        Ok(())
    }

    /// One file's `uses:`: only in a file a System Pack ships.
    fn collect_uses(
        &mut self,
        origin: &DataOrigin,
        file: &AttributesFile,
    ) -> Result<(), AttributeError> {
        let Some(uses) = &file.uses else {
            return Ok(());
        };
        let at = At::spanned(&origin.path, uses);
        if origin.source != DataSource::SystemPack {
            return Err(bad(
                at,
                "uses",
                "uses",
                "only a System Pack declares what it reads and proposes",
            ));
        }
        let pack = SystemId::new(&origin.pack)
            .map_err(|error| bad(at.clone(), "uses", "uses", error.to_string()))?;
        let RawUses { reads, proposes } = &uses.value;
        let entry = self.uses.entry(pack).or_default();
        for (id, read) in reads {
            let read = match read {
                RawRead::Kind(kind) => Read {
                    kind: *kind,
                    optional: false,
                },
                RawRead::Full(read) => read.clone(),
            };
            let id =
                QualifiedId::new(id).map_err(|reason| bad(at.clone(), "uses", "reads", reason))?;
            entry.reads.insert(id, read);
        }
        for id in proposes {
            let id = QualifiedId::new(id)
                .map_err(|reason| bad(at.clone(), "uses", "proposes", reason))?;
            entry.proposes.insert(id);
        }
        Ok(())
    }

    /// Every pack's `uses:` against the frozen catalogue (F-S30c-2): each read held and of its kind,
    /// never `text`; each proposal an `evolving` attribute `attributes` owns.
    fn check_uses(&self) -> Result<(), AttributeError> {
        for (pack, uses) in &self.uses {
            for (id, read) in &uses.reads {
                let Some(declaration) = self.attributes.get(id) else {
                    return Err(AttributeError::Missing {
                        pack: pack.clone(),
                        attribute: id.to_string(),
                    });
                };
                let declared = declaration.kind.name();
                if read.kind == KindName::Text || read.kind != declared {
                    return Err(AttributeError::WrongRead {
                        pack: pack.clone(),
                        attribute: id.clone(),
                        declared,
                        requested: read.kind,
                    });
                }
            }
            for id in &uses.proposes {
                let Some(declaration) = self.attributes.get(id) else {
                    return Err(AttributeError::Missing {
                        pack: pack.clone(),
                        attribute: id.to_string(),
                    });
                };
                if declaration.owner != ATTRIBUTES || declaration.mutability != Mutability::Evolving
                {
                    return Err(bad(
                        At::file(&self.origins[id]),
                        id,
                        "proposes",
                        format!(
                            "the '{pack}' system may propose only an evolving attribute `attributes` owns"
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Every declaration, in catalogue order.
    pub fn attributes(&self) -> impl Iterator<Item = &Declaration> {
        self.attributes.values()
    }

    /// How many declarations it holds.
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// The declaration of a qualified id.
    pub fn get(&self, id: &QualifiedId) -> Option<&Declaration> {
        self.attributes.get(id)
    }

    /// The file a declaration came from (not in a running catalogue).
    pub fn origin(&self, id: &QualifiedId) -> Option<&Path> {
        self.origins.get(id).map(PathBuf::as_path)
    }

    /// Every vocabulary.
    pub const fn vocabularies(&self) -> &BTreeMap<QualifiedId, Vec<ValueSlug>> {
        &self.vocabularies
    }

    /// What each pack declared it reads and proposes.
    pub const fn uses(&self) -> &BTreeMap<SystemId, Uses> {
        &self.uses
    }

    /// The qualified id `written` names: itself when qualified and declared, else a unique short slug.
    ///
    /// # Errors
    ///
    /// `UnknownAttribute` with the nearest declared ids, or `AmbiguousSlug` with the candidates.
    pub fn resolve_id(&self, written: &str) -> Result<QualifiedId, AttributeError> {
        if let Ok(id) = QualifiedId::new(written)
            && self.attributes.contains_key(&id)
        {
            return Ok(id);
        }
        match self.short.get(written).map(Vec::as_slice) {
            Some([one]) => Ok(one.clone()),
            Some(candidates) if candidates.len() > 1 => Err(AttributeError::AmbiguousSlug {
                slug: written.to_owned(),
                candidates: candidates.to_vec(),
            }),
            _ => Err(AttributeError::UnknownAttribute {
                at: None,
                id: written.to_owned(),
                nearest: nearest(written, self.attributes.keys().map(QualifiedId::as_str)),
            }),
        }
    }

    /// The declaration `written` names (qualified, or a unique short slug).
    ///
    /// # Errors
    ///
    /// As [`Catalogue::resolve_id`].
    pub fn resolve(&self, written: &str) -> Result<&Declaration, AttributeError> {
        let id = self.resolve_id(written)?;
        Ok(&self.attributes[&id])
    }

    /// The catalogue as `attributes-composed` states it.
    pub fn projection(&self) -> Projection {
        Projection {
            attributes: self.attributes.values().cloned().collect(),
            vocabularies: self.vocabularies.clone(),
            uses: self.uses.clone(),
        }
    }

    /// The running catalogue a projection describes: no file paths.
    pub fn from_projection(projection: Projection) -> Self {
        let attributes: BTreeMap<QualifiedId, Declaration> = projection
            .attributes
            .into_iter()
            .map(|declaration| (declaration.id.clone(), declaration))
            .collect();
        Self {
            short: short_slugs(&attributes),
            attributes,
            origins: BTreeMap::new(),
            vocabularies: projection.vocabularies,
            uses: projection.uses,
        }
    }
}

/// Every short slug, with the qualified ids that have it.
fn short_slugs(
    attributes: &BTreeMap<QualifiedId, Declaration>,
) -> BTreeMap<String, Vec<QualifiedId>> {
    let mut short: BTreeMap<String, Vec<QualifiedId>> = BTreeMap::new();
    for id in attributes.keys() {
        short
            .entry(id.slug().to_owned())
            .or_default()
            .push(id.clone());
    }
    short
}

/// Every vocabulary of every `vocabularies/` file, as `<namespace>.<name>`.
fn vocabularies(
    files: &[(DataOrigin, DataFile)],
) -> Result<BTreeMap<QualifiedId, Vec<ValueSlug>>, AttributeError> {
    let mut vocabularies = BTreeMap::new();
    for (origin, file) in files {
        let DataFile::Vocabularies(file) = file else {
            continue;
        };
        let at = At::file(&origin.path);
        let namespace = namespace_of(origin, &at)?;
        for (name, values) in &file.vocabularies {
            let id = QualifiedId::new(&format!("{namespace}.{name}"))
                .map_err(|reason| bad(at.clone(), name, "vocabularies", reason))?;
            if vocabularies.insert(id.clone(), values.0.clone()).is_some() {
                return Err(bad(
                    at,
                    &id,
                    "vocabularies",
                    "this vocabulary is declared twice",
                ));
            }
        }
    }
    Ok(vocabularies)
}

/// The qualified id of `slug` in `namespace`: a slug whose first segment is another known namespace is
/// `ForeignNamespace`; one whose first segment is the file's own namespace is taken as qualified.
fn qualified(
    namespace: &str,
    slug: &Slug,
    namespaces: &BTreeSet<String>,
    origin: &DataOrigin,
    at: &At,
) -> Result<QualifiedId, AttributeError> {
    let written = slug.as_str();
    if let Some((first, rest)) = written.split_once('.') {
        if first == namespace {
            return QualifiedId::new(&format!("{first}.{rest}"))
                .map_err(|reason| bad(at.clone(), written, "the key", reason));
        }
        if namespaces.contains(first) {
            return Err(AttributeError::ForeignNamespace {
                at: at.clone(),
                id: written.to_owned(),
                pack: origin.pack.clone(),
            });
        }
    }
    QualifiedId::new(&format!("{namespace}.{written}"))
        .map_err(|reason| bad(at.clone(), written, "the key", reason))
}

/// The owner a declaration names (F-S30c-7): `attributes` by default; a System Pack may name itself; a
/// World or Entity Pack naming another owner is `BadDeclaration` when that pack ships declarations here,
/// else `OwnerNotInstalled`.
fn owner(
    raw: &RawDeclaration,
    id: &QualifiedId,
    origin: &DataOrigin,
    shippers: &BTreeSet<&str>,
    at: &At,
) -> Result<SystemId, AttributeError> {
    let Some(named) = raw.owner() else {
        return Ok(ATTRIBUTES);
    };
    if named == ATTRIBUTES.as_str() {
        return Ok(ATTRIBUTES);
    }
    if origin.source == DataSource::SystemPack && named == origin.pack {
        return SystemId::new(named)
            .map_err(|error| bad(at.clone(), id, "owner", error.to_string()));
    }
    if shippers.contains(named) {
        return Err(bad(
            at.clone(),
            id,
            "owner",
            format!("only '{named}' declares the attributes it owns"),
        ));
    }
    Err(AttributeError::OwnerNotInstalled {
        at: at.clone(),
        attribute: id.clone(),
        owner: named.to_owned(),
    })
}

/// A declaration fault as the catalogue's refusal.
fn refusal(fault: DeclarationFault, at: &At, id: &QualifiedId) -> AttributeError {
    match fault {
        DeclarationFault::Field { field, reason } => bad(at.clone(), id, field, reason),
        DeclarationFault::UnknownVocabulary {
            vocabulary,
            nearest,
        } => AttributeError::UnknownVocabulary {
            at: at.clone(),
            attribute: id.clone(),
            vocabulary,
            nearest,
        },
        DeclarationFault::Default(error) => AttributeError::Value {
            at: Some(at.clone()),
            attribute: id.clone(),
            error,
        },
    }
}

/// Why an override's field was refused.
enum Overridden {
    NotAllowed(Option<String>),
    Bad(String),
    Value(ValueError),
}

/// The fields a declaration has but an override may not change.
const FIXED_FIELDS: [&str; 8] = [
    "kind",
    "owner",
    "mutability",
    "group",
    "derive",
    "since",
    "prompt_rank",
    "vocabulary",
];

fn text(value: &Loose) -> Result<String, Overridden> {
    match value {
        Loose::Text(text) => Ok(text.clone()),
        other => Err(Overridden::Bad(format!("{} is not text", other.describe()))),
    }
}

/// One overridden field, by the table: presentation, added facets, a default, a narrowed visibility.
fn apply(declaration: &mut Declaration, field: &str, value: &Loose) -> Result<(), Overridden> {
    match field {
        "label" => declaration.label = text(value)?,
        "phrase" => {
            let phrase = text(value)?;
            if !crate::declaration::is_phrase(&phrase) {
                return Err(Overridden::Bad(
                    "a phrase names exactly one {value}".to_owned(),
                ));
            }
            declaration.phrase = phrase;
        }
        "words" => {
            let Loose::Map(words) = value else {
                return Err(Overridden::Bad("words map values to words".to_owned()));
            };
            let values = declaration.kind.values().unwrap_or_default().to_vec();
            for (slug, words) in words {
                let slug = values
                    .iter()
                    .find(|value| value.as_str() == slug)
                    .ok_or_else(|| {
                        Overridden::Bad(format!(
                            "'{slug}' is not a value of {}",
                            declaration.kind.domain()
                        ))
                    })?;
                declaration.words.insert(slug.clone(), text(words)?);
            }
        }
        "facets" => {
            let Loose::List(facets) = value else {
                return Err(Overridden::Bad("facets is a list".to_owned()));
            };
            for facet in facets {
                declaration.facets.insert(text(facet)?);
            }
        }
        "default" => {
            if declaration.mutability == Mutability::Derived {
                return Err(Overridden::Bad("nothing states a derived value".to_owned()));
            }
            declaration.default =
                Some(decode(&declaration.kind, value).map_err(Overridden::Value)?);
        }
        "visibility" => {
            let narrowed: Visibility = match value {
                Loose::Text(text) => {
                    serde_json::from_value(serde_json::Value::String(text.clone()))
                        .map_err(|_| Overridden::Bad(format!("'{text}' is not a visibility")))?
                }
                other => {
                    return Err(Overridden::Bad(format!(
                        "{} is not a visibility",
                        other.describe()
                    )));
                }
            };
            if narrowed < declaration.visibility {
                return Err(Overridden::NotAllowed(Some(format!(
                    "{} would widen {}",
                    narrowed, declaration.visibility
                ))));
            }
            declaration.visibility = narrowed;
        }
        "perceived_as" => perceived_words(declaration, value)?,
        fixed if FIXED_FIELDS.contains(&fixed) => return Err(Overridden::NotAllowed(None)),
        other => {
            return Err(Overridden::Bad(format!(
                "'{other}' is not a field of a declaration"
            )));
        }
    }
    Ok(())
}

/// `perceived_as: { bands: [{ words }, …] }`: the words of each band, in order; never a threshold.
fn perceived_words(declaration: &mut Declaration, value: &Loose) -> Result<(), Overridden> {
    let Some(bands) = declaration.perceived_as.as_mut() else {
        return Err(Overridden::Bad(
            "the attribute declares no perceived_as".to_owned(),
        ));
    };
    let Loose::Map(map) = value else {
        return Err(Overridden::Bad(
            "perceived_as is { bands: [...] }".to_owned(),
        ));
    };
    let Some(Loose::List(written)) = map.get("bands") else {
        return Err(Overridden::Bad(
            "perceived_as is { bands: [...] }".to_owned(),
        ));
    };
    if written.len() != bands.len() {
        return Err(Overridden::NotAllowed(Some(
            "the number of bands".to_owned(),
        )));
    }
    for (band, written) in bands.iter_mut().zip(written) {
        let Loose::Map(written) = written else {
            return Err(Overridden::Bad("a band is { words }".to_owned()));
        };
        if written.contains_key("below") {
            return Err(Overridden::NotAllowed(Some(
                "a band's threshold".to_owned(),
            )));
        }
        band.words = text(
            written
                .get("words")
                .ok_or_else(|| Overridden::Bad("a band states its words".to_owned()))?,
        )?;
    }
    Ok(())
}

/// The thresholds a derived band's source is compared with, for tests that read them.
pub fn band_thresholds(declaration: &Declaration) -> Option<&[Threshold]> {
    match &declaration.derive {
        Some(Derive::Band { below, .. }) => Some(below),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
