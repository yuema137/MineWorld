//! NV-5 — the planner's invariant, by an independent oracle (step-11 §21.8; `DECISIONS.md` `DEP-34`).
//!
//! 2 000 seeded random scenes: a floor of 3–14 m a side, up to 64 solids, up to 32 loose objects
//! (boxes and balls), sometimes a person to plan round. For each, a start the planner may begin at and
//! a goal anywhere on the floor. Then:
//!
//! ```text
//! a route         every point of every leg, sampled every 10 mm, at least R + GAP (310 mm) from every
//!                 solid and every object's footprint, 2R + GAP (610 mm) from the avoided person, and
//!                 inside the floor shrunk by R + GAP; at most 64 waypoints; its end the goal, or —
//!                 when the goal is not free — the oracle's own nearest free lattice point
//! unreachable     confirmed: no free point to end at, or a flood fill on a 25 mm grid from the start
//!                 does not reach the end point
//! ```
//!
//! The oracle shares no code with `route.rs`: distances are brute force on the original shapes, and
//! the flood fill is its own. The one model it shares is the planner's specification — obstacles grown
//! by 360 mm (R + GAP + PLAN_MARGIN), squares for balls — which the flood fill needs to know what
//! "free" means; the clearance check does not use it.

use mineworld_bodies::{BodyShape, PlaceShape, route_in};
use mineworld_contracts::{LocalPosition, Millimetres};
use mineworld_movement::RouteAnswer;

const R_GAP: i64 = 310;
const GROWN: i32 = 360;
const SCENES: usize = 2_000;

/// A deterministic generator (a 64-bit LCG), so that the scenes are the same on every run.
struct Cases(u64);

impl Cases {
    fn next(&mut self, below: i32) -> i32 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        i32::try_from((self.0 >> 33) % u64::from(below.unsigned_abs())).expect("small")
    }
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
}

impl Rect {
    /// Squared Euclidean distance from (x, y) to the closed rectangle.
    fn distance2(&self, x: i64, y: i64) -> i64 {
        let dx = (i64::from(self.x0) - x).max(x - i64::from(self.x1)).max(0);
        let dy = (i64::from(self.y0) - y).max(y - i64::from(self.y1)).max(0);
        dx * dx + dy * dy
    }

    fn grown(&self, by: i32) -> Self {
        Self {
            x0: self.x0 - by,
            y0: self.y0 - by,
            x1: self.x1 + by,
            y1: self.y1 + by,
        }
    }

    fn holds_open(&self, x: i64, y: i64) -> bool {
        x > i64::from(self.x0)
            && x < i64::from(self.x1)
            && y > i64::from(self.y0)
            && y < i64::from(self.y1)
    }
}

enum Footprint {
    Rect(Rect),
    Disc { x: i32, y: i32, r: i32 },
}

impl Footprint {
    fn clear(&self, x: i64, y: i64, reach: i64) -> bool {
        match self {
            Self::Rect(rect) => rect.distance2(x, y) >= reach * reach,
            Self::Disc { x: cx, y: cy, r } => {
                let (dx, dy) = (x - i64::from(*cx), y - i64::from(*cy));
                let edge = reach + i64::from(*r);
                dx * dx + dy * dy >= edge * edge
            }
        }
    }

    fn square(&self) -> Rect {
        match self {
            Self::Rect(rect) => *rect,
            Self::Disc { x, y, r } => Rect {
                x0: x - r,
                y0: y - r,
                x1: x + r,
                y1: y + r,
            },
        }
    }
}

struct Scene {
    floor: Rect,
    solids: Vec<Rect>,
    objects: Vec<Footprint>,
    avoid: Option<(i32, i32)>,
}

