//! Where things are, in the world's own vocabulary — and what an action requires of that.
//!
//! `docs/ENGINEERING_RULES.md` §5 separates two ideas that are easy to collapse into one:
//!
//! ```text
//! semantic world position     the simulation's:   a Place, and optionally where in it
//! render-space position       one client's:       a mesh transform, a navmesh point,
//!                                                an animation anchor, a camera
//! ```
//!
//! Only the first belongs here. A [`Location`] is a [`PlaceId`] with an *optional* continuous
//! refinement, which is what lets one type describe both "standing 1.2 m from Alice, facing her,
//! inside the café" and "in the café, position irrelevant" (`DD-4`). A headless world and a 2D
//! client leave the refinement out and lose nothing; an embodied 3D client fills it in and needs
//! no other type.
//!
//! # Integers, because a log must replay identically
//!
//! There is no floating point anywhere in this crate, and that is a determinism requirement
//! (`AC-12`), not a preference: positions reach the event log, the log is replayed, and floating
//! point arithmetic is not reproducible across platforms or even across optimization levels.
//! Positions are `i32` millimetres and angles are `i32` millidegrees — 0.001 mm and 0.001° of
//! resolution over a range of about ±2 147 km, which is more than an embodied world needs and is
//! exactly reproducible. `contracts/tests/spatial.rs` asserts structurally that no `f32` or `f64`
//! appears in the crate's sources.
//!
//! # Nothing here is engine-shaped
//!
//! No mesh, no navmesh, no collider, no camera, no scene node, no animation, no skeleton, no
//! physics (`docs/ENGINEERING_RULES.md` §12). Three numbers and two angles are not a rendering
//! concept; they are how far apart two things are, which the simulation has to know in order to
//! answer whether an interaction is possible at all.
//!
//! # The requirement is data, and the server evaluates it
//!
//! `docs/ENGINEERING_RULES.md` §§7–8: whether an action needs the same place, an interaction
//! radius, a line of access or an available target belongs to the System contract, and no renderer
//! decides it. [`SpatialRequirement`] is therefore a *value* a system declares, so that the server
//! can check it uniformly and a client can be *told* what an action requires without implementing
//! the check. [`SpatialRequirement::evaluate`] is that one implementation.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::action::Rejection;
use crate::error::ContractError;
use crate::ids::PlaceId;

/// A length or a coordinate in millimetres.
///
/// A newtype rather than a bare `i32`, so a distance cannot be passed where an angle is expected
/// and a unit mistake cannot compile. Signed, because a coordinate is relative to a place's origin
/// and may be on either side of it.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct Millimetres(i32);

impl Millimetres {
    /// Exactly at the origin, or no distance at all.
    pub const ZERO: Self = Self(0);

    /// The length or coordinate `millimetres`.
    pub const fn new(millimetres: i32) -> Self {
        Self(millimetres)
    }

    /// The value in millimetres.
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl fmt::Display for Millimetres {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}mm", self.0)
    }
}

/// An angle in thousandths of a degree.
///
/// Why not radians: a radian is only useful as a floating-point value, and this crate has none.
/// Millidegrees keep a full turn an exact integer — 360 000 — so a rotation can be normalized
/// without rounding, and 0.001° is far finer than any world needs to distinguish.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct Millidegrees(i32);

impl Millidegrees {
    /// No rotation.
    pub const ZERO: Self = Self(0);

    /// The angle `millidegrees`.
    pub const fn new(millidegrees: i32) -> Self {
        Self(millidegrees)
    }

    /// The value in millidegrees.
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl fmt::Display for Millidegrees {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}mdeg", self.0)
    }
}

/// Where inside a place something is, relative to that place's own origin.
///
/// The axes are the world's, not a renderer's: `x` and `y` span the ground plane and `z` is
/// height. A 2D client ignores `z`; a 3D client uses it; a headless world never constructs one of
/// these at all. What the world means by a place's origin is the world's own business — this
/// contract only requires that two positions inside one place are comparable, which is what makes
/// a distance meaningful.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct LocalPosition {
    x: Millimetres,
    y: Millimetres,
    z: Millimetres,
}

impl LocalPosition {
    /// A position at the place's origin.
    pub const ORIGIN: Self = Self {
        x: Millimetres::ZERO,
        y: Millimetres::ZERO,
        z: Millimetres::ZERO,
    };

    /// The position `(x, y, z)` relative to the place's origin.
    pub const fn new(x: Millimetres, y: Millimetres, z: Millimetres) -> Self {
        Self { x, y, z }
    }

