//! Label packs (`DECISIONS.md` `ARC-129`; `MODULE_SPEC.md` §4.1 "Rendering"; `pr-s30k-localisation.md`
//! §§5.2, 5.3): `labels/<locale>.yaml`, the words an attribute renders with in a language other than
//! English. English is the declaration itself; a label pack never changes it.
//!
//! ```yaml
//! format: 1
//! locale: <locale>                  the file name's stem, canonical; never en
//! words:  { list, and, join, more, pair, yes, no }      all seven, or none in this file
//! labels:
//!   <qualified id>: { label?, phrase?, flag?, words?: { <value slug>: <words> }, perceived_as?: [<words>] }
//! ```
//!
//! Every file of one composition is merged per field in collection order: the world's files win over
//! every pack's; two packs giving one field is `DuplicateLabel`. The merged [`Labels`] are presentation
//! data: they are never stated as a fact, so no label file moves a digest (I-k-1).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_authoring::{DataOrigin, DataSource, Locale};
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use serde_saphyr::Spanned;

use crate::catalogue::{At, Catalogue};
use crate::declaration::{Declaration, FormatOne, is_phrase};
use crate::id::{QualifiedId, ValueSlug, nearest};
use crate::kind::Kind;
use crate::render::TEXT_BOUND;

/// One string of a label file: non-empty, at most [`TEXT_BOUND`] bytes, no control character.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Word(String);

impl Word {
    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Word {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        if text.is_empty() {
            return Err(de::Error::custom("BadLabel: a label string is empty"));
        }
        if text.len() > TEXT_BOUND {
            return Err(de::Error::custom(format!(
                "BadLabel: {text:?} is over {TEXT_BOUND} bytes"
            )));
        }
        if text.chars().any(char::is_control) {
            return Err(de::Error::custom(format!(
                "BadLabel: {text:?} holds a control character"
            )));
        }
        Ok(Self(text))
    }
}

/// A phrase of a label file: a [`Word`] with exactly one `{value}` and no other placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelPhrase(Word);

impl<'de> Deserialize<'de> for LabelPhrase {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let word = Word::deserialize(deserializer)?;
        if is_phrase(word.as_str()) {
            Ok(Self(word))
        } else {
            Err(de::Error::custom(format!(
                "BadLabel: the phrase {:?} must name exactly one {{value}} and no other placeholder",
                word.as_str()
            )))
        }
    }
}

/// A locale's joining words: what the renderer joins set members with, and what a reader joins texts,
/// ends a cut composition, separates a label from its text and shows a flag with.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JoiningWords {
    /// Between set members, except before the last.
    pub list: Word,
    /// Before the last set member.
    pub and: Word,
    /// Between the texts of a composition.
    pub join: Word,
    /// What a composition cut at an entry boundary ends with.
    pub more: Word,
    /// Between a label and its text in the own sheet.
    pub pair: Word,
    /// A true flag with no text, in the own sheet.
    pub yes: Word,
    /// A false flag, in the own sheet.
    pub no: Word,
}

impl JoiningWords {
    /// The words a reader needs, as a block carries them.
    pub fn for_readers(&self) -> ReaderWords {
        ReaderWords {
            join: self.join.0.clone(),
            more: self.more.0.clone(),
            pair: self.pair.0.clone(),
            yes: self.yes.0.clone(),
            no: self.no.0.clone(),
        }
    }

    fn fields(&self) -> [(&'static str, &Word); 7] {
        [
            ("list", &self.list),
            ("and", &self.and),
            ("join", &self.join),
            ("more", &self.more),
            ("pair", &self.pair),
            ("yes", &self.yes),
            ("no", &self.no),
        ]
    }
}

/// The joining words a disclosed block carries per locale (schema 3): only what a reader composes
/// with; `list` and `and` stay on the server, which alone joins set members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReaderWords {
    /// Between the texts of a composition.
    pub join: String,
    /// What a cut composition ends with.
    pub more: String,
    /// Between a label and its text in the own sheet.
    pub pair: String,
    /// A true flag, in the own sheet.
    pub yes: String,
    /// A false flag, in the own sheet.
    pub no: String,
}

impl ReaderWords {
    /// English, as S30-f's readers hard-coded it.
    pub fn english() -> Self {
        Self {
            join: ", ".to_owned(),
            more: " …".to_owned(),
            pair: ": ".to_owned(),
            yes: "yes".to_owned(),
            no: "no".to_owned(),
        }
    }
}

/// One attribute's entry in a label file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawAttributeLabels {
    #[serde(default)]
    label: Option<Word>,
    #[serde(default)]
    phrase: Option<LabelPhrase>,
    #[serde(default)]
    flag: Option<Word>,
    #[serde(default)]
    words: Option<BTreeMap<String, Word>>,
    #[serde(default)]
    perceived_as: Option<Vec<Word>>,
}

