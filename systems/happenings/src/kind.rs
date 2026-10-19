//! The kind catalog: what may happen in a world, where, at which hours, for how many days, how far
//! ahead it is announced and who hears of it (design §5.1). Content, never code: a world author adds
//! "a book fair" without a rebuild (F-WE-a-4, QWE-3). A kind has no behaviour, so this pack carries its
//! id and never interprets it — as `group-activity` carries an `ActivityKind`.
//!
//! The types are generic over how a place is named — an authored [`EntityKey`] in the file, a
//! [`PlaceId`] once seeded — so the file and the fact share one validated shape.

use std::collections::BTreeMap;

use mineworld_contracts::{EntityKey, PlaceId};
use serde::{Deserialize, Serialize};

/// The longest kind id, in bytes.
pub const KIND_ID_MAX_BYTES: usize = 32;
/// The longest title, in bytes of UTF-8.
pub const TITLE_MAX_BYTES: usize = 64;
/// The most places one kind may happen at.
pub const PLACES_MAX: usize = 16;
/// The most kinds one world may declare.
pub const KINDS_MAX: usize = 64;
/// Minutes in a day: the latest `until`.
pub const MINUTES_PER_DAY: u16 = 1_440;

/// A kind's id: `market-day`, `festival`. The identifier rule of `contracts` (lowercase ASCII
/// letters, digits, `-` and `_`, neither first nor last a separator), at most [`KIND_ID_MAX_BYTES`].
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
pub struct HappeningKindId(String);

impl HappeningKindId {
    /// The id, if it obeys the rule.
    ///
    /// # Errors
    ///
    /// A message naming the id and the rule.
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        let legal = |byte: u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        let separator = |byte: u8| byte == b'-' || byte == b'_';
        let bytes = id.as_bytes();
        let valid = !bytes.is_empty()
            && bytes.len() <= KIND_ID_MAX_BYTES
            && bytes.iter().all(|byte| legal(*byte) || separator(*byte))
            && !separator(bytes[0])
            && !separator(bytes[bytes.len() - 1]);
        if valid {
            Ok(Self(id))
        } else {
            Err(format!(
                "a kind id is 1 to {KIND_ID_MAX_BYTES} bytes of a-z, 0-9, '-' and '_', not \
                 beginning or ending with '-' or '_', not '{id}'"
            ))
        }
    }

    /// The id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for HappeningKindId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<HappeningKindId> for String {
    fn from(value: HappeningKindId) -> Self {
        value.0
    }
}

impl core::fmt::Display for HappeningKindId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a kind is called when it is shown: at most [`TITLE_MAX_BYTES`] of UTF-8 with no control
/// character. Shown, never parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Title(String);

impl Title {
    /// The title as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Title {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.is_empty() || text.len() > TITLE_MAX_BYTES || text.chars().any(char::is_control) {
            return Err(format!(
                "a title is 1 to {TITLE_MAX_BYTES} bytes with no control character, not {text:?}"
            ));
        }
        Ok(Self(text))
    }
}

impl From<Title> for String {
    fn from(value: Title) -> Self {
        value.0
    }
}

/// The local hours a happening runs on each of its days, in minutes since midnight: `from` before
/// `until`, both on one day; `24:00` is a legal `until`. Written `HH:MM-HH:MM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Hours {
    from: u16,
    until: u16,
}

impl Hours {
    /// Minutes after midnight it begins.
    pub const fn from(self) -> u16 {
        self.from
    }

    /// Minutes after midnight it ends, at most 1 440.
    pub const fn until(self) -> u16 {
        self.until
    }
}

/// `HH:MM` as minutes, 00:00 … 24:00.
fn minute(text: &str) -> Option<u16> {
    let (hours, minutes) = text.split_once(':')?;
    if hours.len() != 2 || minutes.len() != 2 {
        return None;
    }
    let (hours, minutes) = (hours.parse::<u16>().ok()?, minutes.parse::<u16>().ok()?);
    let total = hours.checked_mul(60)?.checked_add(minutes)?;
    (minutes < 60 && total <= MINUTES_PER_DAY).then_some(total)
}

impl TryFrom<String> for Hours {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let refused = || {
            format!(
                "hours are HH:MM-HH:MM on one day, from before until, until at most 24:00, such \
                 as \"08:00-14:00\", not '{text}'"
            )
        };
        let (from, until) = text.split_once('-').ok_or_else(refused)?;
        let (Some(from), Some(until)) = (minute(from), minute(until)) else {
            return Err(refused());
        };
        if from >= until {
            return Err(refused());
        }
        Ok(Self { from, until })
    }
}

impl From<Hours> for String {
    fn from(value: Hours) -> Self {
        let clock = |minutes: u16| format!("{:02}:{:02}", minutes / 60, minutes % 60);
        format!("{}-{}", clock(value.from), clock(value.until))
    }
}

/// An integer in `MIN ..= MAX`, refused otherwise with a message naming `what`.
macro_rules! bounded {
    ($(#[$doc:meta])* $name:ident: $int:ty = $text:literal, $min:expr, $max:expr, $default:expr, $what:literal) => {
        $(#[$doc])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
            schemars::JsonSchema,
        )]
        #[serde(try_from = $text, into = $text)]
        pub struct $name($int);

        impl $name {
            /// The value.
            pub const fn get(self) -> $int {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self($default)
            }
        }

        impl TryFrom<$int> for $name {
            type Error = String;

            fn try_from(value: $int) -> Result<Self, Self::Error> {
                if ($min..=$max).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(format!(concat!($what, " is {} to {}, not {}"), $min, $max, value))
                }
            }
        }

        impl From<$name> for $int {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}
pub(crate) use bounded;

bounded!(
    /// How many days a happening runs: from day 1's `from` to day N's `until`.
    Days: u8 = "u8", 1, 14, 1, "days"
);
bounded!(
    /// How many days before it begins a happening is announced; 0 announces it at its own midnight.
    AnnounceDays: u8 = "u8", 0, 30, 0, "announce_days"
);

/// Who hears that a happening began and ended: everybody, or those at its place.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum Audience {
    /// Heard everywhere.
    #[default]
    Public,
    /// Heard at its place.
    Place,
}

