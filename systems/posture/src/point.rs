//! A sit point, and the sit points of one place: validated where they are built (SD-a-1).
//!
//! A sit point is data of its place, not an entity (`CORE_CONCEPTS.md` §6.4, `ARC-147`). Its position
//! is free floor in front of the seat, its facing is the direction a seated person faces, and the
//! value knows nothing of height: the pelvis over the seat is presentation.

use mineworld_contracts::{LocalPosition, Millidegrees, Millimetres};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::key::{SitGroupKey, SitPointKey};

/// Most sit points one place holds (`ARC-147`, SD-a-1).
pub const SIT_POINTS_MAX: usize = 64;

/// The nearest two sit points of one place may stand, centre to centre: two seated people never
/// overlap. Posture's own constant (`ARC-147`): `2 × PERSON_RADIUS + GAP` on the 12d values, which
/// posture does not import from bodies.
pub const SIT_POINT_SPACING: Millimetres = Millimetres::new(510);

/// The largest absolute coordinate of a sit point, in millimetres, in either axis.
pub const COORDINATE_BOUND: Millimetres = Millimetres::new(100_000);

/// Why a sit point, or a place's list of them, may not be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InvalidSitPoints {
    /// A coordinate beyond the bound.
    #[error("sit point {key} is at ({x}, {y}) mm; a coordinate must lie within ±{bound} mm")]
    Coordinate {
        /// The point.
        key: SitPointKey,
        /// Its x, in millimetres.
        x: i32,
        /// Its y, in millimetres.
        y: i32,
        /// The bound, in millimetres.
        bound: i32,
    },
    /// More points than a place holds.
    #[error("a place holds at most {max} sit points; this one lists {count}")]
    TooMany {
        /// The most a place holds.
        max: usize,
        /// How many were listed.
        count: usize,
    },
    /// Two points with one key.
    #[error("sit point {key} is listed twice; a key is unique within its place")]
    DuplicateKey {
        /// The key listed twice.
        key: SitPointKey,
    },
    /// Two points closer than the spacing.
    #[error(
        "sit points {first} and {second} stand closer than {spacing} mm centre to centre; \
         two seated people must not overlap"
    )]
    TooClose {
        /// The first of the two, in key order.
        first: SitPointKey,
        /// The second of the two, in key order.
        second: SitPointKey,
        /// The spacing, in millimetres.
        spacing: i32,
    },
}

/// The position of a sit point, as a file writes it: `{ x: 2700, y: 1160 }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Spot {
    /// Millimetres east of the place's origin.
    #[schemars(with = "mineworld_authoring::schema::Length")]
    x: Millimetres,
    /// Millimetres north of the place's origin.
    #[schemars(with = "mineworld_authoring::schema::Length")]
    y: Millimetres,
}

/// The file and wire shape of a [`SitPoint`]: every field named, nothing else accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Written {
    /// The point's name, unique within its place: 1 to 48 bytes of `a`–`z`, `0`–`9`, `-` and `.`.
    key: SitPointKey,
    /// The seated person's position on the place's floor, in millimetres.
    at: Spot,
    /// The yaw a seated person faces, in millidegrees: `0` faces north, `90 000` east, `180 000` south.
    #[schemars(with = "i32")]
    facing: Millidegrees,
    /// The furniture this point belongs to, if any: a slug such as `window-bench`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    group: Option<SitGroupKey>,
}

/// One place where one person may sit, on free floor.
///
/// Serialized as `{ key, at: { x, y }, facing, group? }`, the same shape a place file writes, so an
/// authored section and a stored fact read back through one type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Written", into = "Written")]
#[schemars(with = "Written")]
pub struct SitPoint {
    key: SitPointKey,
    at: LocalPosition,
    facing: Millidegrees,
    group: Option<SitGroupKey>,
}

impl SitPoint {
    /// A sit point at `(x, y)` millimetres, facing `facing`, refused unless both coordinates lie within
    /// [`COORDINATE_BOUND`].
    ///
    /// # Errors
    ///
    /// [`InvalidSitPoints::Coordinate`] when a coordinate lies beyond the bound.
    pub fn new(
        key: SitPointKey,
        x: Millimetres,
        y: Millimetres,
        facing: Millidegrees,
        group: Option<SitGroupKey>,
    ) -> Result<Self, InvalidSitPoints> {
        let bound = COORDINATE_BOUND.value();
        if x.value().abs() > bound || y.value().abs() > bound {
            return Err(InvalidSitPoints::Coordinate {
                key,
                x: x.value(),
                y: y.value(),
                bound,
            });
        }
        Ok(Self {
            key,
            at: LocalPosition::on_ground(x, y),
            facing,
            group,
        })
    }

