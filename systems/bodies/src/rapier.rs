//! The adapter to Rapier (`DECISIONS.md` `DEP-13`): **the only file in this pack, and in the
//! workspace's code, that names `rapier3d`.**
//!
//! Integers in, integers out. A [`Scene`] is built for one place from a [`Room`] and the positions of
//! the people standing in it, answers sweeps, and is dropped when the resolution that built it ends:
//! nothing of Rapier survives a resolution — no component, field, static or cache holds any of it
//! (step-11 I-5). What crosses the boundary is converted by exactly two functions, [`metres`] in and
//! [`millimetres`] out (`(metres × 1000).round()` to `i32`, step-11 DC-6).
//!
//! # A scene, in canonical order
//!
//! Rapier's results depend on insertion order (step-11 F-P2), so a scene inserts in one documented
//! order, a function of the integers alone (DC-2):
//!
//! ```text
//! 1  the floor slab, its top at z = 0, one metre wider than the floor on every side
//! 2  the four perimeter walls, west, east, south, north: 200 mm thick, 3 000 mm high, standing just
//!    outside the floor's edge
//! 3  the solids, in authored order, from the floor up to their height
//! 4  the people, in the order given (the caller gives EntityId order), as kinematic position-based
//!    capsules: radius 300 mm, half-segment 560 mm, centre 870 mm up (feet 10 mm above the floor)
//! ```
//!
//! then detects collisions once, so that queries see every collider, and re-marks every dynamic body
//! (see [`refresh`] for the Rapier 0.36.0 defect that makes this necessary, step-11 F-P1).
//!
//! # A sweep
//!
//! The character controller is the prototype's, which measured mode R′ (step-11 §§9.5, 9.8): up is
//! +z, a 10 mm offset, sliding on, no autostep, no snapping to the ground, `dt` 1/60 s. A sweep moves a
//! capsule from a point by a planar offset against the fixed geometry only, or against the fixed
//! geometry and the people, and answers where it ends, quantized, and the first person it touched.
//! No step of the simulation ever runs here.

use rapier3d::control::{CharacterLength, KinematicCharacterController};
use rapier3d::prelude::*;

use crate::geometry::{GAP, PERSON_HEIGHT, PERSON_RADIUS, Point, Room};

/// The character controller's fixed time step: one sixtieth of a second (step-11 DC-4).
const DT: f32 = 1.0 / 60.0;

/// Gravity, down the world's z axis. Nothing steps in a resolution of people; it is set so that the
/// one place that does step — the F-P1 tests — falls the way the world does.
const GRAVITY_Z: f32 = -9.81;

/// The perimeter walls' thickness and height, and the floor slab's thickness and overhang.
const WALL_THICKNESS: i32 = 200;
const WALL_HEIGHT: i32 = 3_000;
const SLAB_THICKNESS: i32 = 200;
const SLAB_OVERHANG: i32 = 1_000;

/// A person's capsule: half of the straight segment between its two half-spheres, and how high its
/// centre stands (feet 10 mm above the floor, as the prototype).
const HALF_SEGMENT: i32 = (PERSON_HEIGHT.value() - 2 * PERSON_RADIUS.value()) / 2;
const CENTRE_Z: i32 = HALF_SEGMENT + PERSON_RADIUS.value() + 10;

/// Integer millimetres in: the one conversion into Rapier's metres.
fn metres(millimetres: i32) -> f32 {
    // Exact: every coordinate this pack accepts is within ±100 000 mm (COORDINATE_BOUND), well
    // inside f32's 24-bit mantissa.
    #[allow(clippy::cast_precision_loss)]
    let value = millimetres as f32;
    value / 1000.0
}

/// Rapier's metres out: the one quantization, to the nearest whole millimetre (step-11 DC-6).
fn millimetres(metres: f32) -> i32 {
    #[allow(clippy::cast_possible_truncation)]
    let value = (metres * 1000.0).round() as i32;
    value
}

/// What a sweep may run into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Against {
    /// The floor, the walls and the solids only: a walls-only reach, and every nudge.
    Fixed,
    /// The fixed geometry and every person in the scene but the one moving: a contact reach.
    FixedAndPeople,
}

/// Where a sweep ended, quantized; the first person it touched, by the index the scene was built
/// with; and where the capsule stood, quantized, when it first touched them — before the controller
/// slid it on along their curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Swept {
    pub(crate) end: Point,
    pub(crate) touched: Option<usize>,
    pub(crate) contact: Option<Point>,
}

/// One place, built for one resolution and dropped with it.
pub(crate) struct Scene {
    world: PhysicsWorld,
    people: Vec<(RigidBodyHandle, ColliderHandle)>,
}