    /// A position on the ground plane, for a world that does not model height.
    pub const fn on_ground(x: Millimetres, y: Millimetres) -> Self {
        Self {
            x,
            y,
            z: Millimetres::ZERO,
        }
    }

    /// Distance along the first ground axis.
    pub const fn x(self) -> Millimetres {
        self.x
    }

    /// Distance along the second ground axis.
    pub const fn y(self) -> Millimetres {
        self.y
    }

    /// Height. A 2D world leaves this at zero and a 2D client ignores it.
    pub const fn z(self) -> Millimetres {
        self.z
    }

    /// Whether `other` is no further away than `range`.
    ///
    /// Squared distances are compared in `i128` rather than distance in `i32`: two coordinates at
    /// opposite ends of the representable range are about 4 295 km apart, which does not fit in an
    /// `i32` of millimetres, and their squares do not fit in an `i64`. The comparison is exact —
    /// there is no square root, and therefore nothing to round.
    fn within(self, range: Millimetres, other: Self) -> bool {
        let dx = i128::from(self.x.value()) - i128::from(other.x.value());
        let dy = i128::from(self.y.value()) - i128::from(other.y.value());
        let dz = i128::from(self.z.value()) - i128::from(other.z.value());
        let squared_distance = dx * dx + dy * dy + dz * dz;
        let squared_range = i128::from(range.value()) * i128::from(range.value());
        squared_distance <= squared_range
    }
}

/// Which way something is facing.
///
/// `yaw` is the heading around the vertical axis, and `pitch` is optional because a world that
/// does not model looking up and down has no use for it — a 2D client sets yaw alone.
///
/// Yaw is *canonicalized* on construction: 450 000 millidegrees and 90 000 millidegrees are the
/// same direction, and storing them differently would put two representations of one fact in the
/// event log. That is not the same as repairing a bad value, which this crate never does — which is
/// why pitch, where 100° is not equivalent to any legal value but a different and impossible one,
/// is *rejected* instead of clamped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "OrientationFields")]
pub struct Orientation {
    yaw: Millidegrees,
    pitch: Option<Millidegrees>,
}

impl Orientation {
    /// One full turn, in millidegrees: the modulus yaw is canonicalized against.
    pub const FULL_TURN: Millidegrees = Millidegrees::new(360_000);

    /// The steepest legal pitch, in millidegrees: straight up, or straight down as its negation.
    pub const PITCH_LIMIT: Millidegrees = Millidegrees::new(90_000);

    /// Facing `yaw`, with no pitch — the whole of what a 2D world needs.
    ///
    /// Yaw is canonicalized into `[0, 360_000)`, so any multiple of a full turn is accepted and
    /// stored as the direction it names.
    pub const fn facing(yaw: Millidegrees) -> Self {
        Self {
            yaw: Millidegrees::new(yaw.value().rem_euclid(Self::FULL_TURN.value())),
            pitch: None,
        }
    }

    /// Facing `yaw`, looking `pitch` above or below the horizon.
    ///
    /// Rejects a pitch outside `±90°`: beyond straight up there is no steeper direction, so such a
    /// value is not another way of writing a legal one and must not be silently turned into one.
    pub fn new(yaw: Millidegrees, pitch: Option<Millidegrees>) -> Result<Self, ContractError> {
        if let Some(pitch) = pitch
            && pitch.value().unsigned_abs() > Self::PITCH_LIMIT.value().unsigned_abs()
        {
            return Err(ContractError::PitchOutOfRange {
                millidegrees: pitch.value(),
                limit: Self::PITCH_LIMIT.value(),
            });
        }
        Ok(Self {
            pitch,
            ..Self::facing(yaw)
        })
    }

    /// The heading, canonicalized into `[0, 360_000)` millidegrees.
    pub const fn yaw(self) -> Millidegrees {
        self.yaw
    }

    /// The angle above or below the horizon, if the world models one.
    pub const fn pitch(self) -> Option<Millidegrees> {
        self.pitch
    }
}

/// The serialized shape of an [`Orientation`], canonicalized and checked on the way back in.
///
/// A recorded orientation is read through the same constructor that produced it, so a hand-edited
/// save or a corrupted frame cannot introduce a yaw outside the canonical range or an impossible
/// pitch.
#[derive(Deserialize)]
struct OrientationFields {
    yaw: Millidegrees,
    pitch: Option<Millidegrees>,
}

impl TryFrom<OrientationFields> for Orientation {
    type Error = ContractError;

    fn try_from(value: OrientationFields) -> Result<Self, Self::Error> {
        Self::new(value.yaw, value.pitch)
    }
}

