//! This pack's answer to movement's question: how does a person get from here to there in a shaped
//! place (`DECISIONS.md` `ARC-75`, `DEP-34`, `ARC-39` note 5; step-11 SD-N3 … SD-N5, SD-N9).
//!
//! ```text
//! inert       the place has no shape                                     → None
//! obstacles   every solid's footprint, every loose object's (a ball as its bounding square), and
//!             the person to avoid (a disc of PERSON_RADIUS, as its square) — each grown by
//!             M = PERSON_RADIUS + GAP + PLAN_MARGIN on every side; the floor shrunk by M
//! goal        inside a grown box or outside the shrunk floor → the nearest free point of the 50 mm
//!             lattice (distance, then y, then x: entry's E3 order); none → Unreachable
//! start       not free (in a grown box or outside the shrunk floor) → the same snap: the route first
//!             steps to the nearest free point and is planned from there (§21.15 M-3)
//! fast path   from → goal meets no grown box's open interior               → [goal]
//! graph       nodes: from, goal, then every grown corner strictly inside the shrunk floor and
//!             outside every other grown box (solids, objects, the avoided person; SW, SE, NE, NW);
//!             an edge when its segment meets no grown box's open interior; cost ⌈|d|⌉, heuristic
//!             ⌊|goal − n|⌋; pathfinding's A*, successors in node order, edges decided as a node is
//!             expanded
//! answer      the path without its start — at most WAYPOINTS_MAX — else Unreachable
//! ```
//!
//! A start inside a grown box (somebody standing at the counter, nearer it than M) is snapped to the
//! nearest free point, which lies on the near side of the solid, never across it. Only when the place
//! has no free point at all is the start kept, and then, for edges from the start only, a box holding
//! it is replaced by its core — the box shrunk until the start lies on its edge — so the start can
//! step away from the solid but never through it (§21.15 N-D7). The core rule alone left a start
//! pinned between a wall and a solid's corner with no edge out (M-3: alice at the café's bench).
//!
//! Integers only: every predicate is a cross product in `i128`, every length an `isqrt`. No float, no
//! hash map, nothing kept between calls: the answer is a function of the place's shape, its objects,
//! the person to avoid and the two points (`ARC-25`). The only file that names `pathfinding`.

use mineworld_contracts::{LocalPosition, Millimetres};
use mineworld_kernel::WorldRead;
use mineworld_movement::{RouteAnswer, RouteAsk, Wayfinder, Waypoints};
use pathfinding::directed::astar::astar;

use crate::component::{BodyShape, PlaceShape};
use crate::entry::nearest_free;
use crate::geometry::{Area, GAP, PERSON_RADIUS, Point, Room};
use crate::objects::lying_in;
use crate::system::BodiesSystem;
use mineworld_kernel::SystemIdentity;

/// The clearance a route keeps beyond a body's radius and the controller's gap: one lattice step, so
/// that a routed stride's corridor is clear and the resolver takes its integer fast path (step-11
/// SD-N4, F-N7).
pub const PLAN_MARGIN: Millimetres = Millimetres::new(50);

/// The most waypoints one route may have; a longer one is `unreachable`, never truncated.
pub const WAYPOINTS_MAX: usize = 64;

/// How far every obstacle is grown, and the floor shrunk: 360 mm.
const GROWN: i32 = PERSON_RADIUS.value() + GAP.value() + PLAN_MARGIN.value();
impl Wayfinder for BodiesSystem {
    fn wayfinder_of(&self) -> mineworld_contracts::SystemId {
        Self::ID
    }

    fn route(&self, world: &WorldRead<'_>, ask: &RouteAsk) -> Option<RouteAnswer> {
        let place = ask.place();
        let shape = world.component::<PlaceShape>(place.entity_id())?;
        let objects: Vec<(BodyShape, LocalPosition)> = lying_in(world, place)
            .into_iter()
            .map(|(_, placed)| {
                (
                    placed.shape,
                    LocalPosition::on_ground(
                        Millimetres::new(placed.centre.x),
                        Millimetres::new(placed.centre.y),
                    ),
                )
            })
            .collect();
        let avoid = ask
            .avoid()
            .and_then(|person| {
                world
                    .component::<mineworld_presence::Presence>(person.entity_id())
                    .map(mineworld_presence::Presence::location)
            })
            .filter(|location| location.place() == place)
            .and_then(|location| location.local());
        Some(route_in(shape, &objects, ask.from(), ask.to(), avoid))
    }
}