impl Scene {
    /// The place `room`, with a person standing at each of `people`, inserted in canonical order.
    pub(crate) fn build(room: &Room, people: &[Point]) -> Self {
        let mut world = empty_world();
        insert_fixed(&mut world, room);
        let people = people
            .iter()
            .map(|at| {
                world.insert(
                    RigidBodyBuilder::kinematic_position_based().translation(centre(*at)),
                    ColliderBuilder::capsule_z(metres(HALF_SEGMENT), metres(PERSON_RADIUS.value())),
                )
            })
            .collect();
        refresh(&mut world);
        Self { world, people }
    }

    /// Sweeps a person's capsule from `from` by the planar offset `by`.
    ///
    /// `mover` is the moving person's index in the scene, so that the sweep does not run into their
    /// own body; [`None`] for a person who is not in the scene (arriving from elsewhere).
    pub(crate) fn sweep(
        &self,
        mover: Option<usize>,
        from: Point,
        by: Point,
        against: Against,
    ) -> Swept {
        let filter = match against {
            Against::Fixed => QueryFilter::only_fixed(),
            Against::FixedAndPeople => QueryFilter::exclude_dynamic(),
        };
        let filter = match mover {
            Some(index) => filter.exclude_rigid_body(self.people[index].0),
            None => filter,
        };
        let queries = self.world.query_pipeline_with_filter(filter);
        let mut touched = None;
        let moved = controller().move_shape(
            DT,
            &queries,
            &capsule(),
            &Pose::from_translation(centre(from)),
            Vector::new(metres(by.x), metres(by.y), 0.0),
            |collision| {
                if touched.is_none() {
                    touched = self
                        .people
                        .iter()
                        .position(|(_, collider)| *collider == collision.handle)
                        .map(|index| (index, collision.translation_applied));
                }
            },
        );
        let at = |translation: Vector| {
            Point::new(
                millimetres(metres(from.x) + translation.x),
                millimetres(metres(from.y) + translation.y),
            )
        };
        Swept {
            end: at(moved.translation),
            touched: touched.map(|(index, _)| index),
            contact: touched.map(|(_, applied)| at(applied)),
        }
    }
}

/// A world with nothing in it yet: gravity down z, the fixed time step.
fn empty_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    world.gravity = Vector::new(0.0, 0.0, GRAVITY_Z);
    world.integration_parameters.dt = DT;
    world
}

/// Steps 1–3 of the canonical order: the slab, the four walls, the solids in authored order.
fn insert_fixed(world: &mut PhysicsWorld, room: &Room) {
    let (min, max) = (room.floor.min, room.floor.max);
    // Centres and half extents are computed from integer sums, then halved in floating point, which
    // is exact.
    let mid = |a: i32, b: i32| metres(a + b) / 2.0;
    let half = |a: i32, b: i32| metres(b - a) / 2.0;
    let fixed =
        |x: f32, y: f32, z: f32| RigidBodyBuilder::fixed().translation(Vector::new(x, y, z));
    let (wall, height) = (WALL_THICKNESS, WALL_HEIGHT);

    world.insert(
        fixed(
            mid(min.x, max.x),
            mid(min.y, max.y),
            -metres(SLAB_THICKNESS) / 2.0,
        ),
        ColliderBuilder::cuboid(
            half(min.x, max.x) + metres(SLAB_OVERHANG),
            half(min.y, max.y) + metres(SLAB_OVERHANG),
            metres(SLAB_THICKNESS) / 2.0,
        ),
    );
    let (wall_z, wall_half_z) = (metres(height) / 2.0, metres(height) / 2.0);
    for (x, y, half_x, half_y) in [
        // west and east: spanning the floor's depth and both corners
        (
            metres(2 * min.x - wall) / 2.0,
            mid(min.y, max.y),
            metres(wall) / 2.0,
            half(min.y, max.y) + metres(wall),
        ),
        (
            metres(2 * max.x + wall) / 2.0,
            mid(min.y, max.y),
            metres(wall) / 2.0,
            half(min.y, max.y) + metres(wall),
        ),
        // south and north: spanning the floor's width
        (
            mid(min.x, max.x),
            metres(2 * min.y - wall) / 2.0,
            half(min.x, max.x),
            metres(wall) / 2.0,
        ),
        (
            mid(min.x, max.x),
            metres(2 * max.y + wall) / 2.0,
            half(min.x, max.x),
            metres(wall) / 2.0,
        ),
    ] {
        world.insert(
            fixed(x, y, wall_z),
            ColliderBuilder::cuboid(half_x, half_y, wall_half_z),
        );
    }
    for (area, height) in &room.solids {
        world.insert(
            fixed(
                mid(area.min.x, area.max.x),
                mid(area.min.y, area.max.y),
                metres(*height) / 2.0,
            ),
            ColliderBuilder::cuboid(
                half(area.min.x, area.max.x),
                half(area.min.y, area.max.y),
                metres(*height) / 2.0,
            ),
        );
    }
}