/// Where something is: a place, and optionally where in it and which way it faces.
///
/// The place is authoritative and always present; the rest is a refinement a world may not model
/// (`DD-4`). Place *hierarchy* is not here — that a kitchen is inside a café is a relation (S1),
/// not a field of this type, because it is a fact about two places rather than about anything
/// standing in one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Location {
    place: PlaceId,
    local: Option<LocalPosition>,
    facing: Option<Orientation>,
}

impl Location {
    /// Somewhere in `place`, position and orientation unmodelled or irrelevant.
    pub const fn in_place(place: PlaceId) -> Self {
        Self {
            place,
            local: None,
            facing: None,
        }
    }

    /// The same location, at a known position inside the place.
    #[must_use]
    pub const fn with_local(mut self, local: LocalPosition) -> Self {
        self.local = Some(local);
        self
    }

    /// The same location, facing a known direction.
    #[must_use]
    pub const fn with_facing(mut self, facing: Orientation) -> Self {
        self.facing = Some(facing);
        self
    }

    /// The place, which every location has.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The position inside the place, if the world models one.
    pub const fn local(&self) -> Option<LocalPosition> {
        self.local
    }

    /// The direction faced, if the world models one.
    pub const fn facing(&self) -> Option<Orientation> {
        self.facing
    }
}

/// Which place an action can be performed in.
///
/// A statement about the *actor's* place. Whether a target must be near the actor is the
/// `SamePlaceAsActor` case and the interaction range; whether the action can only happen somewhere
/// particular is `Specific`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceRequirement {
    /// Anywhere. Sending a message and applying for a remote job are the examples
    /// `docs/ENGINEERING_RULES.md` §7 gives of actions that require no proximity at all.
    Any,
    /// The target must be in the same place as the actor.
    SamePlaceAsActor,
    /// The actor must be in this particular place.
    Specific(PlaceId),
}

/// What an action requires of space, as a value a system declares.
///
/// `docs/ENGINEERING_RULES.md` §7 puts this requirement in the System contract and forbids a
/// renderer from deciding it independently. Making it data rather than code has a second
/// consequence that §8 needs: the requirement can be *sent to a client* — inside an
/// [`Affordance`](crate::observation::Affordance) — so a client can show what an action needs
/// without implementing the check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "SpatialRequirementFields")]
pub struct SpatialRequirement {
    place: PlaceRequirement,
    within_range: Option<Millimetres>,
    requires_line_of_access: bool,
    requires_target_available: bool,
}

impl SpatialRequirement {
    /// Requires nothing of space. The declaration for an action that works across a world: a
    /// message, a phone call, a remote application.
    pub const NONE: Self = Self {
        place: PlaceRequirement::Any,
        within_range: None,
        requires_line_of_access: false,
        requires_target_available: false,
    };

    /// Requires the target to be in the same place as the actor, and nothing more.
    pub const fn same_place() -> Self {
        Self {
            place: PlaceRequirement::SamePlaceAsActor,
            ..Self::NONE
        }
    }

    /// Requires the actor to be in one particular place.
    pub const fn at_place(place: PlaceId) -> Self {
        Self {
            place: PlaceRequirement::Specific(place),
            ..Self::NONE
        }
    }

    /// Also requires the target to be within `range` of the actor.
    ///
    /// Rejects a negative range where it is written rather than where it is evaluated: a
    /// requirement is declared once, in a system's source or its configuration, and a range of
    /// `-3 m` is a mistake that would otherwise be compared as if it were `3 m`.
    pub fn within(mut self, range: Millimetres) -> Result<Self, ContractError> {
        if range.value() < 0 {
            return Err(ContractError::NegativeInteractionRange {
                millimetres: range.value(),
            });
        }
        self.within_range = Some(range);
        Ok(self)
    }

    /// Also requires an unobstructed line of access from the actor to the target.
    ///
    /// Declared but not evaluated by this crate — see [`SpatialRequirement::evaluate`].
    #[must_use]
    pub const fn requiring_line_of_access(mut self) -> Self {
        self.requires_line_of_access = true;
        self
    }

    /// Also requires the target to be available to be acted upon: not busy, not closed, not
    /// already engaged. What availability *means* is the owning system's business; this flag only
    /// says that the answer matters.
    #[must_use]
    pub const fn requiring_target_available(mut self) -> Self {
        self.requires_target_available = true;
        self
    }

    /// Which place the action can be performed in.
    pub const fn place(&self) -> PlaceRequirement {
        self.place
    }

    /// How close the target must be, if closeness is required at all.
    pub const fn within_range(&self) -> Option<Millimetres> {
        self.within_range
    }