impl Scene {
    fn generate(cases: &mut Cases) -> Self {
        let (w, h) = (3_000 + cases.next(11_001), 3_000 + cases.next(11_001));
        let floor = Rect {
            x0: 0,
            y0: 0,
            x1: w,
            y1: h,
        };
        let solids = (0..cases.next(65))
            .map(|_| {
                let (sw, sh) = (100 + cases.next(2_400), 100 + cases.next(2_400));
                let (x, y) = (cases.next(w), cases.next(h));
                Rect {
                    x0: x,
                    y0: y,
                    x1: (x + sw).min(w),
                    y1: (y + sh).min(h),
                }
            })
            .filter(|rect| rect.x0 < rect.x1 && rect.y0 < rect.y1)
            .collect();
        let objects = (0..cases.next(33))
            .map(|_| {
                let (x, y) = (400 + cases.next(w - 800), 400 + cases.next(h - 800));
                if cases.next(2) == 0 {
                    Footprint::Disc {
                        x,
                        y,
                        r: 50 + cases.next(351),
                    }
                } else {
                    let (hx, hy) = (50 + cases.next(351), 50 + cases.next(351));
                    Footprint::Rect(Rect {
                        x0: x - hx,
                        y0: y - hy,
                        x1: x + hx,
                        y1: y + hy,
                    })
                }
            })
            .collect();
        let avoid = (cases.next(4) == 0).then(|| (cases.next(w), cases.next(h)));
        Self {
            floor,
            solids,
            objects,
            avoid,
        }
    }

    fn shape(&self) -> PlaceShape {
        let solids: Vec<_> = self
            .solids
            .iter()
            .map(|s| {
                serde_json::json!({ "min": { "x": s.x0, "y": s.y0 }, "max": { "x": s.x1, "y": s.y1 },
                                    "height": 1_000 })
            })
            .collect();
        serde_json::from_value(serde_json::json!({
            "floor": { "min": { "x": 0, "y": 0 }, "max": { "x": self.floor.x1, "y": self.floor.y1 } },
            "solids": solids,
        }))
        .expect("a valid shape")
    }

    fn bodies(&self) -> Vec<(BodyShape, LocalPosition)> {
        self.objects
            .iter()
            .map(|object| {
                let (shape, x, y) = match object {
                    Footprint::Disc { x, y, r } => (serde_json::json!({ "ball": r }), *x, *y),
                    Footprint::Rect(rect) => (
                        serde_json::json!({ "box": { "x": (rect.x1 - rect.x0) / 2,
                                                     "y": (rect.y1 - rect.y0) / 2, "z": 100 } }),
                        (rect.x0 + rect.x1) / 2,
                        (rect.y0 + rect.y1) / 2,
                    ),
                };
                (serde_json::from_value(shape).expect("a body"), ground(x, y))
            })
            .collect()
    }

    /// The planner's notion of "a centre may be here": in the floor shrunk by 360 mm and outside every
    /// obstacle grown by 360 mm (open interiors).
    fn grown(&self) -> Vec<Rect> {
        let mut grown: Vec<Rect> = self.solids.iter().map(|s| s.grown(GROWN)).collect();
        grown.extend(self.objects.iter().map(|o| o.square().grown(GROWN)));
        if let Some((x, y)) = self.avoid {
            grown.push(
                Rect {
                    x0: x - 300,
                    y0: y - 300,
                    x1: x + 300,
                    y1: y + 300,
                }
                .grown(GROWN),
            );
        }
        grown
    }

    fn free(&self, grown: &[Rect], x: i64, y: i64) -> bool {
        let g = i64::from(GROWN);
        x >= g
            && y >= g
            && x <= i64::from(self.floor.x1) - g
            && y <= i64::from(self.floor.y1) - g
            && !grown.iter().any(|rect| rect.holds_open(x, y))
    }

    /// Every sampled point keeps its clearance; the first offence, if any.
    fn clearance_offence(&self, x: i64, y: i64) -> Option<String> {
        let floor_ok = x >= R_GAP
            && y >= R_GAP
            && x <= i64::from(self.floor.x1) - R_GAP
            && y <= i64::from(self.floor.y1) - R_GAP;
        if !floor_ok {
            return Some(format!("({x}, {y}) outside the floor shrunk by 310"));
        }
        if let Some(solid) = self
            .solids
            .iter()
            .find(|s| s.distance2(x, y) < R_GAP * R_GAP)
        {
            return Some(format!("({x}, {y}) within 310 of solid {solid:?}"));
        }
        if self.objects.iter().any(|o| !o.clear(x, y, R_GAP)) {
            return Some(format!("({x}, {y}) within 310 of an object"));
        }
        if let Some((ax, ay)) = self.avoid {
            let (dx, dy) = (x - i64::from(ax), y - i64::from(ay));
            if dx * dx + dy * dy < 610 * 610 {
                return Some(format!("({x}, {y}) within 610 of the avoided person"));
            }
        }
        None
    }

