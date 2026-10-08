//! The state this pack owns: a place's shape — its walkable floor and the solids standing on it —,
//! an object's shape, and where the loose objects of a place lie (step-11 SD-O2).

use mineworld_contracts::{ItemId, LocalPosition, Millimetres};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::geometry::{
    Area, COORDINATE_BOUND, GAP, OBJECT_HALF_HEIGHT_MAX, OBJECT_HALF_MAX, OBJECT_HALF_MIN,
    OBJECTS_MAX, PERSON_RADIUS, Point, Room,
};
use crate::system::BodiesSystem;

/// The most solids one place may hold: the resolver builds every one into each scene it builds.
pub const SOLIDS_MAX: usize = 64;

/// The tallest solid, and the shortest: a solid is at least a millimetre high and at most ten metres.
pub const SOLID_HEIGHT_MAX: Millimetres = Millimetres::new(10_000);

/// A corner on a place's floor, in the place's frame: `x` east and `y` north of its origin
/// (`CORE_CONCEPTS.md` §6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Corner {
    x: Millimetres,
    y: Millimetres,
}

impl Corner {
    /// The corner at `(x, y)`.
    pub const fn new(x: Millimetres, y: Millimetres) -> Self {
        Self { x, y }
    }

    /// East of the origin.
    pub const fn x(self) -> Millimetres {
        self.x
    }

    /// North of the origin.
    pub const fn y(self) -> Millimetres {
        self.y
    }

    const fn point(self) -> Point {
        Point::new(self.x.value(), self.y.value())
    }
}

/// The walkable floor: an axis-aligned rectangle from its south-west corner `min` to its north-east
/// corner `max`. Its edge is the place's walls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Floor {
    min: Corner,
    max: Corner,
}

impl Floor {
    /// The south-west corner.
    pub const fn min(&self) -> Corner {
        self.min
    }

    /// The north-east corner.
    pub const fn max(&self) -> Corner {
        self.max
    }
}

/// A solid box standing on the floor — a counter, a table, a pillar: its footprint from `min` to
/// `max`, and its height above the floor. Nobody stands inside one or walks through it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Solid {
    min: Corner,
    max: Corner,
    height: Millimetres,
}

impl Solid {
    /// The footprint's south-west corner.
    pub const fn min(&self) -> Corner {
        self.min
    }

    /// The footprint's north-east corner.
    pub const fn max(&self) -> Corner {
        self.max
    }

    /// How high it stands above the floor.
    pub const fn height(&self) -> Millimetres {
        self.height
    }
}

/// A place's fixed geometry, in its own frame (step-11 SD-B3).
///
/// Authored as the place file's `body:` section, stated at genesis as `place-shaped`, and written
/// only by this pack while reducing that fact. Disclosed to whoever perceives the place, so that a
/// client builds the walls the server resolves against from the server's own numbers (step-11 R-B4).
///
/// Its type is its validation: a value that breaks a rule below cannot be constructed, from YAML, from
/// a fact or from a snapshot.
///
/// ```text
/// floor    each side at least 620 mm (two radii and two gaps: room for one person to turn)
/// solids   at most 64; each footprint non-empty, each height 1 … 10 000 mm
/// every    coordinate within ±100 000 mm
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "Authored")]
pub struct PlaceShape {
    floor: Floor,
    solids: Vec<Solid>,
}

owned_component! {
    component = PlaceShape,
    owner = BodiesSystem,
    component_type = "place-shape",
    schema_version = 1,
}

impl PlaceShape {
    /// The walkable floor.
    pub const fn floor(&self) -> Floor {
        self.floor
    }

    /// The solids, in authored order — the order a scene inserts them in.
    pub fn solids(&self) -> &[Solid] {
        &self.solids
    }

    /// The same geometry as the resolver reads it.
    pub(crate) fn room(&self) -> Room {
        Room {
            floor: Area {
                min: self.floor.min.point(),
                max: self.floor.max.point(),
            },
            solids: self
                .solids
                .iter()
                .map(|solid| {
                    (
                        Area {
                            min: solid.min.point(),
                            max: solid.max.point(),
                        },
                        solid.height.value(),
                    )
                })
                .collect(),
        }
    }
}

/// The shape as written, before its rules are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Authored {
    floor: Floor,
    #[serde(default)]
    solids: Vec<Solid>,
}