    /// Whether an unobstructed line of access is required.
    pub const fn requires_line_of_access(&self) -> bool {
        self.requires_line_of_access
    }

    /// Whether the target must be available.
    pub const fn requires_target_available(&self) -> bool {
        self.requires_target_available
    }

    /// Whether this requirement is met, and if not, which answer a client should be given.
    ///
    /// Pure, total and integer-only: the same inputs give the same answer on every platform and in
    /// every run, which is what lets this one implementation serve the server, a system and a
    /// replay alike. It decides nothing about permissions, ownership, relationships, schedules or
    /// system availability — those are other checks, owned elsewhere, and a `Ok(())` here means
    /// only that space does not stand in the way.
    ///
    /// # The order the checks run in is part of the contract
    ///
    /// 1. **Availability.** A target that is not available cannot be made available by walking
    ///    closer, so reporting distance first would send a player on a pointless journey.
    /// 2. **Place.** Being in the wrong place is reported as [`Rejection::TooFarAway`]: from the
    ///    actor's point of view the difference between the wrong room and the wrong town is only
    ///    how far they must travel.
    /// 3. **Range**, when the world models continuous position.
    /// 4. **Line of access** — see the gap below.
    ///
    /// # Two documented degeneracies
    ///
    /// A requirement about a target cannot be satisfied by a request that named none, and no
    /// distance was ever established, so a missing target is [`Rejection::PreconditionFailed`]
    /// rather than `TooFarAway`.
    ///
    /// A range requirement in a world that does not model continuous position degenerates to
    /// *same place* rather than to either extreme. Passing everything would let a millimetre range
    /// be satisfied from another town; failing everything would make an action declared by a
    /// 3D-capable system unusable in a purely semantic 2D world, which `DD-4` promises it must not
    /// be. Being in the same place is the finest proximity such a world can express, so that is
    /// what the requirement means there.
    ///
    /// # The line-of-access gap (`DD-7`)
    ///
    /// Line of access is a declared requirement this crate does **not** evaluate, and returns
    /// `Ok` for. Deciding whether a wall stands between two positions needs world geometry, which
    /// no layer owns yet; answering it from place identity and a distance would be a check that
    /// looks real and is not. A world that installs a geometry provider evaluates it there, and
    /// [`SpatialRequirement::requires_line_of_access`] is how that provider is told to.
    pub fn evaluate(
        &self,
        actor: &Location,
        target: Option<&Location>,
        target_available: bool,
    ) -> Result<(), Rejection> {
        if self.requires_target_available && !target_available {
            return Err(Rejection::TargetUnavailable);
        }

        match self.place {
            PlaceRequirement::Any => {}
            PlaceRequirement::SamePlaceAsActor => {
                let target = target.ok_or(Rejection::PreconditionFailed)?;
                if target.place() != actor.place() {
                    return Err(Rejection::TooFarAway);
                }
            }
            PlaceRequirement::Specific(required) => {
                if actor.place() != required {
                    return Err(Rejection::TooFarAway);
                }
            }
        }

        if let Some(range) = self.within_range {
            let target = target.ok_or(Rejection::PreconditionFailed)?;
            match (actor.local(), target.local()) {
                (Some(actor_position), Some(target_position)) => {
                    if !actor_position.within(range, target_position) {
                        return Err(Rejection::TooFarAway);
                    }
                }
                // The world does not model continuous position here: the range degenerates to the
                // finest proximity such a world can express.
                _ => {
                    if actor.place() != target.place() {
                        return Err(Rejection::TooFarAway);
                    }
                }
            }
        }

        // Line of access is declared here and evaluated by a world geometry provider that does not
        // exist yet (DD-7). Answering it from place identity would be a check that only looks real.
        Ok(())
    }
}

/// The serialized shape of a [`SpatialRequirement`], checked on the way back in so that an
/// authored or configured declaration is rejected where it is read.
#[derive(Deserialize)]
struct SpatialRequirementFields {
    place: PlaceRequirement,
    within_range: Option<Millimetres>,
    requires_line_of_access: bool,
    requires_target_available: bool,
}

impl TryFrom<SpatialRequirementFields> for SpatialRequirement {
    type Error = ContractError;

    fn try_from(value: SpatialRequirementFields) -> Result<Self, Self::Error> {
        let requirement = Self {
            place: value.place,
            within_range: None,
            requires_line_of_access: value.requires_line_of_access,
            requires_target_available: value.requires_target_available,
        };
        match value.within_range {
            Some(range) => requirement.within(range),
            None => Ok(requirement),
        }
    }
}
