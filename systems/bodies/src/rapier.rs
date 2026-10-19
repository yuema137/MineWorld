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
//!    capsules: radius `PERSON_RADIUS` (250 mm), half-segment `(PERSON_HEIGHT − 2r) / 2` (610 mm),
//!    centre `half-segment + r + 10` up (870 mm; feet 10 mm above the floor)
//! ```
//!
//! then builds the broad phase once, so that queries see every collider. A [`Scene`] of people builds
//! the broad phase and nothing else ([`index`], step-11 SD-Z2): no query reads the narrow phase. A
//! [`Pile`] and a flight also detect collisions and re-mark every dynamic body (see [`refresh`] for
//! the Rapier 0.36.0 defect that makes this necessary, step-11 F-P1).
//!
//! # A sweep
//!
//! The character controller is the prototype's, which measured mode R′ (step-11 §§9.5, 9.8): up is
//! +z, a 10 mm offset, sliding on, no autostep, no snapping to the ground, `dt` 1/60 s. A sweep moves a
//! capsule from a point by a planar offset against the fixed geometry only, or against the fixed
//! geometry and the people, and answers where it ends, quantized, and the first person it touched.
//! No step of the simulation ever runs here.

use rapier3d::control::{CharacterLength, KinematicCharacterController};
use rapier3d::parry::query::ShapeCastOptions;
use rapier3d::prelude::*;

use crate::component::BodyShape;
use crate::footprint::Placed;
use crate::geometry::{
    GAP, PATH_EVERY, PATH_MAX, PERSON_HEIGHT, PERSON_RADIUS, Point, REST_SPEED, REST_STEPS, Room,
};

/// The character controller's fixed time step: one sixtieth of a second (step-11 DC-4).
const DT: f32 = 1.0 / 60.0;

/// Every object's material (step-11 §18.3.1): what a flight slides and bounces with.
const FRICTION: f32 = 0.5;
const RESTITUTION: f32 = 0.1;

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
    /// The floor, the walls and the solids only: a walls-only reach, and a nudge while objects are
    /// pushed.
    Walls,
    /// The fixed geometry and the loose objects: a walls-only reach, and a nudge, while objects are
    /// solid (step-11 SD-O9 step 6).
    WallsAndObjects,
    /// The fixed geometry, the loose objects and every person in the scene but the one moving: a
    /// contact reach.
    Contact,
}

/// What a sweep touched first: a person or a loose object, by the index the scene was built with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Touch {
    Person(usize),
    Object(usize),
}

/// Where a sweep ended, quantized; the first person or object it touched; and where the capsule
/// stood, quantized, when it first touched it — before the controller slid it on along its curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Swept {
    pub(crate) end: Point,
    pub(crate) touched: Option<Touch>,
    pub(crate) contact: Option<Point>,
}

/// One place, built for one resolution and dropped with it.
pub(crate) struct Scene {
    world: PhysicsWorld,
    people: Vec<(RigidBodyHandle, ColliderHandle)>,
    objects: Vec<ColliderHandle>,
}

impl Scene {
    /// The place `room`, with a person standing at each of `people` and the loose `objects` lying in
    /// it, inserted in canonical order (people, then objects, each in the order given).
    ///
    /// For a person's sweep an object is its **footprint, extruded** from the floor to the walls'
    /// height: a box's rectangle as a tall box, a ball's disc as a tall capsule. The integer checks
    /// all read footprints (step-11 SD-O2), and a sweep must see the same outline: against a low
    /// ball's real shape the capsule's rounded bottom meets the ball below its equator, 53 mm later
    /// than the footprint says, and the walker ends inside the footprint the checks then refuse
    /// (§18.11 DO-4).
    pub(crate) fn build(room: &Room, people: &[Point], objects: &[Placed]) -> Self {
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
        let objects = objects
            .iter()
            .map(|object| {
                let (body, collider) = post(object);
                world.insert(body, collider).1
            })
            .collect();
        index(&mut world);
        Self {
            world,
            people,
            objects,
        }
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
        self.sweep_past(mover, from, by, against, &[])
    }

    /// [`Scene::sweep`], with the people at the scene indexes `aside` left out of a contact sweep
    /// (step-11 SD-Z5: people the mover stands within the offset of and moves away from). A walls
    /// sweep never meets people, so `aside` matters only for [`Against::Contact`].
    pub(crate) fn sweep_past(
        &self,
        mover: Option<usize>,
        from: Point,
        by: Point,
        against: Against,
        aside: &[usize],
    ) -> Swept {
        let not_an_object = |handle: ColliderHandle, _: &Collider| !self.objects.contains(&handle);
        let passed: Vec<ColliderHandle> = aside.iter().map(|index| self.people[*index].1).collect();
        let not_passed = |handle: ColliderHandle, _: &Collider| !passed.contains(&handle);
        let filter = match against {
            Against::Walls => QueryFilter::only_fixed().predicate(&not_an_object),
            Against::WallsAndObjects => QueryFilter::only_fixed(),
            Against::Contact if passed.is_empty() => QueryFilter::exclude_dynamic(),
            Against::Contact => QueryFilter::exclude_dynamic().predicate(&not_passed),
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
                        .touch(collision.handle)
                        .map(|touch| (touch, collision.translation_applied));
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
            touched: touched.map(|(touch, _)| touch),
            contact: touched.map(|(_, applied)| at(applied)),
        }
    }

