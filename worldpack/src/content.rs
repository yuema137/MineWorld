//! Reading one content file (person, place, item kind or organization): the format's own fields, and the sections System Packs own.
//!
//! A derived `Deserialize` with `deny_unknown_fields` cannot express this file: which top-level keys
//! are legal depends on the sections this build's packs declare (`DECISIONS.md` `ARC-31`). So the file
//! is read through a [`DeserializeSeed`] that knows them, and that keeps the rule the derive kept —
//! **an unknown key is refused**, in serde's own words, at the line that wrote it.
//!
//! ```text
//! a field of the format              → its typed field
//! a section an enabled pack owns     → decoded by that pack's type, straight from the stream
//! a section of a pack not enabled    → recorded, not decoded: `read` refuses it naming the pack
//! a section this kind may not carry  → recorded, not decoded: `read` refuses it naming the kinds
//! anything else                      → refused: unknown field, listing every legal key
//! ```
//!
//! This module never learns what a section means. It asks the catalog who owns a key and has the owner
//! decode it.

use mineworld_authoring::ContentKind;
use mineworld_contracts::Tags;
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, Visitor};

use crate::catalog::{AVAILABLE, Capability, SectionOwner};
use crate::format::{
    AuthoredItem, AuthoredLocation, AuthoredOrganization, AuthoredPassage, AuthoredPerson,
    AuthoredPlace, FoundSection, SectionState,
};

/// The format's own fields of a person file, in the order a refusal lists them.
pub const PERSON_FIELDS: &[&str] = &["tags", "note", "location"];

/// The format's own fields of a place file, in the order a refusal lists them.
pub const PLACE_FIELDS: &[&str] = &["tags", "note", "passages"];

/// The format's own fields of an item file, in the order a refusal lists them (`ARC-36`).
pub const ITEM_FIELDS: &[&str] = &["tags", "note"];

/// The format's own fields of an organization file, in the order a refusal lists them.
pub const ORGANIZATION_FIELDS: &[&str] = &["tags", "note"];

/// The format's own fields of one kind of file. The only place a kind's legal fields are stated, so a
/// `location` on an item is an unknown key, refused at its line like any other.
pub const fn fields(kind: ContentKind) -> &'static [&'static str] {
    match kind {
        ContentKind::Person => PERSON_FIELDS,
        ContentKind::Place => PLACE_FIELDS,
        ContentKind::Item => ITEM_FIELDS,
        ContentKind::Organization => ORGANIZATION_FIELDS,
    }
}

/// Everything one content file may state, before it is known which kind it is.
#[derive(Default)]
pub(crate) struct Fields {
    tags: Option<Tags>,
    note: Option<String>,
    location: Option<AuthoredLocation>,
    passages: Option<Vec<AuthoredPassage>>,
    sections: Vec<FoundSection>,
}

/// Reads one content file of `kind` in a world that enables `enabled`.
pub(crate) struct ContentFile<'a> {
    kind: ContentKind,
    enabled: &'a [Capability],
}

impl<'a> ContentFile<'a> {
    pub(crate) const fn new(kind: ContentKind, enabled: &'a [Capability]) -> Self {
        Self { kind, enabled }
    }