impl TryFrom<Authored> for PlaceShape {
    type Error = String;

    fn try_from(authored: Authored) -> Result<Self, String> {
        let Authored { floor, solids } = authored;
        Self::checked(floor, solids)
    }
}

impl PlaceShape {
    /// A shape from its floor and its solids, under the rules above, with the messages an author is
    /// shown — the one check behind both a decoded `PlaceShape` and the `body:` section's place form.
    pub(crate) fn checked(floor: Floor, solids: Vec<Solid>) -> Result<Self, String> {
        let narrowest = 2 * (PERSON_RADIUS.value() + GAP.value());
        let corners = [floor.min, floor.max]
            .into_iter()
            .chain(solids.iter().flat_map(|solid| [solid.min, solid.max]));
        let bound = COORDINATE_BOUND.value();
        if let Some(far) = corners
            .into_iter()
            .find(|corner| corner.x.value().abs() > bound || corner.y.value().abs() > bound)
        {
            return Err(format!(
                "a body coordinate lies beyond ±{bound} mm: ({}, {})",
                far.x.value(),
                far.y.value()
            ));
        }
        let (width, depth) = (
            floor.max.x.value() - floor.min.x.value(),
            floor.max.y.value() - floor.min.y.value(),
        );
        if width < narrowest || depth < narrowest {
            return Err(format!(
                "the floor runs from min to max and each side is at least {narrowest} mm; this one \
                 is {width} mm by {depth} mm"
            ));
        }
        if solids.len() > SOLIDS_MAX {
            return Err(format!(
                "a place holds at most {SOLIDS_MAX} solids; this one has {}",
                solids.len()
            ));
        }
        for (index, solid) in solids.iter().enumerate() {
            if solid.min.x >= solid.max.x || solid.min.y >= solid.max.y {
                return Err(format!(
                    "solid {index} is empty: min must lie south-west of max"
                ));
            }
            if !(1..=SOLID_HEIGHT_MAX.value()).contains(&solid.height.value()) {
                return Err(format!(
                    "solid {index} is {} mm high; a solid is 1 to {} mm high",
                    solid.height.value(),
                    SOLID_HEIGHT_MAX.value()
                ));
            }
        }
        Ok(Self { floor, solids })
    }
}

/// A box's half-extents, in millimetres.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HalfExtents {
    x: Millimetres,
    y: Millimetres,
    z: Millimetres,
}

impl HalfExtents {
    /// Half the box's width (east–west).
    pub const fn x(self) -> Millimetres {
        self.x
    }

    /// Half the box's depth (south–north).
    pub const fn y(self) -> Millimetres {
        self.y
    }

    /// Half the box's height.
    pub const fn z(self) -> Millimetres {
        self.z
    }
}

/// A loose object's shape (step-11 SD-O2), written only by this pack while reducing `body-formed`.
///
/// Its type is its validation: a box's half-extents are 50 … 400 mm in x and y and 50 … 500 mm in z; a
/// ball's radius is 50 … 400 mm. Authored as `{ box: { x, y, z } }` or `{ ball: <radius> }`. Never
/// disclosed alone — items are never perceived (`ARC-37`) — but joined into a place's listing of the
/// objects lying in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", try_from = "AuthoredShape")]
pub enum BodyShape {
    /// A box, axis-aligned with the place's frame (rotations are never persisted).
    Box(HalfExtents),
    /// A ball of this radius.
    Ball(Millimetres),
}

owned_component! {
    component = BodyShape,
    owner = BodiesSystem,
    component_type = "body-shape",
    schema_version = 1,
}

impl BodyShape {
    /// Half its footprint's extent in x and in y (a ball's radius on both axes).
    pub(crate) const fn half_footprint(self) -> (i32, i32) {
        match self {
            Self::Box(half) => (half.x.value(), half.y.value()),
            Self::Ball(radius) => (radius.value(), radius.value()),
        }
    }

    /// How high its centre stands above the surface it rests on.
    pub(crate) const fn half_height(self) -> i32 {
        match self {
            Self::Box(half) => half.z.value(),
            Self::Ball(radius) => radius.value(),
        }
    }
}

/// The shape as written, before its bounds are checked.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum AuthoredShape {
    Box(HalfExtents),
    Ball(Millimetres),
}