/// One `labels/<locale>.yaml` file, decoded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelFile {
    #[allow(dead_code)]
    format: FormatOne,
    locale: Spanned<String>,
    #[serde(default)]
    words: Option<JoiningWords>,
    #[serde(default)]
    labels: BTreeMap<String, Spanned<RawAttributeLabels>>,
}

/// One attribute's merged labels in one locale. Any piece may be missing; a missing piece makes the
/// entries that need it English, whole.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttributeLabels {
    /// The label.
    pub label: Option<String>,
    /// The phrase, with one `{value}`.
    pub phrase: Option<String>,
    /// The words of a true flag.
    pub flag: Option<String>,
    /// The words per value slug.
    pub words: BTreeMap<ValueSlug, String>,
    /// The words per band, in the declaration's band order.
    pub perceived_as: Option<Vec<String>>,
}

/// One locale's merged labels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleLabels {
    /// The joining words.
    pub words: JoiningWords,
    /// Each attribute's labels.
    pub attributes: BTreeMap<QualifiedId, AttributeLabels>,
}

/// Every label pack of one composition, merged: the offered locales are its keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    locales: BTreeMap<Locale, LocaleLabels>,
}

/// A refusal of a label file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelRefusal {
    /// The file.
    pub file: PathBuf,
    /// The refusal, named.
    pub reason: String,
}

fn refusal(file: &Path, reason: String) -> LabelRefusal {
    LabelRefusal {
        file: file.to_path_buf(),
        reason,
    }
}

/// One mergeable field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Field {
    Joining(&'static str),
    Label(String),
    Phrase(String),
    Flag(String),
    Word(String, String),
    Bands(String),
}

impl Field {
    fn describe(&self) -> String {
        match self {
            Self::Joining(word) => format!("words.{word}"),
            Self::Label(id) => format!("{id} label"),
            Self::Phrase(id) => format!("{id} phrase"),
            Self::Flag(id) => format!("{id} flag"),
            Self::Word(id, slug) => format!("{id} words.{slug}"),
            Self::Bands(id) => format!("{id} perceived_as"),
        }
    }
}

/// A raw merged value, with where it came from.
#[derive(Debug, Clone)]
enum Raw {
    Text(String),
    Bands(Vec<String>),
}

/// One locale's merge in progress: each field with its value and origin, and each id with the place
/// it was first written, for a refusal.
#[derive(Default)]
struct Merging {
    fields: BTreeMap<Field, (Raw, DataOrigin)>,
    written_at: BTreeMap<String, At>,
}

impl Merging {
    fn put(&mut self, field: Field, raw: Raw, origin: &DataOrigin) -> Result<(), LabelRefusal> {
        match self.fields.get(&field) {
            Some((_, first)) if first.source == DataSource::World && origin.source != DataSource::World => {
                Ok(())
            }
            Some((_, first)) if origin.source == DataSource::World && first.source != DataSource::World => {
                self.fields.insert(field, (raw, origin.clone()));
                Ok(())
            }
            Some((_, first)) => Err(refusal(
                &origin.path,
                format!(
                    "DuplicateLabel: {} is given in {} and again in {}",
                    field.describe(),
                    first.path.display().to_string().replace('\\', "/"),
                    origin.path.display().to_string().replace('\\', "/")
                ),
            )),
            None => {
                self.fields.insert(field, (raw, origin.clone()));
                Ok(())
            }
        }
    }
}

/// The stem of `path`.
fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

impl Labels {
    /// The labels of every file in collection order, checked against the built `catalogue`.
    ///
    /// # Errors
    ///
    /// The first refusal: `BadLabelFile`, `EnglishIsTheDeclaration`, `DuplicateLabel`,
    /// `UnknownAttribute`, `OutOfDomain` or `BadLabel`, naming the file and, where it has one, the line.
    pub fn build(
        files: Vec<(DataOrigin, LabelFile)>,
        catalogue: &Catalogue,
    ) -> Result<Self, LabelRefusal> {
        let mut merging: BTreeMap<Locale, Merging> = BTreeMap::new();
        for (origin, file) in files {
            let locale = Self::locale_of(&origin.path, &file)?;
            let merge = merging.entry(locale).or_default();
            if let Some(words) = &file.words {
                for (name, word) in words.fields() {
                    merge.put(Field::Joining(name), Raw::Text(word.0.clone()), &origin)?;
                }
            }
            for (id, entry) in &file.labels {
                merge
                    .written_at
                    .entry(id.clone())
                    .or_insert_with(|| At::spanned(&origin.path, entry));
                let raw = &entry.value;
                let id = id.clone();
                if let Some(label) = &raw.label {
                    merge.put(Field::Label(id.clone()), Raw::Text(label.0.clone()), &origin)?;
                }
                if let Some(phrase) = &raw.phrase {
                    let text = phrase.0.0.clone();
                    merge.put(Field::Phrase(id.clone()), Raw::Text(text), &origin)?;
                }
                if let Some(flag) = &raw.flag {
                    merge.put(Field::Flag(id.clone()), Raw::Text(flag.0.clone()), &origin)?;
                }
                for (slug, word) in raw.words.iter().flatten() {
                    let field = Field::Word(id.clone(), slug.clone());
                    merge.put(field, Raw::Text(word.0.clone()), &origin)?;
                }
                if let Some(bands) = &raw.perceived_as {
                    let bands = bands.iter().map(|word| word.0.clone()).collect();
                    merge.put(Field::Bands(id.clone()), Raw::Bands(bands), &origin)?;
                }
            }
        }
        let mut locales = BTreeMap::new();
        for (locale, merge) in merging {
            locales.insert(locale.clone(), Self::checked(&locale, merge, catalogue)?);
        }
        Ok(Self { locales })
    }

