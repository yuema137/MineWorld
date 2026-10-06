//! Whether a person may walk where they ask, and which places open onto which.
//!
//! A **System Pack**. A world that installs it has walking in it; a world that does not answers
//! `move` with `ActionUnavailable` and is otherwise the same world (`AC-2`).
//!
//! ```text
//! owns        Passages            which places a place opens onto, and where the doorway is
//! provides    move                a stride within a place, or through a doorway into the next
//! emits       passage-opened      the genesis fact that gives places their Passages
//!             arrived             presence's fact, built by presence's `arrival` (ARC-26)
//! subscribes  passage-opened      so Passages is a projection of the log
//! depends on  presence            whose Presence says where a person is moving from
//! ```
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
//!                  constructor; writes only Passages
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

pub use action::{MAX_STRIDE, Move, move_offer_requirement, stride_requirement};
pub use component::{Passage, Passages};
pub use event::{PassageOpened, passage};
pub use system::MovementSystem;
