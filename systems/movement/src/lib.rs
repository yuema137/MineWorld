//! Whether a person may walk where they ask, and which places open onto which.
//!
//! A **System Pack**. A world that installs it has walking in it; a world that does not answers
//! `move` with `ActionUnavailable` and is otherwise the same world (`AC-2`).
//!
//! ```text
//! owns        Passages            which places a place opens onto, and where the doorway is
//!             Walking             a walk in progress: where to, and the route the world planned
//! provides    move                a stride within a place, or through a doorway into the next
//!             walk-to             go somewhere: plan the route, record the walk (ARC-75)
//!             walk-step           the next stride of one's own walk — an embodied request
//! emits       passage-opened      the genesis fact that gives places their Passages
//!             arrived             presence's fact, built by presence's `arrivals` (ARC-26)
//!             stopped-short       presence's too, when a resolver ends a stride short (ARC-39)
//!             walk-started        a walk was recorded
//!             walk-ended          and ended: arrived, stalled, no-route, replaced or stopped
//! subscribes  passage-opened      so Passages is a projection of the log
//!             arrived             a walker recorded at the walk's end ends the walk
//!             stopped-short       who stopped a walker, to plan round them
//! catalog     Wayfinder           the packs that plan routes through places they know (ARC-62)
//! depends on  presence            whose Presence says where a person is moving from
//! ```
//!
//! # Walking somewhere
//!
//! A walk is this pack's state, its route is the geometry owner's answer, and its strides are
//! embodied requests (`DECISIONS.md` `ARC-75`). `walk-to` plans the first leg — asking the registered
//! [`Wayfinder`]s, or going straight when none answers — and records [`Walking`]; each `walk-step`
//! takes at most [`WALK_STRIDE`] along it, checked by the same rule as `move` and stated through
//! presence's `arrivals`, so a walk is never a way round that rule. A walk takes no calendar time:
//! its pace is how often its sender asks for steps, and nothing here reads a clock or a time scale.
//!
//! # Deciding, not recording
//!
//! The split with presence is by responsibility (`DECISIONS.md` `ARC-26`):
//!
//! ```text
//! PresenceSystem   owns where people are — Presence and present-in — and writes it only by
//!                  reducing `arrived`, whoever stated it; it can refuse a value its state may not
//!                  hold, and it never learns this pack exists
//! MovementSystem   decides whether a move is possible; states `arrived` through presence's
//!                  constructor; writes only Passages and Walking
//! ```
//!
//! So there is one spatial truth, and the dependency points one way: `movement → presence`. A
//! later travel system — a `Process` that takes simulated time between places that do not adjoin —
//! ends in the same `arrived`, reduced by the same owner, which is how walking across a room and
//! travelling between towns stay two scales of one spatial model (`ENGINEERING_RULES.md` §6).
//!
//! # What `move` checks
//!
//! Every distance is decided by the contract layer's one evaluator,
//! [`SpatialRequirement::evaluate`](mineworld_contracts::SpatialRequirement::evaluate), with the
//! destination or a doorway as the target: within a place, at most [`MAX_STRIDE`] from where the
//! person is; into another place, only through a [`Passage`], within a stride of the doorway on
//! both sides. A refusal is `TooFarAway`, decided here, on the server. [`MAX_STRIDE`] bounds a
//! request and is not a speed limit; walls inside a place are not evaluated (`DD-7`). Both are
//! stated limitations (`ARC-26`), not approximations.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod action;
pub mod component;
pub mod event;
pub mod system;
mod walk;
pub mod wayfinder;

pub use action::{
    Destination, MAX_STRIDE, Move, PERSON_APPROACH, REPLANS_MAX, STALL_PROGRESS, STALLS_MAX,
    TARGET_MOVED, WALK_STRIDE, WalkStep, WalkTo, move_offer_requirement, stride_requirement,
};
pub use component::{Passage, Passages, Walking};
pub use event::{Ended, PassageOpened, WalkEnded, WalkStarted, passage};
pub use system::{MovementSystem, is_walking};
pub use walk::NO_ROUTE;
pub use wayfinder::{
    RouteAnswer, RouteAsk, Wayfinder, Waypoints, register_wayfinders, registered_wayfinders,
    require_wayfinder,
};