/// One kind: its title, the places it may happen at, its hours, its length, how far ahead it is
/// announced, and its audience. `P` is how a place is named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningKind<P> {
    title: Option<Title>,
    places: Vec<P>,
    hours: Hours,
    days: Days,
    announce_days: AnnounceDays,
    audience: Audience,
}

impl<P: Ord + Clone> HappeningKind<P> {
    /// The title, if it has one.
    pub const fn title(&self) -> Option<&Title> {
        self.title.as_ref()
    }

    /// The places it may happen at, in ascending order, each once.
    pub fn places(&self) -> &[P] {
        &self.places
    }

    /// Whether it may happen at `place`.
    pub fn admits(&self, place: &P) -> bool {
        self.places.binary_search(place).is_ok()
    }

    /// Its hours on each day.
    pub const fn hours(&self) -> Hours {
        self.hours
    }

    /// How many days it runs.
    pub const fn days(&self) -> Days {
        self.days
    }

    /// How many days ahead it is announced.
    pub const fn announce_days(&self) -> AnnounceDays {
        self.announce_days
    }

    /// Who hears of it beginning and ending.
    pub const fn audience(&self) -> Audience {
        self.audience
    }

    /// The same kind with its places renamed by `rename`, which may refuse.
    pub(crate) fn map_places<Q: Ord, E>(
        &self,
        mut rename: impl FnMut(&P) -> Result<Q, E>,
    ) -> Result<HappeningKind<Q>, E> {
        let mut places = self
            .places
            .iter()
            .map(&mut rename)
            .collect::<Result<Vec<Q>, E>>()?;
        places.sort();
        Ok(HappeningKind {
            title: self.title.clone(),
            places,
            hours: self.hours,
            days: self.days,
            announce_days: self.announce_days,
            audience: self.audience,
        })
    }
}

/// The kind catalog: 1 … [`KINDS_MAX`] kinds by id. Written as a list in the file and kept as a map in
/// the fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KindCatalog<P = PlaceId>(BTreeMap<HappeningKindId, HappeningKind<P>>);

impl<P: Ord + Clone> KindCatalog<P> {
    /// The kind with this id, if the catalog has it.
    pub fn get(&self, id: &HappeningKindId) -> Option<&HappeningKind<P>> {
        self.0.get(id)
    }

    /// Every kind, in id order.
    pub fn iter(&self) -> impl Iterator<Item = (&HappeningKindId, &HappeningKind<P>)> {
        self.0.iter()
    }

    /// The same catalog with its places renamed by `rename`, which may refuse.
    pub(crate) fn map_places<Q: Ord, E>(
        &self,
        mut rename: impl FnMut(&P) -> Result<Q, E>,
    ) -> Result<KindCatalog<Q>, E> {
        self.0
            .iter()
            .map(|(id, kind)| Ok((id.clone(), kind.map_places(&mut rename)?)))
            .collect::<Result<BTreeMap<_, _>, E>>()
            .map(KindCatalog)
    }
}

/// One kind as the file writes it: the catalog entry and its id.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawKind {
    /// The kind's id, such as market-day: 1 to 32 bytes of a-z, 0-9, - and _.
    id: HappeningKindId,
    /// What the kind is called when shown, at most 64 bytes.
    #[serde(default)]
    title: Option<Title>,
    /// The 1 to 16 distinct places it may happen at, by key.
    #[schemars(with = "Vec<mineworld_authoring::schema::Key>")]
    places: Vec<EntityKey>,
    /// The local hours it runs on each of its days, HH:MM-HH:MM, such as "08:00-14:00".
    hours: Hours,
    /// How many days it runs, 1 to 14; 1 when unsaid.
    #[serde(default)]
    days: Days,
    /// How many days before it begins it is announced, 0 to 30; 0 when unsaid.
    #[serde(default)]
    announce_days: AnnounceDays,
    /// Who hears that it began and ended: public (everybody, the default) or place (those there).
    #[serde(default)]
    audience: Audience,
}

impl KindCatalog<EntityKey> {
    /// The catalog of the file's list: 1 … [`KINDS_MAX`] kinds, ids unique, each with 1 …
    /// [`PLACES_MAX`] distinct places.
    pub(crate) fn from_list(list: Vec<RawKind>) -> Result<Self, String> {
        if list.is_empty() || list.len() > KINDS_MAX {
            return Err(format!(
                "kinds lists 1 to {KINDS_MAX} kinds, not {}",
                list.len()
            ));
        }
        let mut kinds = BTreeMap::new();
        for (index, raw) in list.into_iter().enumerate() {
            let mut places = raw.places;
            places.sort();
            let count = places.len();
            places.dedup();
            if count == 0 || count > PLACES_MAX || places.len() != count {
                return Err(format!(
                    "kinds[{index}] ('{}'): places lists 1 to {PLACES_MAX} distinct places",
                    raw.id
                ));
            }
            let kind = HappeningKind {
                title: raw.title,
                places,
                hours: raw.hours,
                days: raw.days,
                announce_days: raw.announce_days,
                audience: raw.audience,
            };
            if kinds.insert(raw.id.clone(), kind).is_some() {
                return Err(format!(
                    "kinds[{index}]: the kind '{}' is declared twice",
                    raw.id
                ));
            }
        }
        Ok(Self(kinds))
    }
}