    /// The person or object a collider belongs to; [`None`] for the fixed geometry.
    fn touch(&self, handle: ColliderHandle) -> Option<Touch> {
        self.people
            .iter()
            .position(|(_, collider)| *collider == handle)
            .map(Touch::Person)
            .or_else(|| {
                self.objects
                    .iter()
                    .position(|collider| *collider == handle)
                    .map(Touch::Object)
            })
    }
}

/// The loose objects of one place as they lay before a request, for the push's shape cast (step-11
/// SD-O8): the fixed geometry, then each object as its real shape, in the order given. No people:
/// in a reaction they are not yet where the request leaves them (step-11 F-O2), so the resolver
/// checks them instead. Built once per prediction or per reaction, and dropped with it.
pub(crate) struct Pile {
    world: PhysicsWorld,
    objects: Vec<ColliderHandle>,
    placed: Vec<Placed>,
}

impl Pile {
    pub(crate) fn build(room: &Room, objects: &[Placed]) -> Self {
        let mut world = empty_world();
        insert_fixed(&mut world, room);
        let handles = objects
            .iter()
            .map(|object| {
                let (body, collider) = real(object, 0);
                world.insert(body, collider).1
            })
            .collect();
        refresh(&mut world);
        Self {
            world,
            objects: handles,
            placed: objects.to_vec(),
        }
    }

    /// Casts object `index`'s shape along the planar `offset` against the fixed geometry and every
    /// other object, and answers where its centre would stop, quantized. The shape is cast one
    /// millimetre above where it rests, so the floor or the solid under it is never its first hit.
    pub(crate) fn cast(&self, index: usize, offset: Point) -> Point {
        let object = self.placed[index];
        let filter = QueryFilter::only_fixed().exclude_collider(self.objects[index]);
        let queries = self.world.query_pipeline_with_filter(filter);
        let start = lifted(&object, 1);
        let collider = shape_of(object.shape);
        let travel = Vector::new(metres(offset.x), metres(offset.y), 0.0);
        let options = ShapeCastOptions {
            max_time_of_impact: 1.0,
            target_distance: 0.0,
            stop_at_penetration: false,
            compute_impact_geometry_on_penetration: false,
        };
        let shape = collider.build();
        let fraction = queries
            .cast_shape(
                &Pose::from_translation(start),
                travel,
                shape.shape(),
                options,
            )
            .map_or(1.0, |(_, hit)| hit.time_of_impact);
        Point::new(
            millimetres(start.x + travel.x * fraction),
            millimetres(start.y + travel.y * fraction),
        )
    }
}

/// A position in a place's frame, in whole millimetres: `(x, y)` on the floor and `z` up.
pub(crate) type At = (Point, i32);

/// One object's flight (step-11 SD-O14), simulated at the instant of a kick or a throw: where it came
/// to rest, quantized, and its keyframes for a client to animate.
pub(crate) struct Flown {
    pub(crate) end: At,
    pub(crate) path: Vec<At>,
}

/// What a flight is launched into: the place, the people standing in it, the other objects as they
/// lie, and the flying object with its launch velocity (millimetres per second) and its step bound.
pub(crate) struct Launch<'a> {
    pub(crate) room: &'a Room,
    pub(crate) people: &'a [Point],
    pub(crate) others: &'a [Placed],
    pub(crate) flying: Placed,
    pub(crate) velocity: (i32, i32, i32),
    pub(crate) steps: u32,
}

