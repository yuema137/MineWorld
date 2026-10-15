//! Reading `decisions.yaml`: one positioned pass, then every refusal by name with its line and column
//! (`docs/MODULE_SPEC.md` §4.1; `pr-dl-a-decision-list.md` SD-DLa-5 … SD-DLa-9).
//!
//! ```text
//! the pass        unknown keys, shapes and types — serde's own refusals, which the YAML decoder
//!                 positions; each model's parameters, decoded by the model's own type where they stand
//! the checks      every model id installed; fallback only on external models and chains of at most
//!                 four links with no cycle; seats are seats, classes are defined classes of persons;
//!                 one entry per seat and per class; memory policy ids spelled — each at the node's
//!                 position, carried through the pass by `serde_saphyr::Spanned`
//! ```
//!
//! A refusal raised with `serde::de::Error::custom` carries no position (serde-saphyr keeps custom
//! errors locationless), so every check that judges a value against the catalog or the roster runs after
//! the pass on a [`Spanned`] value and builds its [`DecisionsError`] from that value's own position.

use core::fmt;
use core::marker::PhantomData;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use mineworld_contracts::EntityKey;
use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer, Error as _, IgnoredAny, MapAccess, Visitor};
use serde_json::{Map, Value};
use serde_saphyr::Spanned;

use crate::format::{ClassRef, JsonObject, PolicyId, Position, Roster};
use crate::model::ModelId;
use crate::models::{Catalog, Decoded};

/// A decision list refused: the file, where in it, and what was wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionsError {
    /// The file.
    pub path: PathBuf,
    /// The line of the offending node, 1-based; 0 when the YAML decoder could not say.
    pub line: u64,
    /// Its column, 1-based; 0 when unknown.
    pub column: u64,
    /// What was wrong, naming the node.
    pub detail: String,
}

impl fmt::Display for DecisionsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.path.display(), self.detail)
        } else {
            write!(
                f,
                "{}:{}:{}: {}",
                self.path.display(),
                self.line,
                self.column,
                self.detail
            )
        }
    }
}

impl std::error::Error for DecisionsError {}

/// Where a chain goes after a model: another model, or nowhere (the seat stays idle).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallback {
    /// The next model.
    Model(ModelId),
    /// `none`: the seat stays idle.
    None,
}

/// One model the file lists under `models:`.
#[derive(Debug, Clone)]
pub(crate) struct Listed {
    /// An in-process model's parameters, decoded (defaults when the entry gave none).
    pub(crate) parameters: Option<Decoded>,
    pub(crate) fallback: Option<Fallback>,
}

/// A `memory:` choice as written: passed through, never interpreted (QFGb-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryChoice {
    /// The policy.
    pub policy: PolicyId,
    /// Its fallback policy, if the entry names one.
    pub fallback: Option<PolicyId>,
    /// Its parameters, as JSON; empty when the entry gives none.
    pub parameters: Map<String, Value>,
    /// Where the entry was written.
    pub at: Position,
}

/// The `memory:` section, validated.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MemorySection {
    pub(crate) default: Option<MemoryChoice>,
    pub(crate) seats: BTreeMap<EntityKey, MemoryChoice>,
    pub(crate) classes: BTreeMap<ClassRef, MemoryChoice>,
}

/// A decision list, decoded and checked against one world and one build's models; resolved per seat by
/// [`DecisionList::resolve`](crate::DecisionList::resolve).
#[derive(Debug, Clone)]
pub struct DecisionList<C> {
    /// The file as written in `world.yaml` or passed to `--decisions`; empty for no file.
    pub(crate) file: String,
    pub(crate) models: BTreeMap<ModelId, Listed>,
    pub(crate) default: Option<ModelId>,
    pub(crate) seats: BTreeMap<EntityKey, ModelId>,
    pub(crate) classes: BTreeMap<ClassRef, ModelId>,
    pub(crate) memory: Option<MemorySection>,
    /// The build's models every id above was checked against.
    catalog: PhantomData<C>,
}

impl<C: Catalog> DecisionList<C> {
    /// No decision list: every seat `rule`, no memory choice.
    pub fn none() -> Self {
        Self {
            file: String::new(),
            models: BTreeMap::new(),
            default: None,
            seats: BTreeMap::new(),
            classes: BTreeMap::new(),
            memory: None,
            catalog: PhantomData,
        }
    }