    /// The oracle's nearest free point of the 50 mm lattice anchored at the floor's corner plus 300 mm,
    /// by distance, then y, then x: brute force.
    fn nearest_free(&self, grown: &[Rect], tx: i64, ty: i64) -> Option<(i64, i64)> {
        let mut best: Option<(i64, i64, i64)> = None;
        let mut y = 300;
        while y <= i64::from(self.floor.y1) - 300 {
            let mut x = 300;
            while x <= i64::from(self.floor.x1) - 300 {
                if self.free(grown, x, y) {
                    let key = ((x - tx).pow(2) + (y - ty).pow(2), y, x);
                    if best.is_none_or(|kept| key < kept) {
                        best = Some(key);
                    }
                }
                x += 50;
            }
            y += 50;
        }
        best.map(|(_, y, x)| (x, y))
    }

    /// Whether a 4-connected flood over the free nodes of a 25 mm grid (anchored like the lattice)
    /// reaches `to` from the node nearest `from`.
    fn flood_reaches(&self, grown: &[Rect], from: (i64, i64), to: (i64, i64)) -> bool {
        let (cols, rows) = (
            usize::try_from(i64::from(self.floor.x1) / 25 + 1).expect("small"),
            usize::try_from(i64::from(self.floor.y1) / 25 + 1).expect("small"),
        );
        let origin = 300 % 25;
        let node = |i: usize, j: usize| {
            (
                origin + 25 * i64::try_from(i).expect("small"),
                origin + 25 * i64::try_from(j).expect("small"),
            )
        };
        let mut free = vec![false; cols * rows];
        for j in 0..rows {
            for i in 0..cols {
                let (x, y) = node(i, j);
                free[j * cols + i] = self.free(grown, x, y);
            }
        }
        let index = |(x, y): (i64, i64)| {
            let i = usize::try_from((x - origin + 12).div_euclid(25)).ok()?;
            let j = usize::try_from((y - origin + 12).div_euclid(25)).ok()?;
            (i < cols && j < rows).then_some(j * cols + i)
        };
        let (Some(start), Some(goal)) = (index(from), index(to)) else {
            return false;
        };
        if !free[start] || !free[goal] {
            return false;
        }
        let mut seen = vec![false; cols * rows];
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(at) = stack.pop() {
            if at == goal {
                return true;
            }
            let (i, j) = (at % cols, at / cols);
            let mut push = |n: usize| {
                if free[n] && !seen[n] {
                    seen[n] = true;
                    stack.push(n);
                }
            };
            if i > 0 {
                push(at - 1);
            }
            if i + 1 < cols {
                push(at + 1);
            }
            if j > 0 {
                push(at - cols);
            }
            if j + 1 < rows {
                push(at + cols);
            }
        }
        false
    }
}

fn ground(x: i32, y: i32) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

/// A start the planner may begin at: a free point at least 40 mm from every grown edge, so the flood's
/// nearest node is free too. [`None`] when 200 tries find none.
fn start(scene: &Scene, grown: &[Rect], cases: &mut Cases) -> Option<(i64, i64)> {
    for _ in 0..200 {
        let (x, y) = (
            i64::from(cases.next(scene.floor.x1)),
            i64::from(cases.next(scene.floor.y1)),
        );
        let roomy = [
            (0, 0),
            (40, 0),
            (-40, 0),
            (0, 40),
            (0, -40),
            (40, 40),
            (-40, -40),
            (40, -40),
            (-40, 40),
        ]
        .iter()
        .all(|(dx, dy)| scene.free(grown, x + dx, y + dy));
        if roomy {
            return Some((x, y));
        }
    }
    None
}

