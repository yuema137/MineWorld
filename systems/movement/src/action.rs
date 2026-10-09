//! The action this pack provides, and the one number a client benefits from knowing.

use mineworld_contracts::{
    Action, ActionTypeId, Location, Millimetres, SpatialRequirement, SystemId,
};
use mineworld_kernel::SystemIdentity;
use serde::{Deserialize, Serialize};

use crate::system::MovementSystem;

/// The furthest one `move` may carry a person from their authoritative position: two metres.
///
/// Millimetres, as an integer, because positions reach the event log (`AC-12`). What it is: a bound
/// on **one request**. A person walking or jogging who reports before travelling this far since
/// its last accepted position is never refused (`server/PROTOCOL.md` §6.2, the reporting rule);
/// a client that reports a five-metre jump is.
///
/// What it is **not**: a speed limit. It bounds one request, not requests per second — a client that
/// sends strides back to back moves as fast as it sends them (`DECISIONS.md` `ARC-26`, limitation
/// L-1). Nor is it about walls: where a place has walls and furniture, the arrival resolver the world
/// installs stops the stride at them before the arrival is recorded (`ARC-39`); this pack decides only
/// the stride's length and the passage. It is this pack's policy, not a law of nature, and becomes
/// world configuration with S7.
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
