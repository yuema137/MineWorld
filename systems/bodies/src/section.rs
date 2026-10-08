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

use crate::component::{BodyShape, Floor, PlaceShape, Solid};
use crate::event::{body_formed, place_shaped};
use crate::system::BodiesSystem;

/// Why a form was refused on the kind of file it was found in.
const SECTION_KIND: RejectionCode = RejectionCode::from_static("bodies-section-kind");

/// Where an object lies, as authored: a place by key, and a point on its floor.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lies {
    place: EntityKey,
    x: Millimetres,
    y: Millimetres,
}

/// The `body:` section, decoded into exactly one of its two forms.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "Written")]
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    floor: Option<Floor>,
    solids: Option<Vec<Solid>>,
    shape: Option<BodyShape>,
    at: Option<Lies>,
}

impl TryFrom<Written> for Body {
    type Error = String;

    fn try_from(written: Written) -> Result<Self, String> {
        let Written {
            floor,
            solids,
            shape,
            at,
        } = written;
        let place_keys = floor.is_some() || solids.is_some();
        let object_keys = shape.is_some() || at.is_some();
        match (floor, shape, at) {
            (Some(floor), None, None) => {
                PlaceShape::checked(floor, solids.unwrap_or_default()).map(Self::Place)
            }
            (None, Some(shape), Some(at)) if solids.is_none() => Ok(Self::Object { shape, at }),
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
    ///   solids:
    ///     - { min: { x: 3860, y: 6570 }, max: { x: 8320, y: 7170 }, height: 1100 }
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