/// The route this pack plans from `from` to `to` in a place of `shape` holding `objects` (each one's
/// shape and where its centre lies), planning round a person standing at `avoid` — the computation the
/// wayfinder runs, for tests and tools. Waypoints carry `to`'s height.
pub fn route_in(
    shape: &PlaceShape,
    objects: &[(BodyShape, LocalPosition)],
    from: LocalPosition,
    to: LocalPosition,
    avoid: Option<LocalPosition>,
) -> RouteAnswer {
    let room = shape.room();
    let boxes = obstacles(&room, objects, avoid, GROWN);
    let floor = grow(room.floor, -GROWN);
    let height = to.z();
    let Some(goal) = goal(&room, floor, &boxes, point(to)) else {
        return RouteAnswer::Unreachable;
    };
    // A start that is not free — within the margin of the floor's edge or of a grown box, where the
    // resolver may legally have left the walker — first steps to the nearest free point, the goal's
    // own snap (SD-N5; step-11 §21.15 M-3), when that point is within the margin's own width (GROWN)
    // of it: a short step out of the margin, which the resolver resolves like any stride. A start with
    // no free point that near is held where it is, and N-D7's core rule lets it leave the box holding
    // it.
    let start = point(from);
    let snapped = if free(floor, &boxes, start) {
        None
    } else {
        nearest_free(&room, start, &|p| free(floor, &boxes, p))
            .filter(|p| *p != start && length2(start, *p) <= i64::from(GROWN) * i64::from(GROWN))
    };
    let graph = Graph::new(snapped.unwrap_or(start), goal, floor, boxes);
    let Some(mut path) = graph.search() else {
        return RouteAnswer::Unreachable;
    };
    if let Some(first) = snapped
        && path.first() != Some(&first)
    {
        path.insert(0, first);
    }
    if path.len() > WAYPOINTS_MAX {
        return RouteAnswer::Unreachable;
    }
    let waypoints = path
        .into_iter()
        .map(|p| LocalPosition::new(Millimetres::new(p.x), Millimetres::new(p.y), height))
        .collect();
    Waypoints::new(waypoints).map_or(RouteAnswer::Unreachable, RouteAnswer::Waypoints)
}

/// Every obstacle of the plan, each grown by `by` on every side: the place's solids, its loose objects
/// (a ball as its bounding square) and the person to avoid (a square of PERSON_RADIUS).
fn obstacles(
    room: &Room,
    objects: &[(BodyShape, LocalPosition)],
    avoid: Option<LocalPosition>,
    by: i32,
) -> Vec<Area> {
    let square = |centre: Point, hx: i32, hy: i32| Area {
        min: Point::new(centre.x - hx, centre.y - hy),
        max: Point::new(centre.x + hx, centre.y + hy),
    };
    let mut boxes: Vec<Area> = room
        .solids
        .iter()
        .map(|(area, _)| grow(*area, by))
        .collect();
    boxes.extend(objects.iter().map(|(object, at)| {
        let (hx, hy) = object.half_footprint();
        grow(square(point(*at), hx, hy), by)
    }));
    if let Some(at) = avoid {
        let r = PERSON_RADIUS.value();
        boxes.push(grow(square(point(at), r, r), by));
    }
    boxes
}

/// A position's ground point.
fn point(at: LocalPosition) -> Point {
    Point::new(at.x().value(), at.y().value())
}

/// `area` grown by `by` on every side (shrunk when `by` is negative).
const fn grow(area: Area, by: i32) -> Area {
    Area {
        min: Point::new(area.min.x - by, area.min.y - by),
        max: Point::new(area.max.x + by, area.max.y + by),
    }
}

/// Whether `p` lies in the open interior of `area`.
const fn inside(area: &Area, p: Point) -> bool {
    p.x > area.min.x && p.x < area.max.x && p.y > area.min.y && p.y < area.max.y
}

/// Whether `p` is a point a route may end at: in the shrunk floor, edges included, and in no grown
/// box's open interior.
fn free(floor: Area, boxes: &[Area], p: Point) -> bool {
    floor.holds(p, 0) && !boxes.iter().any(|area| inside(area, p))
}

/// Where the route ends: `to` when it is free, else the nearest free point of the 50 mm lattice
/// (step-11 SD-N5), [`None`] when there is none.
fn goal(room: &Room, floor: Area, boxes: &[Area], to: Point) -> Option<Point> {
    if floor.min.x > floor.max.x || floor.min.y > floor.max.y {
        return None;
    }
    if free(floor, boxes, to) {
        return Some(to);
    }
    nearest_free(room, to, &|p| free(floor, boxes, p))
}

