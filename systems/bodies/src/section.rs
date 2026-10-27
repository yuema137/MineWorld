//! The `body:` section, which this pack owns (`ARC-31`; step-11 SD-B3, SD-O4): one type with two
//! forms, one for each kind of file that may carry it.
//!
//! ```text
//! place form    floor (required), solids (optional)    a place's walls and furniture   place-shaped
//! object form   shape and at (both required)          one loose object (ARC-36 note)  body-formed
//! ```
//!
//! Mixing the forms, or neither, is refused by the loader at the section's line, naming the keys; a
//! form on the other kind of file is refused when seeded, with `bodies-section-kind`.

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, ItemId, LocalPosition, Location, Millimetres, PlaceId,
    Rejection, RejectionCode,
};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::component::{BodyShape, Corner, Floor, PlaceShape, Solid};
use crate::event::{body_formed, place_shaped};
use crate::furnishing::{self, Furnishing, FurnishingKey};
use crate::system::BodiesSystem;

/// Why a form was refused on the kind of file it was found in.
const SECTION_KIND: RejectionCode = RejectionCode::from_static("bodies-section-kind");

/// Why a place's body was refused for two solids of different things standing in each other
/// (`ARC-332`). It leads the author's message, since a decoding error carries no code of its own.
const SOLIDS_OVERLAP: &str = "bodies-solids-overlap";

/// The name an author gives the solids of one thing (`ARC-332`): `terrace-table.1`, `table.east`. The
/// slug grammar of a furnishing key, and of `posture`'s sit group: 1 to 48 bytes of `a`–`z`, `0`–`9`,
/// `-` and `.`. Authoring data only: a [`PlaceShape`] does not store it, so nothing is disclosed and
/// no digest depends on it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String")]
#[schemars(extend("pattern" = "^[a-z0-9.-]+$", "minLength" = 1, "maxLength" = furnishing::FURNISHING_SLUG_MAX))]
struct SolidGroup(String);

impl TryFrom<String> for SolidGroup {
    type Error = String;

    fn try_from(value: String) -> Result<Self, String> {
        furnishing::checked("solid group", value).map(Self)
    }
}

/// A solid as a place file writes it: its footprint, its height, the furnishing it belongs to, and
/// the `group` it shares with the other solids of the same thing.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct AuthoredSolid {
    /// The footprint's south-west corner.
    min: Corner,
    /// The footprint's north-east corner; the footprint is not empty.
    max: Corner,
    /// How high it stands above the floor, in millimetres: 1 to 10000.
    #[schemars(with = "mineworld_authoring::schema::Length")]
    height: Millimetres,
    /// The furnishing this solid belongs to, if any: the key of one of the place's `furnishings`.
    #[serde(default)]
    furnishing: Option<FurnishingKey>,
    /// The thing this solid is part of, when another solid of the same thing overlaps it — a chair
    /// tucked under its table, two crossed boxes holding one round table. Two solids whose footprints
    /// share any area must share a `group` or name one `furnishing`; otherwise the place is refused
    /// (`bodies-solids-overlap`). Authoring data only: never stored or disclosed.
    #[serde(default)]
    group: Option<SolidGroup>,
}

impl AuthoredSolid {
    /// Whether `self` and `other` are parts of one thing: the same `group`, or the same `furnishing`.
    fn one_thing_with(&self, other: &Self) -> bool {
        let same_group = self.group.is_some() && self.group == other.group;
        let same_piece = self.furnishing.is_some() && self.furnishing == other.furnishing;
        same_group || same_piece
    }

    /// How far the two footprints overlap, east–west and north–south, in millimetres; `None` when
    /// they share no area (touching along an edge or at a corner shares none).
    fn overlap(&self, other: &Self) -> Option<(i32, i32)> {
        let wide = self.max.x().value().min(other.max.x().value())
            - self.min.x().value().max(other.min.x().value());
        let deep = self.max.y().value().min(other.max.y().value())
            - self.min.y().value().max(other.min.y().value());
        (wide > 0 && deep > 0).then_some((wide, deep))
    }

    fn into_solid(self) -> Solid {
        Solid::authored(self.min, self.max, self.height, self.furnishing)
    }
}

/// The rule of `ARC-332`: no two solids of different things overlap. The first offending pair, in
/// authored order, is named by index (from 0) with its overlap and the remedy.
fn apart(solids: &[AuthoredSolid]) -> Result<(), String> {
    for (i, first) in solids.iter().enumerate() {
        for (j, second) in solids.iter().enumerate().skip(i + 1) {
            if let Some((wide, deep)) = first.overlap(second)
                && !first.one_thing_with(second)
            {
                return Err(format!(
                    "{SOLIDS_OVERLAP}: solids {i} and {j} (from 0, in the order written) overlap by \
                     {wide} mm x {deep} mm; solids of different things may not overlap, and solids \
                     of one thing share a `group` (or name one `furnishing`)"
                ));
            }
        }
    }
    Ok(())
}

/// Where an object lies, as authored: a place by key, and a point on its floor.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Lies {
    /// The place it lies in: a place key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    place: EntityKey,
    /// Millimetres east of the place's origin.
    #[schemars(with = "mineworld_authoring::schema::Length")]
    x: Millimetres,
    /// Millimetres north of the place's origin.
    #[schemars(with = "mineworld_authoring::schema::Length")]
    y: Millimetres,
}