#[test]
fn every_route_keeps_its_clearance_and_every_refusal_has_no_way() {
    let mut cases = Cases(12);
    let (mut routed, mut unreachable, mut skipped, mut snapped, mut bends) = (0, 0, 0, 0, 0);
    for scene_index in 0..SCENES {
        let scene = Scene::generate(&mut cases);
        let grown = scene.grown();
        let Some(from) = start(&scene, &grown, &mut cases) else {
            skipped += 1;
            continue;
        };
        let to = (
            i64::from(cases.next(scene.floor.x1)),
            i64::from(cases.next(scene.floor.y1)),
        );
        let avoid = scene.avoid.map(|(x, y)| ground(x, y));
        let answer = route_in(
            &scene.shape(),
            &scene.bodies(),
            ground(
                i32::try_from(from.0).expect("small"),
                i32::try_from(from.1).expect("small"),
            ),
            ground(
                i32::try_from(to.0).expect("small"),
                i32::try_from(to.1).expect("small"),
            ),
            avoid,
        );
        let end = if scene.free(&grown, to.0, to.1) {
            Some(to)
        } else {
            scene.nearest_free(&grown, to.0, to.1)
        };
        match answer {
            RouteAnswer::Waypoints(waypoints) => {
                routed += 1;
                let points: Vec<(i64, i64)> = waypoints
                    .points()
                    .iter()
                    .map(|p| (i64::from(p.x().value()), i64::from(p.y().value())))
                    .collect();
                assert!(
                    points.len() <= 64,
                    "scene {scene_index}: {} waypoints",
                    points.len()
                );
                bends += points.len() - 1;
                if end != Some(to) {
                    snapped += 1;
                }
                assert_eq!(
                    points.last().copied(),
                    end,
                    "scene {scene_index}: the route's end"
                );
                let mut at = from;
                for next in points {
                    let length2 = (next.0 - at.0).pow(2) + (next.1 - at.1).pow(2);
                    let steps = (length2.isqrt() / 10).max(1) + 1;
                    for k in 0..=steps {
                        let x = at.0 + (next.0 - at.0) * k / steps;
                        let y = at.1 + (next.1 - at.1) * k / steps;
                        if let Some(offence) = scene.clearance_offence(x, y) {
                            panic!("scene {scene_index}: from {from:?} to {to:?}: {offence}");
                        }
                    }
                    at = next;
                }
            }
            RouteAnswer::Unreachable => {
                unreachable += 1;
                if let Some(end) = end {
                    assert!(
                        !scene.flood_reaches(&grown, from, end),
                        "scene {scene_index}: unreachable, but the flood fill reaches {end:?} from \
                         {from:?}"
                    );
                }
            }
        }
    }
    println!(
        "NV-5: {SCENES} scenes — {routed} routed ({bends} bends, {snapped} goals moved), \
         {unreachable} unreachable, {skipped} without a start"
    );
    assert!(
        routed > 500 && unreachable > 20 && bends > 500,
        "the scenes exercise both answers"
    );
}

/// A start where the resolver may leave a person — at least R + GAP (310 mm) clear of every solid,
/// object and the floor's edge — but inside the planner's margin, so not free: [`None`] when 400 tries
/// find none.
fn margin_start(scene: &Scene, grown: &[Rect], cases: &mut Cases) -> Option<(i64, i64)> {
    for _ in 0..400 {
        let (x, y) = (
            i64::from(cases.next(scene.floor.x1)),
            i64::from(cases.next(scene.floor.y1)),
        );
        if scene.clearance_offence(x, y).is_none() && !scene.free(grown, x, y) {
            return Some((x, y));
        }
    }
    None
}

/// Whether the straight step from `a` to `b` keeps out of every obstacle grown by R + GAP (the
/// resolver's clearance, squares for balls as the planner models them) and inside the floor shrunk by
/// it — sampled every 10 mm, the oracle's own test.
fn step_clear(scene: &Scene, a: (i64, i64), b: (i64, i64)) -> bool {
    let near: Vec<Rect> = scene
        .solids
        .iter()
        .map(|s| s.grown(310))
        .chain(scene.objects.iter().map(|o| o.square().grown(310)))
        .collect();
    let length2 = (b.0 - a.0).pow(2) + (b.1 - a.1).pow(2);
    let steps = (length2.isqrt() / 10).max(1) + 1;
    (0..=steps).all(|k| {
        let (x, y) = (a.0 + (b.0 - a.0) * k / steps, a.1 + (b.1 - a.1) * k / steps);
        x >= R_GAP
            && y >= R_GAP
            && x <= i64::from(scene.floor.x1) - R_GAP
            && y <= i64::from(scene.floor.y1) - R_GAP
            && !near.iter().any(|rect| rect.holds_open(x, y))
    })
}