    /// The person this file states. Only meaningful for [`ContentKind::Person`].
    pub(crate) fn person<'de, D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<AuthoredPerson, D::Error> {
        let fields = self.deserialize(deserializer)?;
        Ok(AuthoredPerson {
            tags: fields.tags.unwrap_or_default(),
            note: fields.note,
            location: fields.location,
            sections: fields.sections,
        })
    }

    /// The place this file states. Only meaningful for [`ContentKind::Place`].
    pub(crate) fn place<'de, D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<AuthoredPlace, D::Error> {
        let fields = self.deserialize(deserializer)?;
        Ok(AuthoredPlace {
            tags: fields.tags.unwrap_or_default(),
            note: fields.note,
            passages: fields.passages.unwrap_or_default(),
            sections: fields.sections,
        })
    }

    /// The item kind this file states. Only meaningful for [`ContentKind::Item`].
    pub(crate) fn item<'de, D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<AuthoredItem, D::Error> {
        let fields = self.deserialize(deserializer)?;
        Ok(AuthoredItem {
            tags: fields.tags.unwrap_or_default(),
            note: fields.note,
            sections: fields.sections,
        })
    }

    /// The organization this file states. Only meaningful for [`ContentKind::Organization`].
    pub(crate) fn organization<'de, D: Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<AuthoredOrganization, D::Error> {
        let fields = self.deserialize(deserializer)?;
        Ok(AuthoredOrganization {
            tags: fields.tags.unwrap_or_default(),
            note: fields.note,
            sections: fields.sections,
        })
    }

    /// Every key a file of this kind may state, for the refusal of one it may not.
    fn legal_keys(&self) -> String {
        let sections = AVAILABLE
            .into_iter()
            .filter_map(Capability::section)
            .filter(|section| section.carried_by(self.kind))
            .map(|section| section.name.as_str());
        fields(self.kind)
            .iter()
            .copied()
            .chain(sections)
            .map(|key| format!("`{key}`"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl<'de> DeserializeSeed<'de> for ContentFile<'_> {
    type Value = Fields;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Fields, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for ContentFile<'_> {
    type Value = Fields;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "a {} file: a map of {}",
            self.kind,
            self.legal_keys()
        )
    }

    /// An empty file states nothing, which is a legal thing for a content file to state.
    fn visit_unit<E: de::Error>(self) -> Result<Fields, E> {
        Ok(Fields::default())
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Fields, A::Error> {
        let mut fields = Fields::default();
        while let Some(key) = map.next_key_seed(KeyOf { file: &self })? {
            match key {
                Key::Field("tags") => fields.tags = Some(map.next_value()?),
                Key::Field("note") => fields.note = map.next_value()?,
                Key::Field("location") => fields.location = map.next_value()?,
                Key::Field("passages") => fields.passages = Some(map.next_value()?),
                Key::Field(other) => {
                    return Err(de::Error::custom(format!(
                        "`{other}` is a field this reader does not handle"
                    )));
                }
                Key::Section(owner, section) => {
                    let state = if !section.carried_by(self.kind) {
                        map.next_value::<IgnoredAny>()?;
                        SectionState::NotCarriedHere
                    } else if !self.enabled.contains(&owner) {
                        map.next_value::<IgnoredAny>()?;
                        SectionState::OwnerNotEnabled
                    } else {
                        SectionState::Decoded(owner.decode_section(&mut map)?)
                    };
                    fields.sections.push(FoundSection {
                        owner,
                        name: section.name,
                        state,
                    });
                }
            }
        }
        Ok(fields)
    }
}

/// A top-level key of a content file, recognized.
enum Key {
    /// One of the format's own fields for this kind of file.
    Field(&'static str),
    /// A section, and the capability that owns it.
    Section(Capability, SectionOwner),
}

/// Recognizes a key **while the parser is reading it**, so that an unknown key is refused at its
/// own line and column: an error raised later, from the map as a whole, would lose the position the
/// author needs (`DEP-10`).
struct KeyOf<'f, 'a> {
    file: &'f ContentFile<'a>,
}

impl<'de> DeserializeSeed<'de> for KeyOf<'_, '_> {
    type Value = Key;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Key, D::Error> {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyOf<'_, '_> {
    type Value = Key;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "one of {}", self.file.legal_keys())
    }

    fn visit_str<E: de::Error>(self, key: &str) -> Result<Key, E> {
        if let Some(field) = fields(self.file.kind).iter().find(|field| **field == key) {
            return Ok(Key::Field(field));
        }
        match Capability::owning_section(key) {
            Some((owner, section)) => Ok(Key::Section(owner, section)),
            None => Err(E::custom(format!(
                "unknown field `{key}`, expected one of {}",
                self.file.legal_keys()
            ))),
        }
    }
}
