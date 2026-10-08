//! The actions this pack provides — `kick`, `throw` and `shove` — and what each requires of space
//! (step-11 §8.1, SD-O11 … SD-O15).
//!
//! `kick` and `throw` are **target-less, with the object in the payload**, as `buy` is (step-11 F-O1,
//! F-O6): perception neither lists nor locates an Item, so an object cannot be a request's target.
//! Each declares `same_place().within(reach)`, which `validate` evaluates against the object's ground
//! point — a position only this pack knows. `shove` targets a person.

use mineworld_contracts::{
    Action, ActionTypeId, ItemId, Millimetres, SpatialRequirement, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::geometry::{KICK_REACH, SHOVE_REACH, THROW_REACH};
use crate::system::BodiesSystem;

/// Kick `object`, lying on the floor within reach, away from the kicker (step-11 SD-O11). No aim: a
/// player aims a kick by where they stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kick {
    object: ItemId,
}

impl Action for Kick {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("kick");
    const OWNER: SystemId = BodiesSystem::ID;
}

impl Kick {
    /// Kick `object`.
    pub const fn new(object: ItemId) -> Self {
        Self { object }
    }

    /// Which object.
    pub const fn object(&self) -> ItemId {
        self.object
    }
}

/// A point on the floor of the thrower's place, in its frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Toward {
    x: Millimetres,
    y: Millimetres,
}

impl Toward {
    /// The point `(x, y)`.
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
}

/// Pick `object` up and throw it, in one action (step-11 SD-O13, QO-11), toward a point — or, with
/// `toward: null`, the complete form, along the thrower → object line, 3 m beyond the object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Throw {
    object: ItemId,
    toward: Option<Toward>,
}

impl Action for Throw {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("throw");
    const OWNER: SystemId = BodiesSystem::ID;
}

impl Throw {
    /// Throw `object` toward `toward`, or along the default line with [`None`].
    pub const fn new(object: ItemId, toward: Option<Toward>) -> Self {
        Self { object, toward }
    }

    /// Which object.
    pub const fn object(&self) -> ItemId {
        self.object
    }

    /// Where to, when aimed.
    pub const fn toward(&self) -> Option<Toward> {
        self.toward
    }
}

/// Shove the target person away, 500 mm (step-11 SD-O15; QB-10): the deliberate, larger
/// displacement. No payload of its own: whom is the request's target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shove {}

impl Action for Shove {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("shove");
    const OWNER: SystemId = BodiesSystem::ID;
}

/// What a kick requires of space: the object in the kicker's place, its ground point within 800 mm
/// of the kicker's centre, and the object kickable (lying on the floor).
pub fn kick_requirement() -> SpatialRequirement {
    within(KICK_REACH)
}

/// What a throw requires of space: as a kick, within 800 mm, the object lying anywhere.
pub fn throw_requirement() -> SpatialRequirement {
    within(THROW_REACH)
}

/// What a shove requires of space: the target in the shover's place, within 1 000 mm, with a body.
pub fn shove_requirement() -> SpatialRequirement {
    within(SHOVE_REACH)
}

fn within(reach: Millimetres) -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(reach)
        .expect("a positive reach")
        .requiring_target_available()
}