/// The cross product of `b − a` and `c − a`: positive when `c` lies left of the line from `a` to `b`.
fn cross(a: Point, b: Point, c: Point) -> i128 {
    let wide = i128::from;
    (wide(b.x) - wide(a.x)) * (wide(c.y) - wide(a.y))
        - (wide(b.y) - wide(a.y)) * (wide(c.x) - wide(a.x))
}

/// Whether the segment `a`–`b` meets the open interior of `area` — exactly, by separating axes: the
/// segment misses the open box when it lies on the closed outer side of one of the box's four edges,
/// or when the box's four corners lie on one closed side of the segment's line. Touching is allowed.
fn blocks(area: &Area, a: Point, b: Point) -> bool {
    if area.min.x >= area.max.x || area.min.y >= area.max.y {
        return false;
    }
    if a.x.max(b.x) <= area.min.x
        || a.x.min(b.x) >= area.max.x
        || a.y.max(b.y) <= area.min.y
        || a.y.min(b.y) >= area.max.y
    {
        return false;
    }
    let sides = corners(area).map(|corner| cross(a, b, corner));
    !(sides.iter().all(|side| *side >= 0) || sides.iter().all(|side| *side <= 0))
}

/// A box's corners: south-west, south-east, north-east, north-west.
const fn corners(area: &Area) -> [Point; 4] {
    [
        Point::new(area.min.x, area.min.y),
        Point::new(area.max.x, area.min.y),
        Point::new(area.max.x, area.max.y),
        Point::new(area.min.x, area.max.y),
    ]
}

/// `area` shrunk until `start`, which lies in its open interior, lies on its edge: the part of the box
/// a start inside it may not cross.
fn core(area: &Area, start: Point) -> Area {
    let depth = (start.x - area.min.x)
        .min(area.max.x - start.x)
        .min(start.y - area.min.y)
        .min(area.max.y - start.y);
    grow(*area, -depth)
}

/// ⌊√v⌋ and ⌈√v⌉ of a squared length.
fn floor_length(v: i64) -> i64 {
    v.isqrt()
}

fn ceil_length(v: i64) -> i64 {
    let root = v.isqrt();
    if root * root == v { root } else { root + 1 }
}

fn length2(a: Point, b: Point) -> i64 {
    let dx = i64::from(a.x) - i64::from(b.x);
    let dy = i64::from(a.y) - i64::from(b.y);
    dx * dx + dy * dy
}

/// One plan's visibility graph: node 0 the start, node 1 the goal, then the usable corners.
struct Graph {
    nodes: Vec<Point>,
    boxes: Vec<Area>,
    /// For edges from the start: each box, or its core when the start lies inside it.
    from_start: Vec<Area>,
}

impl Graph {
    fn new(start: Point, goal: Point, floor: Area, boxes: Vec<Area>) -> Self {
        let mut nodes = vec![start, goal];
        for (index, area) in boxes.iter().enumerate() {
            for corner in corners(area) {
                let in_floor = corner.x > floor.min.x
                    && corner.x < floor.max.x
                    && corner.y > floor.min.y
                    && corner.y < floor.max.y;
                let in_other = boxes
                    .iter()
                    .enumerate()
                    .any(|(other, area)| other != index && inside(area, corner));
                if in_floor && !in_other {
                    nodes.push(corner);
                }
            }
        }
        let from_start = boxes
            .iter()
            .map(|area| {
                if inside(area, start) {
                    core(area, start)
                } else {
                    *area
                }
            })
            .collect();
        Self {
            nodes,
            boxes,
            from_start,
        }
    }

    /// Whether the edge from node `i` to node `j` exists.
    fn visible(&self, i: usize, j: usize) -> bool {
        let (a, b) = (self.nodes[i], self.nodes[j]);
        let boxes = if i == 0 {
            &self.from_start
        } else {
            &self.boxes
        };
        !boxes.iter().any(|area| blocks(area, a, b))
    }