    /// The point's key.
    pub const fn key(&self) -> &SitPointKey {
        &self.key
    }

    /// Where the seated person stands: free floor, `z` zero.
    pub const fn at(&self) -> LocalPosition {
        self.at
    }

    /// The yaw a seated person faces.
    pub const fn facing(&self) -> Millidegrees {
        self.facing
    }

    /// The furniture the point belongs to, if it names any.
    pub const fn group(&self) -> Option<&SitGroupKey> {
        self.group.as_ref()
    }
}

impl TryFrom<Written> for SitPoint {
    type Error = InvalidSitPoints;

    fn try_from(written: Written) -> Result<Self, Self::Error> {
        let Written {
            key,
            at,
            facing,
            group,
        } = written;
        Self::new(key, at.x, at.y, facing, group)
    }
}

impl From<SitPoint> for Written {
    fn from(point: SitPoint) -> Self {
        let at = point.at;
        Self {
            key: point.key,
            at: Spot {
                x: at.x(),
                y: at.y(),
            },
            facing: point.facing,
            group: point.group,
        }
    }
}

/// The sit points of one place: at most [`SIT_POINTS_MAX`], keys unique, every pair at least
/// [`SIT_POINT_SPACING`] apart, held sorted by key (`ARC-147`, I-a-2).
///
/// Built only through its checks, so an invalid list cannot be held: a file's section, a stored fact
/// and a component all decode through [`TryFrom`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Vec<SitPoint>", into = "Vec<SitPoint>")]
#[schemars(with = "Vec<SitPoint>")]
#[schemars(example = serde_json::json!([
    { "key": "window-bench.1", "group": "window-bench", "at": { "x": 2700, "y": 1160 }, "facing": 180000 },
    { "key": "window-bench.2", "group": "window-bench", "at": { "x": 3300, "y": 1160 }, "facing": 180000 }
]))]
pub struct SitPoints(Vec<SitPoint>);

impl SitPoints {
    /// The points of a place, checked and sorted by key.
    ///
    /// # Errors
    ///
    /// [`InvalidSitPoints`] naming the first problem: too many, a duplicate key, or two points too
    /// close. Coordinates were checked when each point was built.
    pub fn new(mut points: Vec<SitPoint>) -> Result<Self, InvalidSitPoints> {
        if points.len() > SIT_POINTS_MAX {
            return Err(InvalidSitPoints::TooMany {
                max: SIT_POINTS_MAX,
                count: points.len(),
            });
        }
        points.sort_by(|a, b| a.key.cmp(&b.key));
        for pair in points.windows(2) {
            if pair[0].key == pair[1].key {
                return Err(InvalidSitPoints::DuplicateKey {
                    key: pair[0].key.clone(),
                });
            }
        }
        let spacing = SIT_POINT_SPACING.value();
        let squared = i64::from(spacing) * i64::from(spacing);
        for (index, first) in points.iter().enumerate() {
            for second in &points[index + 1..] {
                let dx = i64::from(first.at.x().value()) - i64::from(second.at.x().value());
                let dy = i64::from(first.at.y().value()) - i64::from(second.at.y().value());
                if dx * dx + dy * dy < squared {
                    return Err(InvalidSitPoints::TooClose {
                        first: first.key.clone(),
                        second: second.key.clone(),
                        spacing,
                    });
                }
            }
        }
        Ok(Self(points))
    }

    /// The point with this key, if the place holds one.
    pub fn get(&self, key: &SitPointKey) -> Option<&SitPoint> {
        self.0
            .binary_search_by(|point| point.key.cmp(key))
            .ok()
            .map(|index| &self.0[index])
    }

    /// Every point, in key order.
    pub fn iter(&self) -> impl Iterator<Item = &SitPoint> {
        self.0.iter()
    }

    /// How many points the place holds.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the place holds no points.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<Vec<SitPoint>> for SitPoints {
    type Error = InvalidSitPoints;

    fn try_from(points: Vec<SitPoint>) -> Result<Self, Self::Error> {
        Self::new(points)
    }
}

impl From<SitPoints> for Vec<SitPoint> {
    fn from(points: SitPoints) -> Self {
        points.0
    }
}
