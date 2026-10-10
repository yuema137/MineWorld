//! The actions this pack provides — `move`, `walk-to` and `walk-step` — and the numbers a client
//! benefits from knowing.

use mineworld_contracts::{
    Action, ActionTypeId, Location, Millimetres, PersonId, SpatialRequirement, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::MovementSystem;

/// The furthest one `move` may carry a person from their authoritative position: two metres.
///
/// Millimetres, as an integer, because positions reach the event log (`AC-12`). What it is: a bound
/// on **one request**. A body walking or jogging that reports before it has travelled this far since
/// its last accepted position is never refused (`server/PROTOCOL.md` §6.2, the reporting rule);
/// a client that reports a five-metre jump is.
///
/// What it is **not**: a speed limit. It bounds one request, not requests per second — a client that
/// sends strides back to back moves as fast as it sends them (`DECISIONS.md` `ARC-26`, limitation
/// L-1). Nor does it see walls: a stride may pass through a table, because line of access needs
/// geometry no layer owns yet (`DD-7`, L-2). It is this pack's policy rather than a physical
/// constant, and becomes world configuration with S7.
pub const MAX_STRIDE: Millimetres = Millimetres::new(2_000);

/// A person asks to be somewhere else: a stride within the place they are in, or through a doorway
/// into a place that opens onto it.
///
/// The payload is only the destination. Who is moving is the
/// [`ActionIntent`](mineworld_contracts::ActionIntent)'s actor; where they are moving *from* is the
/// world's authoritative record, never the client's report (`ENGINEERING_RULES.md` §8). A turn on the
/// spot is a `move` to the same position with a new facing, because [`Location`] carries orientation.
///
/// This is what `ENGINEERING_RULES.md` §6 calls a *MoveIntent*: a request, refusable, distinct from
/// the authoritative position it may change, from a travel `Process` (not built), and from whatever a
/// renderer draws between two positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Move {
    to: Location,
}

impl Action for Move {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("move");
    const OWNER: SystemId = MovementSystem::ID;
}

impl Move {
    /// Asks to move to `to`.
    pub const fn new(to: Location) -> Self {
        Self { to }
    }

    /// Where the actor is asking to be.
    pub const fn to(&self) -> Location {
        self.to
    }
}

/// What one stride requires of the space between where a person is and where they ask to be: the
/// same place, within [`MAX_STRIDE`].
///
/// Evaluated with the destination — or a doorway — in the *target* position, so that every distance
/// this pack decides is decided by the one evaluator in the contract layer,
/// [`SpatialRequirement::evaluate`], with its degeneracies: in a world that models no continuous
/// position the range becomes *same place*, so a semantic world moves freely within a place.
pub fn stride_requirement() -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(MAX_STRIDE)
        .expect("a positive stride")
}

/// What `move` is offered with: nothing required of a *target*, because it has none.
///
/// An offer's requirement is evaluated against the offer's target (`presence`'s `observe`), and a
/// `move` is offered once, against nobody. Offering it with [`stride_requirement`] would show it as
/// impossible to everyone; the stride is instead a published constant, [`MAX_STRIDE`], and the
/// answer to any one destination is the server's, given when it is asked.
pub const fn move_offer_requirement() -> SpatialRequirement {
    SpatialRequirement::NONE
}

/// The most one `walk-step` carries a walker along their route: 1 340 mm (`DECISIONS.md` `ARC-75`).
///
/// A distance, not a duration: no rule here knows how often steps arrive. Sent once a wall second —
/// the senders' cadence, never this pack's — it is 1.34 m/s, the mean free walking speed of pedestrians
/// (Weidmann 1993; Bohannon 1997 measured 1.27–1.46 m/s comfortable gait for adults aged 20–59). Less
/// than [`MAX_STRIDE`], so every stride of a walk is a stride `move` would also allow.
pub const WALK_STRIDE: Millimetres = Millimetres::new(1_340);

/// How near a walk to a person ends, centre to centre: 1 200 mm, the far limit of personal distance
/// (Hall 1966, 0.46–1.22 m) — near enough to talk, not on top of them.
pub const PERSON_APPROACH: Millimetres = Millimetres::new(1_200);

/// Consecutive steps with less than [`STALL_PROGRESS`] of progress after which a walk ends `stalled`.
pub const STALLS_MAX: u32 = 3;

/// What counts as progress for [`STALLS_MAX`]: 50 mm between one step and the next.
pub const STALL_PROGRESS: Millimetres = Millimetres::new(50);

/// Re-plans one walk may make — after a stopped or displaced stride, or a person destination that moved
/// — before it ends `stalled`.
pub const REPLANS_MAX: u32 = 8;

/// How far a person destination may move from where its leg was planned to end before the walk
/// re-plans: 500 mm.
pub const TARGET_MOVED: Millimetres = Millimetres::new(500);

/// Where a walk goes (`DECISIONS.md` `ARC-75`): a location, or a person.
///
/// Open to extension (step-11 note N-1): a later arm — a region, a remote place the travel system
/// reaches, an object — is an addition, not a breaking change to `walk-to`'s payload or to a `match` in
/// another crate. Serialized with one explicit tag per arm: `{ "place": <Location> }` or
/// `{ "person": <PersonId> }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Destination {
    /// A point in the walker's place or in a place a chain of passages reaches; a location without a
    /// position means "enter that place".
    Place(Location),
    /// Within [`PERSON_APPROACH`] of a person in the walker's own place, following them if they move.
    Person(PersonId),
}

/// A person asks to walk somewhere: the server plans the route and keeps it as the walker's
/// [`Walking`](crate::Walking); nothing moves until the walker asks for a [`WalkStep`].
///
/// One request for "go to X", from every caller — a client's click, a menu entry, a controller — so
/// that no client or controller computes a route (`ENGINEERING_RULES.md` §§8–9).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkTo {
    to: Destination,
}

impl Action for WalkTo {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("walk-to");
    const OWNER: SystemId = MovementSystem::ID;
}

impl WalkTo {
    /// Asks to walk to `to`.
    pub const fn new(to: Destination) -> Self {
        Self { to }
    }

    /// Where the walk goes.
    pub const fn to(&self) -> Destination {
        self.to
    }
}

/// The next stride of the actor's own walk: at most [`WALK_STRIDE`] along its route, or the crossing
/// at a doorway. No payload — the walk is the world's, not the request's.
///
/// An embodied input: its sender paces it (a client once a wall second, a host at its cadence), and
/// the walk takes exactly as many requests as it has strides (`DECISIONS.md` `ARC-67`, `ARC-75`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalkStep {}

impl Action for WalkStep {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("walk-step");
    const OWNER: SystemId = MovementSystem::ID;
}