    /// Reads and checks the decision list at `path`, written as `file` (the text a refusal and a memory
    /// choice's source name), against `roster` and this build's models `C`.
    ///
    /// # Errors
    ///
    /// The file unreadable, or the first refusal, with its line and column.
    pub fn read(path: &Path, file: &str, roster: &Roster) -> Result<Self, DecisionsError> {
        let text = std::fs::read_to_string(path).map_err(|error| DecisionsError {
            path: path.to_path_buf(),
            line: 0,
            column: 0,
            detail: format!("cannot be read: {error}"),
        })?;
        Self::parse(&text, path, file, roster)
    }

    /// [`DecisionList::read`] on text already read.
    ///
    /// # Errors
    ///
    /// The first refusal, with its line and column.
    pub fn parse(
        text: &str,
        path: &Path,
        file: &str,
        roster: &Roster,
    ) -> Result<Self, DecisionsError> {
        let raw = serde_saphyr::with_deserializer_from_str(text, |document| {
            FileSeed::<C>(PhantomData).deserialize(document)
        })
        .map_err(|error| {
            let at = error.location();
            DecisionsError {
                path: path.to_path_buf(),
                line: at.map_or(0, |at| at.line()),
                column: at.map_or(0, |at| at.column()),
                detail: error.to_string(),
            }
        })?;
        Checker {
            path,
            roster,
            _catalog: PhantomData::<C>,
        }
        .check(raw, file)
    }

    /// How many assignments the file makes, seat and class.
    pub fn assignments(&self) -> usize {
        self.seats.len() + self.classes.len()
    }

    /// The model `default:` names, else `rule`.
    pub fn default_model(&self) -> &str {
        self.default.as_ref().map_or("rule", ModelId::as_str)
    }

    /// Whether the file has a `memory:` section.
    pub fn has_memory(&self) -> bool {
        self.memory.is_some()
    }
}

// ---------------------------------------------------------------------------------------------------
// The pass: shapes and types, models' parameters where they stand, positions kept.

/// The file as the pass read it, before any check against the catalog or the roster.
struct Raw<C> {
    models: Vec<RawModel<C>>,
    default: Option<Spanned<String>>,
    assign: Vec<Spanned<RawAssign>>,
    memory: Option<RawMemory>,
}

struct RawModel<C> {
    id: Spanned<String>,
    /// `None` for an id this build does not install: its entry was skipped unread.
    model: Option<C>,
    parameters: Option<Decoded>,
    /// Where an external model was given `parameters`, which this build cannot type.
    external_parameters: Option<Position>,
    fallback: Option<Spanned<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAssign {
    #[serde(default)]
    seat: Option<Spanned<String>>,
    #[serde(default)]
    class: Option<Spanned<String>>,
    model: Spanned<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMemory {
    #[serde(default)]
    default: Option<Spanned<RawPolicy>>,
    #[serde(default)]
    assign: Vec<Spanned<RawMemoryAssign>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicy {
    policy: Spanned<String>,
    #[serde(default)]
    fallback: Option<Spanned<String>>,
    #[serde(default)]
    parameters: Option<JsonObject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMemoryAssign {
    #[serde(default)]
    seat: Option<Spanned<String>>,
    #[serde(default)]
    class: Option<Spanned<String>>,
    policy: Spanned<String>,
    #[serde(default)]
    fallback: Option<Spanned<String>>,
    #[serde(default)]
    parameters: Option<JsonObject>,
}

/// The file's top-level keys, in the order a refusal lists them.
pub(crate) const FILE_KEYS: &[&str] = &["models", "default", "assign", "memory"];

/// A `models:` entry's keys.
pub(crate) const ENTRY_KEYS: &[&str] = &["parameters", "fallback"];

struct FileSeed<C>(PhantomData<C>);

impl<'de, C: Catalog> DeserializeSeed<'de> for FileSeed<C> {
    type Value = Raw<C>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Raw<C>, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, C: Catalog> Visitor<'de> for FileSeed<C> {
    type Value = Raw<C>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a decision list: models, default, assign, memory")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Raw<C>, A::Error> {
        let mut raw = Raw {
            models: Vec::new(),
            default: None,
            assign: Vec::new(),
            memory: None,
        };
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "models" => raw.models = map.next_value_seed(ModelsSeed::<C>(PhantomData))?,
                "default" => raw.default = Some(map.next_value()?),
                "assign" => raw.assign = map.next_value()?,
                "memory" => raw.memory = Some(map.next_value()?),
                other => return Err(A::Error::unknown_field(other, FILE_KEYS)),
            }
        }
        Ok(raw)
    }
}

struct ModelsSeed<C>(PhantomData<C>);

impl<'de, C: Catalog> DeserializeSeed<'de> for ModelsSeed<C> {
    type Value = Vec<RawModel<C>>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, C: Catalog> Visitor<'de> for ModelsSeed<C> {
    type Value = Vec<RawModel<C>>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a mapping of model ids to their entries")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut models = Vec::new();
        while let Some(id) = map.next_key::<Spanned<String>>()? {
            let Some(model) = C::find(&id.value) else {
                // Refused after the pass, at the key; its entry is not this build's to read.
                map.next_value::<IgnoredAny>()?;
                models.push(RawModel {
                    id,
                    model: None,
                    parameters: None,
                    external_parameters: None,
                    fallback: None,
                });
                continue;
            };
            let entry = map.next_value_seed(EntrySeed(model))?;
            models.push(RawModel {
                id,
                model: Some(model),
                parameters: entry.parameters,
                external_parameters: entry.external_parameters,
                fallback: entry.fallback,
            });
        }
        Ok(models)
    }
}