/// Builds the broad phase so that queries see every collider, then works around a Rapier 0.36.0
/// defect.
///
/// FINDING (step-11 F-P1, `DEP-13`): `PhysicsWorld::detect_collisions` on a freshly built world,
/// before its first step, leaves every dynamic body un-integrated afterwards — a box placed in the air
/// does not fall. The collision pipeline consumes the bodies' "modified" flags while passing no island
/// manager (upstream behaviour since v0.35.0: "`CollisionPipeline::step` now clears the rigid-bodies'
/// modified flags"), so the next step never registers them. Re-marking each dynamic body as modified
/// does. No resolution of people has a dynamic body; the re-mark is here so that the first one that
/// does (S15's objects) starts from a guarded adapter, and the canary below turns red the day upstream
/// changes the behaviour.
fn refresh(world: &mut PhysicsWorld) {
    world.detect_collisions(&(), &());
    // Borrowing the bodies mutably is itself what re-marks them: `RigidBodySet::iter_mut` pushes every
    // body it yields onto the modified list (step-11 §17.11, DB-2). `set_translation(.., true)` states
    // the intent and wakes the body.
    for (_, body) in world.bodies.iter_mut() {
        if body.is_dynamic() {
            let at = body.translation();
            body.set_translation(at, true);
        }
    }
}

/// The capsule's centre for a person standing at `at`.
fn centre(at: Point) -> Vector {
    Vector::new(metres(at.x), metres(at.y), metres(CENTRE_Z))
}

fn capsule() -> Capsule {
    Capsule::new_z(metres(HALF_SEGMENT), metres(PERSON_RADIUS.value()))
}

/// The prototype's character controller (step-11 SD-B11).
fn controller() -> KinematicCharacterController {
    KinematicCharacterController {
        up: Vector::Z,
        offset: CharacterLength::Absolute(metres(GAP.value())),
        slide: true,
        autostep: None,
        snap_to_ground: None,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    //! PB-3: the F-P1 canary and its fix. Integers in, as everywhere else; the box is the prototype's
    //! 0.4 m cube placed 0.8 m up, over the floor slab of a 2 m room.

    use super::*;
    use crate::geometry::Area;

    fn room() -> Room {
        Room {
            floor: Area {
                min: Point::new(0, 0),
                max: Point::new(2_000, 2_000),
            },
            solids: Vec::new(),
        }
    }

    /// The box's height after 20 steps, in millimetres, with or without the adapter's re-mark.
    fn fallen_after_twenty_steps(remark: bool) -> i32 {
        let mut world = empty_world();
        insert_fixed(&mut world, &room());
        let (cube, _) = world.insert(
            RigidBodyBuilder::dynamic().translation(Vector::new(
                metres(1_000),
                metres(1_000),
                metres(800),
            )),
            ColliderBuilder::cuboid(metres(200), metres(200), metres(200)),
        );
        if remark {
            refresh(&mut world);
        } else {
            world.detect_collisions(&(), &());
        }
        for _ in 0..20 {
            world.step();
        }
        millimetres(world.bodies[cube].translation().z)
    }

    /// The defect as it is in the pinned version: without the re-mark the box never moves. If this
    /// fails, upstream changed it, and the workaround is removed deliberately (`DEP-13`).
    #[test]
    fn canary_rapier_0_36_0_leaves_a_box_hanging_after_detect_collisions() {
        let z = fallen_after_twenty_steps(false);
        println!("without the re-mark: z = {z} mm after 20 steps");
        assert!((799..=801).contains(&z), "the defect is present: {z} mm");
    }

    /// The adapter's refresh: the box falls (the prototype measured 291 mm).
    #[test]
    fn the_adapters_refresh_lets_a_box_fall() {
        let z = fallen_after_twenty_steps(true);
        println!("with the re-mark: z = {z} mm after 20 steps");
        assert!(z <= 300, "the box fell: {z} mm");
    }

    /// The two conversions round trip every millimetre within the coordinate bound exactly, so a
    /// position that goes into a scene and comes back unmoved comes back unchanged.
    #[test]
    fn a_millimetre_round_trips_exactly_within_the_coordinate_bound() {
        let bound = crate::geometry::COORDINATE_BOUND.value();
        let broken: Vec<i32> = (-bound..=bound)
            .step_by(7)
            .chain([-bound, bound, 0, 1, -1])
            .filter(|mm| millimetres(metres(*mm)) != *mm)
            .collect();
        assert!(
            broken.is_empty(),
            "round trips: {:?}",
            &broken[..broken.len().min(5)]
        );
    }
}