/// Simulates one flight (step-11 SD-O14; `DEP-13` note): a scene of the fixed geometry, the people as
/// kinematic capsules, the other objects fixed as their real shapes, and the flying object last —
/// dynamic, rotations locked, with continuous collision detection. After [`refresh`] (F-P1's re-mark)
/// the launch velocity is set, and the world steps at 1/60 s until the object has been slower than
/// `REST_SPEED` for `REST_STEPS` consecutive sub-steps, or the step bound. A keyframe every
/// `PATH_EVERY` sub-steps, at most `PATH_MAX`, the first being where it started. Nothing else moves.
pub(crate) fn fly(launch: &Launch<'_>) -> Flown {
    let mut world = empty_world();
    insert_fixed(&mut world, launch.room);
    for at in launch.people {
        world.insert(
            RigidBodyBuilder::kinematic_position_based().translation(centre(*at)),
            ColliderBuilder::capsule_z(metres(HALF_SEGMENT), metres(PERSON_RADIUS.value())),
        );
    }
    for other in launch.others {
        let (body, collider) = real(other, 0);
        world.insert(body, collider);
    }
    let flying = launch.flying;
    let (handle, _) = world.insert(
        RigidBodyBuilder::dynamic()
            .translation(lifted(&flying, 0))
            .lock_rotations()
            .ccd_enabled(true),
        shape_of(flying.shape),
    );
    refresh(&mut world);
    let (vx, vy, vz) = launch.velocity;
    world.bodies[handle].set_linvel(Vector::new(metres(vx), metres(vy), metres(vz)), true);

    let at = |world: &PhysicsWorld| {
        let t = world.bodies[handle].translation();
        (
            Point::new(millimetres(t.x), millimetres(t.y)),
            millimetres(t.z),
        )
    };
    let rest = metres(REST_SPEED);
    let mut path = vec![(flying.centre, flying.z)];
    let mut slow = 0;
    for step in 1..=launch.steps {
        world.step();
        if step % PATH_EVERY == 0 && path.len() < PATH_MAX {
            path.push(at(&world));
        }
        let speed = world.bodies[handle].linvel();
        if speed.x * speed.x + speed.y * speed.y + speed.z * speed.z < rest * rest {
            slow += 1;
            if slow >= REST_STEPS {
                break;
            }
        } else {
            slow = 0;
        }
    }
    Flown {
        end: at(&world),
        path,
    }
}

/// An object's footprint extruded from below the floor to above the walls, for people's sweeps.
fn post(object: &Placed) -> (RigidBodyBuilder, ColliderBuilder) {
    let half = metres(WALL_HEIGHT) / 2.0;
    let at = Vector::new(metres(object.centre.x), metres(object.centre.y), half);
    let collider = match object.shape {
        BodyShape::Box(extents) => ColliderBuilder::cuboid(
            metres(extents.x().value()),
            metres(extents.y().value()),
            half,
        ),
        BodyShape::Ball(radius) => ColliderBuilder::capsule_z(half, metres(radius.value())),
    };
    (RigidBodyBuilder::fixed().translation(at), collider)
}

/// An object's real shape at its centre, `lift` millimetres up.
fn real(object: &Placed, lift: i32) -> (RigidBodyBuilder, ColliderBuilder) {
    (
        RigidBodyBuilder::fixed().translation(lifted(object, lift)),
        shape_of(object.shape),
    )
}

/// An object's centre, `lift` millimetres above where it rests.
fn lifted(object: &Placed, lift: i32) -> Vector {
    Vector::new(
        metres(object.centre.x),
        metres(object.centre.y),
        metres(object.z + lift),
    )
}

/// A collider of an object's shape, with the material every object has.
fn shape_of(shape: BodyShape) -> ColliderBuilder {
    let collider = match shape {
        BodyShape::Box(half) => ColliderBuilder::cuboid(
            metres(half.x().value()),
            metres(half.y().value()),
            metres(half.z().value()),
        ),
        BodyShape::Ball(radius) => ColliderBuilder::ball(metres(radius.value())),
    };
    collider.friction(FRICTION).restitution(RESTITUTION)
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

/// Builds the broad phase from every collider inserted so far, and nothing else: what a scene's sweeps
/// read (step-11 SD-Z2, F-Z2).
///
/// A query pipeline is the broad phase's tree read through the narrow phase's query dispatcher
/// (`PhysicsWorld::query_pipeline_with_filter` in 0.36.0); no sweep reads a contact the narrow phase
/// computes. So this is `CollisionPipeline::step` without its narrow phase: the same broad-phase
/// update, with the same parameters (the world's prediction distance, `dt` 0) and the same modified
/// colliders in insertion order, builds the same tree. The rigid bodies' user changes it skips carry
/// nothing for a freshly built scene: a collider's pose is set from its parent's when it is inserted.
/// Nothing dynamic is ever in a scene, so F-P1's re-mark does not apply.
fn index(world: &mut PhysicsWorld) {
    let parameters = IntegrationParameters {
        normalized_prediction_distance: world.integration_parameters.prediction_distance(),
        dt: 0.0,
        ..IntegrationParameters::default()
    };
    let inserted = world.colliders.take_modified();
    let removed = world.colliders.take_removed();
    let mut pairs = Vec::new();
    world.broad_phase.update(
        &parameters,
        &world.colliders,
        &world.bodies,
        &inserted,
        &removed,
        &mut pairs,
    );
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