/// One model's entry: `parameters` decoded by the model's own type, `fallback` kept with its position.
struct EntrySeed<C>(C);

struct Entry {
    parameters: Option<Decoded>,
    external_parameters: Option<Position>,
    fallback: Option<Spanned<String>>,
}

impl<'de, C: Catalog> DeserializeSeed<'de> for EntrySeed<C> {
    type Value = Entry;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Entry, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, C: Catalog> Visitor<'de> for EntrySeed<C> {
    type Value = Entry;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a model entry: parameters, fallback")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Entry, A::Error> {
        let mut entry = Entry {
            parameters: None,
            external_parameters: None,
            fallback: None,
        };
        while let Some(key) = map.next_key::<Spanned<String>>()? {
            match key.value.as_str() {
                "parameters" if self.0.is_external() => {
                    map.next_value::<IgnoredAny>()?;
                    entry.external_parameters = Some(Position::from(&key));
                }
                "parameters" => {
                    entry.parameters = Some(map.next_value_seed(ParametersSeed(self.0))?)
                }
                "fallback" => entry.fallback = Some(map.next_value()?),
                other => return Err(A::Error::unknown_field(other, ENTRY_KEYS)),
            }
        }
        Ok(entry)
    }
}

struct ParametersSeed<C>(C);

impl<'de, C: Catalog> DeserializeSeed<'de> for ParametersSeed<C> {
    type Value = Decoded;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Decoded, D::Error> {
        self.0.decode(deserializer)
    }
}

// ---------------------------------------------------------------------------------------------------
// The checks: every judgement against the catalog and the roster, at the node's own position.

struct Checker<'a, C> {
    path: &'a Path,
    roster: &'a Roster,
    _catalog: PhantomData<C>,
}

/// What an assignment names.
enum Target {
    Seat(EntityKey),
    Class(ClassRef),
}