/// A body: in a place file, the walkable floor and the solids standing on it (floor, solids); in an
/// item file, one loose object (shape, at). Never both forms.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Written")]
#[schemars(with = "Written", example = serde_json::json!({
    "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": 4000, "y": 3000 } },
    "furnishings": [{ "key": "table.1", "kind": "table.round", "at": { "x": 3000, "y": 1500 }, "facing": 0 }],
    "solids": [
        { "min": { "x": 0, "y": 2400 }, "max": { "x": 2000, "y": 3000 }, "height": 900 },
        { "min": { "x": 2600, "y": 1100 }, "max": { "x": 3400, "y": 1900 }, "height": 720, "furnishing": "table.1" }
    ]
}))]
pub enum Body {
    /// A place's floor and solids.
    Place(PlaceShape),
    /// One loose object: its shape, and where it lies.
    Object {
        /// The object's shape.
        shape: BodyShape,
        /// The place and the point it lies at.
        at: Lies,
    },
}

/// The four keys as written, before the form is decided.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Written {
    /// A place's walkable floor: an axis-aligned rectangle whose edge is the place's walls. Required in
    /// a place's body.
    floor: Option<Floor>,
    /// The solid boxes standing on a place's floor — counters, tables, pillars: at most 64. Nobody
    /// stands inside one or walks through it. Two whose footprints share any area must be parts of
    /// one thing: the same `group`, or the same `furnishing` (`ARC-332`).
    solids: Option<Vec<AuthoredSolid>>,
    /// The named pieces standing on a place's floor that a body can meet — tables, chairs, shelf
    /// units: at most 64 (`ARC-307`). Each is named by at least one solid's `furnishing`; a
    /// Presentation Pack draws it by its `kind`.
    furnishings: Option<Vec<Furnishing>>,
    /// An object's shape. Required, with `at`, in an item's body.
    shape: Option<BodyShape>,
    /// Where an object lies: a place and a point on its floor. Required, with `shape`, in an item's
    /// body.
    at: Option<Lies>,
}

impl TryFrom<Written> for Body {
    type Error = String;

    fn try_from(written: Written) -> Result<Self, String> {
        let Written {
            floor,
            solids,
            furnishings,
            shape,
            at,
        } = written;
        let place_keys = floor.is_some() || solids.is_some() || furnishings.is_some();
        let object_keys = shape.is_some() || at.is_some();
        match (floor, shape, at) {
            (Some(floor), None, None) => {
                let solids = solids.unwrap_or_default();
                let apart = apart(&solids);
                let shape = PlaceShape::checked(
                    floor,
                    solids.into_iter().map(AuthoredSolid::into_solid).collect(),
                    furnishings.unwrap_or_default(),
                )?;
                apart.map(|()| Self::Place(shape))
            }
            (None, Some(shape), Some(at)) if solids.is_none() && furnishings.is_none() => {
                Ok(Self::Object { shape, at })
            }
            _ if place_keys && object_keys => Err(
                "a body is a place's (floor, solids) or an object's (shape, at), never both"
                    .to_owned(),
            ),
            _ if object_keys => {
                Err("an object's body needs both `shape` and `at`".to_owned())
            }
            _ if place_keys => Err("a place's body needs `floor`".to_owned()),
            _ => Err(
                "a body is a place's (floor, solids) or an object's (shape, at); this one has neither"
                    .to_owned(),
            ),
        }
    }
}

/// `bodies-section-kind`, naming the form and the kind of file it belongs in.
fn wrong_kind(form: &str, belongs: &str) -> Rejection {
    Rejection::System {
        code: SECTION_KIND,
        detail: Some(format!(
            "the {form} form of `body` belongs in {belongs} file"
        )),
    }
}

impl AuthoredSection for BodiesSystem {
    /// ```yaml
    /// body:            # in a place file
    ///   floor: { min: { x: 0, y: 0 }, max: { x: 8320, y: 10320 } }
    ///   furnishings:   # optional (ARC-307): what stands here, drawn by its kind
    ///     - { key: table.east, kind: table.round, at: { x: 4860, y: 3160 }, facing: 0 }
    ///   solids:
    ///     - { min: { x: 3860, y: 6570 }, max: { x: 8320, y: 7170 }, height: 1100 }
    ///     - { min: { x: 4443, y: 2743 }, max: { x: 5277, y: 3577 }, height: 716, furnishing: table.east }
    ///
    /// body:            # in an item file: one loose object
    ///   shape: { ball: 110 }
    ///   at: { place: hall, x: 3000, y: 2000 }
    /// ```
    const SECTION: SectionName = SectionName::from_static("body");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Place, ContentKind::Item];

    /// The section in one of its two forms: decoding it is [`Body`]'s check.
    type Authored = Body;

    /// The object form names the place it lies in, which must be a place.
    fn references(authored: &Body) -> Vec<Reference<'_>> {
        match authored {
            Body::Place(_) => Vec::new(),
            Body::Object { at, .. } => vec![Reference {
                key: &at.place,
                entity_type: EntityType::Place,
            }],
        }
    }

    /// The place form on a place: one `place-shaped`. The object form on an item: one `body-formed`.
    /// A form on the other kind of file: refused, `bodies-section-kind`.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Body,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        match (authored, entity_type) {
            (Body::Place(shape), EntityType::Place) => {
                let place = PlaceId::new(subject, entity_type)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![place_shaped(place, shape.clone())])
            }
            (Body::Object { shape, at }, EntityType::Item) => {
                let object =
                    ItemId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
                let place = seeding
                    .resolve(&at.place, EntityType::Place)
                    .and_then(|place| PlaceId::new(place, EntityType::Place).ok())
                    .ok_or(Rejection::PreconditionFailed)?;
                let lies =
                    Location::in_place(place).with_local(LocalPosition::on_ground(at.x, at.y));
                Ok(vec![body_formed(object, *shape, lies)])
            }
            (Body::Place(_), _) => Err(wrong_kind("place (floor, solids)", "a place")),
            (Body::Object { .. }, _) => Err(wrong_kind("object (shape, at)", "an item")),
        }
    }
}