impl TryFrom<AuthoredShape> for BodyShape {
    type Error = String;

    fn try_from(authored: AuthoredShape) -> Result<Self, String> {
        let (low, wide, tall) = (
            OBJECT_HALF_MIN.value(),
            OBJECT_HALF_MAX.value(),
            OBJECT_HALF_HEIGHT_MAX.value(),
        );
        match authored {
            AuthoredShape::Box(half) => {
                let (x, y, z) = (half.x.value(), half.y.value(), half.z.value());
                if ![x, y].iter().all(|v| (low..=wide).contains(v)) || !(low..=tall).contains(&z) {
                    return Err(format!(
                        "a box's half-extents are {low} to {wide} mm in x and y and {low} to {tall} \
                         mm in z; this one is x {x}, y {y}, z {z}"
                    ));
                }
                Ok(Self::Box(half))
            }
            AuthoredShape::Ball(radius) => {
                if !(low..=wide).contains(&radius.value()) {
                    return Err(format!(
                        "a ball's radius is {low} to {wide} mm; this one is {} mm",
                        radius.value()
                    ));
                }
                Ok(Self::Ball(radius))
            }
        }
    }
}

/// One loose object lying in a place: which Item, and where its centre is — `x` and `y` on the floor,
/// `z` its centre's height (a box on the floor: its half-height; on a solid: the solid's height plus
/// its half-height).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lying {
    object: ItemId,
    at: LocalPosition,
}

impl Lying {
    /// `object`, lying at `at`.
    pub const fn new(object: ItemId, at: LocalPosition) -> Self {
        Self { object, at }
    }

    /// The object.
    pub const fn object(&self) -> ItemId {
        self.object
    }

    /// Where its centre is.
    pub const fn at(&self) -> LocalPosition {
        self.at
    }
}

/// The loose objects lying in a place, in `ItemId` order — the order a scene inserts them in (step-11
/// SD-O2, QO-2). On the Place rather than on each Item, because perception asks only about a place and
/// the people in it (step-11 F-O1): this row is disclosed with the place, as a listing that joins each
/// object's [`BodyShape`].
///
/// Written only by this pack, while reducing `object-placed` and `object-moved`. Its type is its
/// validation: at most 32 objects, each listed once, in ascending `ItemId` order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "AuthoredObjects")]
pub struct LooseObjects {
    objects: Vec<Lying>,
}

owned_component! {
    component = LooseObjects,
    owner = BodiesSystem,
    component_type = "loose-objects",
    schema_version = 1,
}

impl LooseObjects {
    /// The objects, in `ItemId` order.
    pub fn objects(&self) -> &[Lying] {
        &self.objects
    }

    /// Where `object` lies here, if it does.
    pub fn of(&self, object: ItemId) -> Option<Lying> {
        self.objects
            .iter()
            .copied()
            .find(|lying| lying.object == object)
    }

    /// This row with `lying` added, kept in order — or why it cannot be.
    pub(crate) fn with(&self, lying: Lying) -> Result<Self, String> {
        let mut objects = self.objects.clone();
        objects.push(lying);
        objects.sort_by_key(Lying::object);
        Self::try_from(AuthoredObjects { objects })
    }

    /// This row with `object` moved to `at`.
    pub(crate) fn moved(&self, object: ItemId, at: LocalPosition) -> Self {
        let objects = self
            .objects
            .iter()
            .map(|lying| {
                if lying.object == object {
                    Lying::new(object, at)
                } else {
                    *lying
                }
            })
            .collect();
        Self { objects }
    }
}

/// The row as decoded, before its rules are checked.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredObjects {
    objects: Vec<Lying>,
}

impl TryFrom<AuthoredObjects> for LooseObjects {
    type Error = String;

    fn try_from(authored: AuthoredObjects) -> Result<Self, String> {
        let AuthoredObjects { objects } = authored;
        if objects.len() > OBJECTS_MAX {
            return Err(format!(
                "a place holds at most {OBJECTS_MAX} loose objects; this one would hold {}",
                objects.len()
            ));
        }
        if objects
            .windows(2)
            .any(|pair| pair[0].object >= pair[1].object)
        {
            return Err("loose objects are listed once each, in ascending item order".to_owned());
        }
        Ok(Self { objects })
    }
}