    /// The file's locale: canonical, its file name's stem, not English.
    fn locale_of(path: &Path, file: &LabelFile) -> Result<Locale, LabelRefusal> {
        let at = At::spanned(path, &file.locale);
        let written = &file.locale.value;
        if stem(path) == "en" || written == "en" {
            return Err(refusal(
                path,
                format!(
                    "{at}: EnglishIsTheDeclaration: English is the attributes' own declaration; \
                     change English words with an attributes/overrides.yaml, not a label file"
                ),
            ));
        }
        let locale = Locale::canonical(written)
            .map_err(|error| refusal(path, format!("{at}: BadLabelFile: {error}")))?;
        if stem(path) != locale.as_str() {
            return Err(refusal(
                path,
                format!(
                    "{at}: BadLabelFile: the locale '{locale}' is not the file name's stem '{}'",
                    stem(path)
                ),
            ));
        }
        Ok(locale)
    }

    /// One locale's merge, checked against the catalogue.
    fn checked(
        locale: &Locale,
        merge: Merging,
        catalogue: &Catalogue,
    ) -> Result<LocaleLabels, LabelRefusal> {
        let mut joining: BTreeMap<&'static str, Word> = BTreeMap::new();
        let mut attributes: BTreeMap<QualifiedId, AttributeLabels> = BTreeMap::new();
        let mut first_file: Option<PathBuf> = None;
        for (field, (raw, origin)) in merge.fields {
            first_file.get_or_insert_with(|| origin.path.clone());
            let text = |raw: &Raw| match raw {
                Raw::Text(text) => text.clone(),
                Raw::Bands(_) => String::new(),
            };
            let id_of = |id: &str| -> Result<Option<&Declaration>, LabelRefusal> {
                let at = merge
                    .written_at
                    .get(id)
                    .map(|at| format!("{at}: "))
                    .unwrap_or_default();
                let parsed = QualifiedId::new(id).ok();
                if let Some(declaration) = parsed.as_ref().and_then(|id| catalogue.get(id)) {
                    return Ok(Some(declaration));
                }
                if parsed.is_some_and(|id| catalogue.removed().contains_key(&id)) {
                    return Ok(None);
                }
                let known = catalogue.attributes().map(|declaration| declaration.id.as_str());
                Err(refusal(
                    &origin.path,
                    format!(
                        "{at}UnknownAttribute: '{id}' is declared by no catalogue of this world (nearest: {})",
                        nearest(id, known).join(", ")
                    ),
                ))
            };
            let bad = |id: &str, why: String| {
                let at = merge
                    .written_at
                    .get(id)
                    .map(|at| format!("{at}: "))
                    .unwrap_or_default();
                refusal(&origin.path, format!("{at}BadLabel: '{id}' {why}"))
            };
            match &field {
                Field::Joining(name) => {
                    joining.insert(name, Word(text(&raw)));
                }
                Field::Label(id) => {
                    if let Some(declaration) = id_of(id)? {
                        let entry = attributes.entry(declaration.id.clone()).or_default();
                        entry.label = Some(text(&raw));
                    }
                }
                Field::Phrase(id) => {
                    if let Some(declaration) = id_of(id)? {
                        let entry = attributes.entry(declaration.id.clone()).or_default();
                        entry.phrase = Some(text(&raw));
                    }
                }
                Field::Flag(id) => {
                    if let Some(declaration) = id_of(id)? {
                        if !matches!(declaration.kind, Kind::Flag) {
                            return Err(bad(
                                id,
                                format!(
                                    "field 'flag' is only for a flag, and it is a {}",
                                    declaration.kind.name()
                                ),
                            ));
                        }
                        let entry = attributes.entry(declaration.id.clone()).or_default();
                        entry.flag = Some(text(&raw));
                    }
                }
                Field::Word(id, slug) => {
                    if let Some(declaration) = id_of(id)? {
                        let Some(values) = declaration.kind.values() else {
                            return Err(bad(
                                id,
                                format!(
                                    "field 'words' is only for a category, level or set, and it is a {}",
                                    declaration.kind.name()
                                ),
                            ));
                        };
                        let Some(value) = values.iter().find(|value| value.as_str() == slug) else {
                            let at = merge
                                .written_at
                                .get(id)
                                .map(|at| format!("{at}: "))
                                .unwrap_or_default();
                            return Err(refusal(
                                &origin.path,
                                format!(
                                    "{at}OutOfDomain: '{id}' has no value '{slug}' (nearest: {})",
                                    nearest(slug, values.iter().map(ValueSlug::as_str)).join(", ")
                                ),
                            ));
                        };
                        let entry = attributes.entry(declaration.id.clone()).or_default();
                        entry.words.insert(value.clone(), text(&raw));
                    }
                }
                Field::Bands(id) => {
                    if let Some(declaration) = id_of(id)? {
                        let Raw::Bands(bands) = raw else {
                            continue;
                        };
                        let declared = declaration.perceived_as.as_ref().map_or(0, Vec::len);
                        if declared == 0 {
                            return Err(bad(
                                id,
                                "field 'perceived_as' is only for an attribute declared with bands"
                                    .to_owned(),
                            ));
                        }
                        if bands.len() != declared {
                            return Err(bad(
                                id,
                                format!(
                                    "field 'perceived_as' lists {} band(s), and the declaration has {declared}",
                                    bands.len()
                                ),
                            ));
                        }
                        let entry = attributes.entry(declaration.id.clone()).or_default();
                        entry.perceived_as = Some(bands);
                    }
                }
            }
        }
        let mut take = |name: &'static str| {
            joining.remove(name).ok_or_else(|| {
                refusal(
                    first_file.as_deref().unwrap_or(Path::new("labels")),
                    format!(
                        "BadLabelFile: no labels/{locale}.yaml of this world's packs gives words.{name}; \
                         one file of each locale gives all of list, and, join, more, pair, yes, no"
                    ),
                )
            })
        };
        let words = JoiningWords {
            list: take("list")?,
            and: take("and")?,
            join: take("join")?,
            more: take("more")?,
            pair: take("pair")?,
            yes: take("yes")?,
            no: take("no")?,
        };
        Ok(LocaleLabels { words, attributes })
    }

    /// The offered locales, sorted.
    pub fn locales(&self) -> impl Iterator<Item = &Locale> {
        self.locales.keys()
    }

    /// Every offered locale with its labels, sorted.
    pub fn iter(&self) -> impl Iterator<Item = (&Locale, &LocaleLabels)> {
        self.locales.iter()
    }

    /// One locale's labels, when offered.
    pub fn of(&self, locale: &Locale) -> Option<&LocaleLabels> {
        self.locales.get(locale)
    }

    /// Whether no locale is offered.
    pub fn is_empty(&self) -> bool {
        self.locales.is_empty()
    }

    /// The labels `attributes`' context holds among a provider's `contexts` (§5.7): `None` where the
    /// world does not enable `attributes`.
    pub fn from_contexts(
        contexts: &mineworld_authoring::ProviderContexts<'_>,
    ) -> Option<std::sync::Arc<Self>> {
        contexts
            .of(&crate::ATTRIBUTES)?
            .as_any()
            .downcast_ref::<crate::AttributesContext>()
            .map(|context| context.labels.clone())
    }
}

/// What one locale lacks for one declaration, each as `<id> <piece>`: `label`, `phrase` (a flag's is
/// optional), `flag`, `words.<slug>`, `perceived_as`.
pub fn missing(declaration: &Declaration, labels: Option<&AttributeLabels>) -> Vec<String> {
    let empty = AttributeLabels::default();
    let labels = labels.unwrap_or(&empty);
    let id = &declaration.id;
    let mut out = Vec::new();
    if labels.label.is_none() {
        out.push(format!("{id} label"));
    }
    if labels.phrase.is_none() && !matches!(declaration.kind, Kind::Flag) {
        out.push(format!("{id} phrase"));
    }
    if matches!(declaration.kind, Kind::Flag) && labels.flag.is_none() {
        out.push(format!("{id} flag"));
    }
    for slug in declaration.kind.values().unwrap_or_default() {
        if !labels.words.contains_key(slug) {
            out.push(format!("{id} words.{slug}"));
        }
    }
    if declaration.perceived_as.is_some() && labels.perceived_as.is_none() {
        out.push(format!("{id} perceived_as"));
    }
    out
}

#[cfg(test)]
mod tests;
