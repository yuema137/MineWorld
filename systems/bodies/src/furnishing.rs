//! A furnishing: a named piece standing on a place's floor that a body can meet — a table, a chair,
//! a shelf unit, a lamp post (`CORE_CONCEPTS.md` §6.5, `ARC-307`).
//!
//! It is semantic place data of this pack, not an Entity and not an Item: it never moves, is never
//! owned or carried, and has no state. The place says what stands where; its solids (each naming the
//! furnishing's key) are what a body meets; its look belongs to a Presentation Pack, chosen by `kind`.

use mineworld_contracts::Millidegrees;
use serde::{Deserialize, Serialize};

use crate::component::Corner;

/// The longest furnishing key or kind, in bytes.
pub const FURNISHING_SLUG_MAX: usize = 48;

/// The most furnishings one place may hold.
pub const FURNISHINGS_MAX: usize = 64;

/// A full turn, in millidegrees: a facing is `0 … FULL_TURN − 1`.
const FULL_TURN: i32 = 360_000;

/// Whether `slug` is 1 to [`FURNISHING_SLUG_MAX`] bytes of `a`–`z`, `0`–`9`, `-` and `.`.
fn valid(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= FURNISHING_SLUG_MAX
        && slug.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'.'
        })
}

/// `slug` itself when it is a legal slug, or the message an author is shown, naming the `role`.
pub(crate) fn checked(role: &str, slug: String) -> Result<String, String> {
    if valid(&slug) {
        Ok(slug)
    } else {
        Err(format!(
            "a {role} is 1 to {FURNISHING_SLUG_MAX} bytes of a-z, 0-9, '-' and '.', but got {slug:?}"
        ))
    }
}

/// The name of a furnishing, unique within its place: `table.east`, `chair.east.1`. It names the
/// piece; it says nothing of what the piece is or looks like.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = "^[a-z0-9.-]+$", "minLength" = 1, "maxLength" = FURNISHING_SLUG_MAX))]
pub struct FurnishingKey(String);

impl FurnishingKey {
    /// A key, refused unless it is a legal slug.
    ///
    /// # Errors
    ///
    /// The message an author is shown.
    pub fn new(key: impl Into<String>) -> Result<Self, String> {
        checked("furnishing key", key.into()).map(Self)
    }

    /// The key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FurnishingKey {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        Self::new(value)
    }
}

impl From<FurnishingKey> for String {
    fn from(value: FurnishingKey) -> Self {
        value.0
    }
}

/// What a furnishing is, as the world's author labels it: `table.round`, `chair.painted`. Never what
/// it looks like. Not a closed list: a client that has no binding for `chair.painted` falls back to
/// `chair`, dropping one `.`-segment at a time.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(try_from = "String", into = "String")]
#[schemars(extend("pattern" = "^[a-z0-9.-]+$", "minLength" = 1, "maxLength" = FURNISHING_SLUG_MAX))]
pub struct FurnishingKind(String);

impl FurnishingKind {
    /// A kind, refused unless it is a legal slug.
    ///
    /// # Errors
    ///
    /// The message an author is shown.
    pub fn new(kind: impl Into<String>) -> Result<Self, String> {
        checked("furnishing kind", kind.into()).map(Self)
    }

    /// The kind.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FurnishingKind {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        Self::new(value)
    }
}

impl From<FurnishingKind> for String {
    fn from(value: FurnishingKind) -> Self {
        value.0
    }
}

/// One furnishing of a place: its key, its kind, the centre of its drawn footprint and the way it
/// faces. Its solids name it by key; a furnishing with no solid cannot be built (`ARC-307` rule 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Furnishing {
    /// The piece's name, unique within its place: 1 to 48 bytes of `a`–`z`, `0`–`9`, `-` and `.`.
    key: FurnishingKey,
    /// What it is, as the author labels it (`table.round`): the same slug grammar. A Presentation
    /// Pack draws it by this label.
    kind: FurnishingKind,
    /// The centre of its drawn footprint, in the place's frame; inside the rectangle that bounds its
    /// own solids, and on the floor.
    at: Corner,
    /// The way its front faces, in millidegrees: `0` north, `90 000` east; `0` to `359 999`.
    #[schemars(with = "i32")]
    facing: Millidegrees,
}

impl Furnishing {
    /// The piece's key.
    pub const fn key(&self) -> &FurnishingKey {
        &self.key
    }

    /// What it is.
    pub const fn kind(&self) -> &FurnishingKind {
        &self.kind
    }

    /// The centre of its drawn footprint.
    pub const fn at(&self) -> Corner {
        self.at
    }

    /// The way its front faces.
    pub const fn facing(&self) -> Millidegrees {
        self.facing
    }

    /// Why this facing may not be held, if it may not.
    pub(crate) fn facing_flaw(&self) -> Option<String> {
        let facing = self.facing.value();
        (!(0..FULL_TURN).contains(&facing)).then(|| {
            format!(
                "furnishing {} faces {facing} millidegrees; a facing is 0 to {}",
                self.key.as_str(),
                FULL_TURN - 1
            )
        })
    }
}
