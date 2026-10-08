//! The state this pack owns: a place's shape — its walkable floor and the solids standing on it.

use mineworld_contracts::Millimetres;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::geometry::{Area, COORDINATE_BOUND, GAP, PERSON_RADIUS, Point, Room};
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