    /// The shortest path's points after the start, or [`None`] when the goal cannot be reached.
    fn search(&self) -> Option<Vec<Point>> {
        let goal = self.nodes[1];
        if self.visible(0, 1) {
            return Some(vec![goal]);
        }
        let (path, _) = astar(
            &0_usize,
            |&i| {
                let from = self.nodes[i];
                (0..self.nodes.len())
                    .filter(move |&j| j != i && j != 0 && self.nodes[j] != from)
                    .filter(move |&j| self.visible(i, j))
                    .map(move |j| (j, ceil_length(length2(from, self.nodes[j]))))
                    .collect::<Vec<_>>()
            },
            |&i| floor_length(length2(self.nodes[i], goal)),
            |&i| i == 1,
        )?;
        Some(path.into_iter().skip(1).map(|i| self.nodes[i]).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(min: (i32, i32), max: (i32, i32)) -> Area {
        Area {
            min: Point::new(min.0, min.1),
            max: Point::new(max.0, max.1),
        }
    }

    /// The edge test against hand-drawn cases: through, along an edge, through a corner, beside.
    #[test]
    fn a_segment_is_blocked_only_by_the_open_interior() {
        let b = area((0, 0), (10, 10));
        let p = Point::new;
        assert!(blocks(&b, p(-5, 5), p(15, 5)), "straight through");
        assert!(blocks(&b, p(2, 2), p(3, 3)), "wholly inside");
        assert!(!blocks(&b, p(-5, 0), p(15, 0)), "along the south edge");
        assert!(
            !blocks(&b, p(-5, 5), p(5, -5)),
            "through the south-west corner only"
        );
        assert!(!blocks(&b, p(-5, 11), p(15, 11)), "beside");
        assert!(
            blocks(&b, p(-5, 6), p(6, -5)),
            "across the corner, inside it"
        );
        assert!(
            !blocks(&b, p(-5, 4), p(4, -5)),
            "past the corner, outside it"
        );
    }

    /// A start inside a box may step away, along or out of it, but not across its core.
    #[test]
    fn a_start_inside_a_grown_box_may_leave_it_but_not_cross_it() {
        let b = area((0, 0), (1_000, 1_000));
        let start = Point::new(100, 500);
        let c = core(&b, start);
        assert_eq!(c, area((100, 100), (900, 900)));
        assert!(!blocks(&c, start, Point::new(-500, 500)), "away, west");
        assert!(!blocks(&c, start, Point::new(100, 2_000)), "along its edge");
        assert!(blocks(&c, start, Point::new(1_500, 500)), "across");
    }

    /// M-3 (E-NW4): the café's front corner as 12d draws it — the bench's seat slab (2 360 … 8 020,
    /// 460 … 900) and the open door leaf (751 … 1 153, 0 … 746) against the front wall — with a person
    /// left by the resolver at (2 194, 260): R + GAP from the wall and from the bench's corner, inside
    /// the planner's margin of both. The walk out first steps to the nearest free point and then goes
    /// on; before the fix every walk from here was Unreachable.
    #[test]
    fn a_start_pinned_in_the_margin_steps_out_first() {
        let shape: PlaceShape = serde_json::from_value(serde_json::json!({
            "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": 8_320, "y": 10_320 } },
            "solids": [
                { "min": { "x": 2_360, "y": 460 }, "max": { "x": 8_020, "y": 900 }, "height": 470 },
                { "min": { "x": 751, "y": 0 }, "max": { "x": 1_153, "y": 746 }, "height": 2_070 },
            ],
        }))
        .expect("a shape");
        let at =
            |x: i32, y: i32| LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y));
        // R + GAP from the front wall, and just over R + GAP from the bench's south-west corner
        // (2 360, 460): at R 250 this is alice's (2 192, 260); at R 300, (2 087, 310).
        let clear = PERSON_RADIUS.value() + GAP.value();
        let rise = 460 - clear;
        let along = i32::try_from((i64::from(clear).pow(2) - i64::from(rise).pow(2)).isqrt())
            .expect("small")
            + 2;
        let start = at(2_360 - along, clear);
        let RouteAnswer::Waypoints(route) = route_in(&shape, &[], start, at(4_000, 5_000), None)
        else {
            panic!("a start in the margin has a way out: {start:?}");
        };
        let first = route.points()[0];
        let room = shape.room();
        let floor = grow(room.floor, -GROWN);
        let boxes: Vec<Area> = room.solids.iter().map(|(a, _)| grow(*a, GROWN)).collect();
        assert!(
            !free(floor, &boxes, point(start)),
            "the start is in the margin"
        );
        assert!(
            free(floor, &boxes, point(first)),
            "the first step ends free: {first:?}"
        );
        assert!(
            length2(point(start), point(first)) <= i64::from(GROWN).pow(2),
            "a short step out of the margin: {start:?} → {first:?}"
        );
        assert_eq!(
            route.points().last(),
            Some(&at(4_000, 5_000)),
            "and the walk arrives"
        );
    }
}
