//! A display name: what people call somebody or somewhere, and its forms in other languages.

use std::collections::BTreeMap;

use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The longest name, in bytes.
pub const NAME_MAX_BYTES: usize = 64;

/// What a name may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "a name must be 1 to {max} bytes, with no control characters and no space at either end, but \
     got {name:?}"
)]
pub struct InvalidName {
    /// What was offered.
    pub name: String,
    /// The longest name this pack records.
    pub max: usize,
}

/// A name, as people say it, such as Alice Moreau or The Copper Kettle: 1 to 64 bytes in any script,
/// with no control character and no space at either end.
///
/// Free text, unlike a key: names are content a person reads, in any script. Bounded, because it
/// reaches the event log and the words other people say (`UTTERANCE_MAX_BYTES` has to hold a sentence
/// that quotes it). No control character, so a name cannot break a line of a transcript or a
/// biography, and no surrounding space, so two spellings of one name cannot differ invisibly.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(example = &"Ana Lee")]
pub struct Name(String);

impl Name {
    /// A name, refused unless it keeps the rule above.
    ///
    /// # Errors
    ///
    /// [`InvalidName`] naming what was offered.
    pub fn new(name: impl Into<String>) -> Result<Self, InvalidName> {
        let name = name.into();
        let valid = !name.is_empty()
            && name.len() <= NAME_MAX_BYTES
            && !name.chars().any(char::is_control)
            && name.trim() == name;
        if valid {
            Ok(Self(name))
        } else {
            Err(InvalidName {
                name,
                max: NAME_MAX_BYTES,
            })
        }
    }

    /// The name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Name {
    type Error = InvalidName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}

impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The most forms one name may carry besides its default.
pub const FORMS_MAX: usize = 16;

/// What a locale key may not be.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "a locale must be a catalog stem such as en, zh_Hans, pt_BR or sr_Latn_RS (2 or 3 lower-case \
     letters, then optionally _ and a script such as Hans, then optionally _ and a region such as BR), \
     but got {locale:?}"
)]
pub struct InvalidLocale {
    /// What was offered.
    pub locale: String,
}

/// A language a name is written in, spelled as the stem of a gettext catalog (`ARC-70`): `en`,
/// `zh_Hans`, `fr`, `pt_BR`.
///
/// The catalog stem and not a BCP 47 tag, so a client compares it directly with the locale its
/// settings report, with no conversion table. Checked when read: 2 or 3 lower-case letters, then
/// optionally `_` and a four-letter script (`Hans`), then optionally `_` and a two-letter region (`BR`).
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(example = &"zh_Hans")]
pub struct Locale(String);

impl Locale {
    /// A locale, refused unless it is spelled as a catalog stem.
    ///
    /// # Errors
    ///
    /// [`InvalidLocale`] naming what was offered.
    pub fn new(locale: impl Into<String>) -> Result<Self, InvalidLocale> {
        let locale = locale.into();
        if is_catalog_stem(&locale) {
            Ok(Self(locale))
        } else {
            Err(InvalidLocale { locale })
        }
    }