impl<C: Catalog> Checker<'_, C> {
    fn refuse<T>(&self, at: Position, detail: impl Into<String>) -> Result<T, DecisionsError> {
        Err(DecisionsError {
            path: self.path.to_path_buf(),
            line: at.line,
            column: at.column,
            detail: detail.into(),
        })
    }

    fn check(&self, raw: Raw<C>, file: &str) -> Result<DecisionList<C>, DecisionsError> {
        let mut list = DecisionList::<C>::none();
        list.file = file.to_owned();
        let mut fallbacks: BTreeMap<ModelId, Position> = BTreeMap::new();
        for entry in raw.models {
            let at = Position::from(&entry.id);
            let id = self.model_id(&entry.id, "models")?;
            let Some(model) = entry.model else {
                return self.unknown_model(at, "models", &id);
            };
            if let Some(at) = entry.external_parameters {
                return self.refuse(
                    at,
                    format!(
                        "models.{id}.parameters: '{id}' runs out of process; this build cannot type \
                         its parameters"
                    ),
                );
            }
            let fallback = match &entry.fallback {
                None if model.is_external() => {
                    return self.refuse(
                        at,
                        format!(
                            "models.{id}: '{id}' runs out of process and must name a fallback (an \
                             in-process model, or none)"
                        ),
                    );
                }
                None => None,
                Some(spanned) if !model.is_external() => {
                    return self.refuse(
                        Position::from(spanned),
                        format!(
                            "models.{id}.fallback: '{id}' runs in process and is always present; only \
                             an external model names a fallback"
                        ),
                    );
                }
                Some(spanned) => {
                    fallbacks.insert(id.clone(), Position::from(spanned));
                    Some(self.fallback(spanned, &id)?)
                }
            };
            let parameters = entry.parameters.or_else(|| model.defaults());
            list.models.insert(
                id,
                Listed {
                    parameters,
                    fallback,
                },
            );
        }
        self.check_chains(&list, &fallbacks)?;
        if let Some(default) = &raw.default {
            let id = self.model_id(default, "default")?;
            self.installed(&list, Position::from(default), "default", &id)?;
            list.default = Some(id);
        }
        for (index, entry) in raw.assign.iter().enumerate() {
            let place = format!("assign entry {}", index + 1);
            let target = self.target(
                &entry.value.seat,
                &entry.value.class,
                Position::from(entry),
                &place,
            )?;
            let id = self.model_id(&entry.value.model, &place)?;
            self.installed(&list, Position::from(&entry.value.model), &place, &id)?;
            let fresh = match target {
                Target::Seat(seat) => list.seats.insert(seat, id).is_none(),
                Target::Class(class) => list.classes.insert(class, id).is_none(),
            };
            if !fresh {
                return self.refuse(
                    Position::from(entry),
                    format!(
                        "{place}: a second entry for the same {}",
                        which(&entry.value.seat)
                    ),
                );
            }
        }
        if let Some(memory) = &raw.memory {
            list.memory = Some(self.memory(memory)?);
        }
        Ok(list)
    }

    fn model_id(&self, spanned: &Spanned<String>, place: &str) -> Result<ModelId, DecisionsError> {
        ModelId::new(spanned.value.as_str()).or_else(|error| {
            self.refuse(
                Position::from(spanned),
                format!("{place}: '{}' is not a model id: {error}", spanned.value),
            )
        })
    }

    fn unknown_model<T>(
        &self,
        at: Position,
        place: &str,
        id: &ModelId,
    ) -> Result<T, DecisionsError> {
        self.refuse(
            at,
            format!(
                "{place}: '{id}' is not a model this build installs; it installs: {}",
                C::listed()
            ),
        )
    }

    /// A model an assignment or `default:` names: installed, and an external one listed with its
    /// fallback.
    fn installed(
        &self,
        list: &DecisionList<C>,
        at: Position,
        place: &str,
        id: &ModelId,
    ) -> Result<(), DecisionsError> {
        let Some(model) = C::find(id.as_str()) else {
            return self.unknown_model(at, place, id);
        };
        if model.is_external() && !list.models.contains_key(id) {
            return self.refuse(
                at,
                format!(
                    "{place}: '{id}' runs out of process and has no fallback; list it under models: \
                     with one"
                ),
            );
        }
        Ok(())
    }

    fn fallback(
        &self,
        spanned: &Spanned<String>,
        of: &ModelId,
    ) -> Result<Fallback, DecisionsError> {
        if spanned.value == "none" {
            return Ok(Fallback::None);
        }
        let place = format!("models.{of}.fallback");
        let id = self.model_id(spanned, &place)?;
        if C::find(id.as_str()).is_none() {
            return self.unknown_model(Position::from(spanned), &place, &id);
        }
        Ok(Fallback::Model(id))
    }

    /// Every external model's chain: at most four links, no cycle, every external link listed with its
    /// own fallback; refused at the `fallback:` that starts the chain.
    fn check_chains(
        &self,
        list: &DecisionList<C>,
        fallbacks: &BTreeMap<ModelId, Position>,
    ) -> Result<(), DecisionsError> {
        for (start, at) in fallbacks {
            let place = format!("models.{start}.fallback");
            let mut seen = vec![start.clone()];
            let mut current = start;
            while let Some(Fallback::Model(next)) = list
                .models
                .get(current)
                .and_then(|listed| listed.fallback.as_ref())
            {
                if seen.contains(next) {
                    return self.refuse(
                        *at,
                        format!("{place}: the fallback chain from '{start}' returns to '{next}'"),
                    );
                }
                seen.push(next.clone());
                if seen.len() - 1 > CHAIN_LINKS {
                    return self.refuse(
                        *at,
                        format!(
                            "{place}: the fallback chain from '{start}' has more than {CHAIN_LINKS} \
                             links ({})",
                            chain_text(&seen)
                        ),
                    );
                }
                let external = C::find(next.as_str()).is_some_and(Catalog::is_external);
                if !external {
                    break;
                }
                if !list.models.contains_key(next) {
                    return self.refuse(
                        *at,
                        format!(
                            "{place}: '{next}' runs out of process and has no fallback; list it under \
                             models: with one"
                        ),
                    );
                }
                current = next;
            }
        }
        Ok(())
    }

    /// What an entry names: exactly one of a seat and a class, each checked against the roster.
    fn target(
        &self,
        seat: &Option<Spanned<String>>,
        class: &Option<Spanned<String>>,
        entry: Position,
        place: &str,
    ) -> Result<Target, DecisionsError> {
        match (seat, class) {
            (Some(_), Some(_)) | (None, None) => self.refuse(
                entry,
                format!("{place}: an entry names exactly one of 'seat' and 'class'"),
            ),
            (Some(seat), None) => self.seat(seat, place).map(Target::Seat),
            (None, Some(class)) => self.class(class, place).map(Target::Class),
        }
    }

    fn seat(&self, spanned: &Spanned<String>, place: &str) -> Result<EntityKey, DecisionsError> {
        let at = Position::from(spanned);
        let Ok(key) = EntityKey::new(spanned.value.as_str()) else {
            return self.refuse(
                at,
                format!("{place}: '{}' is not a person's key", spanned.value),
            );
        };
        if !self.roster.people.contains(&key) {
            return self.refuse(
                at,
                format!("{place}: seat '{key}' names no person of this world"),
            );
        }
        if self.roster.seat(&key).is_none() {
            return self.refuse(
                at,
                format!(
                    "{place}: '{key}' is a person but not a seat (world.yaml seats:), so nothing would \
                     drive it"
                ),
            );
        }
        Ok(key)
    }

    fn class(&self, spanned: &Spanned<String>, place: &str) -> Result<ClassRef, DecisionsError> {
        let at = Position::from(spanned);
        let Ok(class) = ClassRef::new(spanned.value.as_str()) else {
            return self.refuse(
                at,
                format!("{place}: '{}' is not a class name", spanned.value),
            );
        };
        if self.roster.implicit.contains(&class) {
            return self.refuse(
                at,
                format!(
                    "{place}: '{class}' is an implicit class, which every person of its type is in; \
                     use default: instead"
                ),
            );
        }
        if !self.roster.classes.contains(&class) {
            return self.refuse(
                at,
                format!(
                    "{place}: class '{class}' is not a class of persons defined in \
                     configure/classes.yaml"
                ),
            );
        }
        Ok(class)
    }

    fn memory(&self, memory: &RawMemory) -> Result<MemorySection, DecisionsError> {
        let mut section = MemorySection::default();
        if let Some(default) = &memory.default {
            section.default = Some(self.choice(
                &default.value.policy,
                default.value.fallback.as_ref(),
                default.value.parameters.as_ref(),
                Position::from(default),
                "memory.default",
            )?);
        }
        for (index, entry) in memory.assign.iter().enumerate() {
            let place = format!("memory.assign entry {}", index + 1);
            let at = Position::from(entry);
            let target = self.target(&entry.value.seat, &entry.value.class, at, &place)?;
            let choice = self.choice(
                &entry.value.policy,
                entry.value.fallback.as_ref(),
                entry.value.parameters.as_ref(),
                at,
                &place,
            )?;
            let fresh = match target {
                Target::Seat(seat) => section.seats.insert(seat, choice).is_none(),
                Target::Class(class) => section.classes.insert(class, choice).is_none(),
            };
            if !fresh {
                return self.refuse(
                    at,
                    format!(
                        "{place}: a second entry for the same {}",
                        which(&entry.value.seat)
                    ),
                );
            }
        }
        Ok(section)
    }

    fn choice(
        &self,
        policy: &Spanned<String>,
        fallback: Option<&Spanned<String>>,
        parameters: Option<&JsonObject>,
        at: Position,
        place: &str,
    ) -> Result<MemoryChoice, DecisionsError> {
        let spelled = |spanned: &Spanned<String>, key: &str| {
            PolicyId::try_from(spanned.value.clone()).or_else(|refusal| {
                self.refuse(Position::from(spanned), format!("{place}.{key}: {refusal}"))
            })
        };
        Ok(MemoryChoice {
            policy: spelled(policy, "policy")?,
            fallback: fallback
                .map(|spanned| spelled(spanned, "fallback"))
                .transpose()?,
            parameters: parameters
                .map(|object| object.0.clone())
                .unwrap_or_default(),
            at,
        })
    }
}

/// The longest fallback chain, in `fallback:` steps (`docs/MODULE_SPEC.md` §4.1 rule 4).
pub const CHAIN_LINKS: usize = 4;

fn chain_text(chain: &[ModelId]) -> String {
    chain
        .iter()
        .map(ModelId::as_str)
        .collect::<Vec<_>>()
        .join(" → ")
}

fn which(seat: &Option<Spanned<String>>) -> &'static str {
    if seat.is_some() { "seat" } else { "class" }
}