/// Step-11 §21.15 M-3 (the operator's ruling): a start in the margin — legal for the resolver, not
/// free for the planner — first steps to the nearest free point, the oracle's own brute-force snap
/// (distance, then y, then x), when that point is within the margin's width (360 mm), and the route
/// goes on from there: every later leg keeps R + GAP from everything, and a refusal is confirmed by the
/// flood fill from the snapped start. With no free point that near the start is held (N-D7's core
/// rule) and every leg's clearance is claimed. Before the fix every margin start was planned from where
/// it stood, and alice at the café's bench had no way out (E-NW4).
#[test]
fn a_start_in_the_margin_steps_to_the_nearest_free_point_and_goes_on() {
    let mut cases = Cases(13);
    let (mut routed, mut unreachable, mut skipped, mut stepped, mut held) = (0, 0, 0, 0, 0);
    let mut close_steps = 0;
    for scene_index in 0..SCENES {
        let mut scene = Scene::generate(&mut cases);
        scene.avoid = None;
        let grown = scene.grown();
        let Some(from) = margin_start(&scene, &grown, &mut cases) else {
            skipped += 1;
            continue;
        };
        let to = (
            i64::from(cases.next(scene.floor.x1)),
            i64::from(cases.next(scene.floor.y1)),
        );
        let answer = route_in(
            &scene.shape(),
            &scene.bodies(),
            ground(
                i32::try_from(from.0).expect("small"),
                i32::try_from(from.1).expect("small"),
            ),
            ground(
                i32::try_from(to.0).expect("small"),
                i32::try_from(to.1).expect("small"),
            ),
            None,
        );
        let snapped = scene
            .nearest_free(&grown, from.0, from.1)
            .filter(|s| (s.0 - from.0).pow(2) + (s.1 - from.1).pow(2) <= 360 * 360);
        if snapped.is_some() {
            stepped += 1;
        } else {
            held += 1;
        }
        let end = if scene.free(&grown, to.0, to.1) {
            Some(to)
        } else {
            scene.nearest_free(&grown, to.0, to.1)
        };
        match answer {
            RouteAnswer::Waypoints(waypoints) => {
                routed += 1;
                let points: Vec<(i64, i64)> = waypoints
                    .points()
                    .iter()
                    .map(|p| (i64::from(p.x().value()), i64::from(p.y().value())))
                    .collect();
                let mut at = from;
                let mut legs = points.clone();
                if let Some(first) = snapped {
                    assert_eq!(
                        points.first().copied(),
                        Some(first),
                        "scene {scene_index}: from {from:?}, the first step is the nearest free point"
                    );
                    // The step out of the margin is a stride the resolver resolves; whether its
                    // straight line keeps R + GAP is counted, not claimed.
                    if !step_clear(&scene, from, first) {
                        close_steps += 1;
                    }
                    at = first;
                    legs.remove(0);
                }
                assert_eq!(points.last().copied(), end, "scene {scene_index}: the end");
                for next in legs {
                    let length2 = (next.0 - at.0).pow(2) + (next.1 - at.1).pow(2);
                    let steps = (length2.isqrt() / 10).max(1) + 1;
                    for k in 0..=steps {
                        let x = at.0 + (next.0 - at.0) * k / steps;
                        let y = at.1 + (next.1 - at.1) * k / steps;
                        if let Some(offence) = scene.clearance_offence(x, y) {
                            panic!(
                                "scene {scene_index}: from {from:?} to {to:?} by {points:?}: {offence}"
                            );
                        }
                    }
                    at = next;
                }
            }
            RouteAnswer::Unreachable => {
                unreachable += 1;
                if let (Some(start), Some(end)) = (snapped, end) {
                    assert!(
                        !scene.flood_reaches(&grown, start, end),
                        "scene {scene_index}: unreachable, but the flood fill reaches {end:?} from \
                         the snapped start {start:?}"
                    );
                }
            }
        }
    }
    println!(
        "M-3: {SCENES} scenes — {routed} routed from a margin start, {unreachable} unreachable, \
         {skipped} without one; {stepped} snapped within 360 mm ({close_steps} of their steps pass \
         nearer than 310 to something), {held} held"
    );
    assert!(
        routed > 500 && stepped > 500,
        "margin starts are routed: {routed}"
    );
}