    /// The locale.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// `^[a-z]{2,3}(_[A-Z][a-z]{3})?(_[A-Z]{2})?$`, without a regular-expression engine.
fn is_catalog_stem(locale: &str) -> bool {
    let mut parts = locale.split('_');
    let language = parts.next().unwrap_or_default();
    if !(2..=3).contains(&language.len()) || !language.bytes().all(|b| b.is_ascii_lowercase()) {
        return false;
    }
    let is_script = |part: &str| {
        part.len() == 4
            && part.as_bytes()[0].is_ascii_uppercase()
            && part.bytes().skip(1).all(|b| b.is_ascii_lowercase())
    };
    let is_region = |part: &str| part.len() == 2 && part.bytes().all(|b| b.is_ascii_uppercase());
    let rest: Vec<&str> = parts.collect();
    match rest.as_slice() {
        [] => true,
        [one] => is_script(one) || is_region(one),
        [script, region] => is_script(script) && is_region(region),
        _ => false,
    }
}

impl TryFrom<String> for Locale {
    type Error = InvalidLocale;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Locale> for String {
    fn from(value: Locale) -> Self {
        value.0
    }
}

impl core::fmt::Display for Locale {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A name's forms in other languages, by locale: at most [`FORMS_MAX`], each a [`Name`].
pub type Forms = BTreeMap<Locale, Name>;

/// A `name:` section as authored: one name, or a default name with its forms in other languages.
///
/// ```yaml
/// name: The Copper Kettle            # the short form
///
/// name:                              # the long form
///   default: The Copper Kettle
///   forms:
///     zh_Hans: 铜壶咖啡馆
/// ```
///
/// A form equal to the default is refused, so a file has one spelling per meaning; so are more than
/// 16 forms, a locale written twice, and any field besides `default` and `forms`. A client never
/// translates a name: it picks the form for its language, else the default (`ARC-70`).
#[derive(Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[serde(untagged)]
#[schemars(example = &"The Copper Kettle")]
pub enum NameSpec {
    /// `name: Alice Moreau`: one name, no other language.
    Short(Name),
    /// A default name and its forms in other languages.
    Long {
        /// The name in the world's own language, shown when no form matches the reader's.
        default: Name,
        /// The name in other languages, by catalog stem (`zh_Hans`, `fr`, `pt_BR`): at most 16, none
        /// equal to `default`. Absent, none.
        #[serde(default)]
        forms: Forms,
    },
}

impl NameSpec {
    /// The default name.
    pub const fn default_name(&self) -> &Name {
        match self {
            Self::Short(name) | Self::Long { default: name, .. } => name,
        }
    }

    /// The forms in other languages; empty for the short form.
    pub fn forms(&self) -> Forms {
        match self {
            Self::Short(_) => Forms::new(),
            Self::Long { forms, .. } => forms.clone(),
        }
    }
}

/// Decoded field by field, so every refusal is raised while the decoder stands at the offending entry
/// and keeps its line and column (`DEP-10`).
impl<'de> Deserialize<'de> for NameSpec {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SpecVisitor)
    }
}

struct SpecVisitor;

impl<'de> Visitor<'de> for SpecVisitor {
    type Value = NameSpec;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("a name, or a map with `default` and optionally `forms`")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<NameSpec, E> {
        Name::new(value).map(NameSpec::Short).map_err(E::custom)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<NameSpec, A::Error> {
        let mut default: Option<Name> = None;
        let mut forms: Option<Forms> = None;
        while let Some(field) = map.next_key::<String>()? {
            match field.as_str() {
                "default" if default.is_none() => default = Some(map.next_value()?),
                "forms" if forms.is_none() => {
                    forms = Some(map.next_value_seed(FormsSeed(default.as_ref()))?);
                }
                "default" | "forms" => return Err(de::Error::duplicate_field("name field")),
                other => return Err(de::Error::unknown_field(other, &["default", "forms"])),
            }
        }
        let default = default.ok_or_else(|| de::Error::missing_field("default"))?;
        let forms = forms.unwrap_or_default();
        if let Some((locale, _)) = forms.iter().find(|(_, form)| **form == default) {
            return Err(de::Error::custom(format!(
                "the {locale} form repeats the default name {default:?}; leave it out"
            )));
        }
        Ok(NameSpec::Long { default, forms })
    }
}

/// Decodes `forms:` entry by entry, refusing a repeated locale, a form equal to an already-read
/// default, and a 17th form at the entry itself.
struct FormsSeed<'a>(Option<&'a Name>);

impl<'de> DeserializeSeed<'de> for FormsSeed<'_> {
    type Value = Forms;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Forms, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for FormsSeed<'_> {
    type Value = Forms;

    fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("a map of locales (such as zh_Hans) to names")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Forms, A::Error> {
        let mut forms = Forms::new();
        while let Some(locale) = map.next_key::<Locale>()? {
            let form: Name = map.next_value()?;
            if forms.contains_key(&locale) {
                return Err(de::Error::custom(format!(
                    "the {locale} form is written twice"
                )));
            }
            if forms.len() == FORMS_MAX {
                return Err(de::Error::custom(format!(
                    "a name may carry at most {FORMS_MAX} forms"
                )));
            }
            if self.0 == Some(&form) {
                return Err(de::Error::custom(format!(
                    "the {locale} form repeats the default name {form:?}; leave it out"
                )));
            }
            forms.insert(locale, form);
        }
        Ok(forms)
    }
}
