//! Where people are, what one of them perceives, and what the server says they may attempt.
//!
//! A **System Pack**, not kernel code. Everything here is domain vocabulary — `presence`,
//! `arrived`, `present-in` — which is exactly what `INV-12` puts *above* the kernel and forbids
//! below it. A world that installs this pack has locations and perception in it; a world that
//! does not is still a valid world, with neither.
//!
//! ```text
//! owns        Presence               where a person is, semantically (INV-5)
//!             present-in             the edge that says which place a person is in
//! provides    nothing                who may move a person is another system's decision (ARC-26)
//! emits       arrived                somebody is now somewhere — at genesis, through `arrival`
//!             person-entered-place   somebody is now in another place: an occupancy change
//! vocabulary  stopped-short          somebody's arrival ended short of where it was asked to go —
//!                                    stated by a system that moves people, through `arrivals`;
//!                                    never stated or reduced here
//! subscribes  arrived                whoever stated it, which is how the state above is a
//!                                    projection of the log
//! ```
//!
//! # Owning where people are, and not deciding it
//!
//! A world places its people by genesis and lets another system decide where they go. That system
//! depends on this one, states this pack's `arrived` through [`arrivals`] or [`arrival`] — the
//! checked constructors — and this pack reduces it, exactly as it reduces a genesis placement. This
//! pack still decides what its state may hold: [`admit`] refuses a value it may not take, before the
//! fact is built and again when it is reduced (`DECISIONS.md` `ARC-26`).
//!
//! # Asking before recording
//!
//! Whether a person actually gets where they were sent is asked of every registered
//! [`ArrivalResolver`] before anything is recorded ([`resolve`], `DECISIONS.md` `ARC-39`): a pack
//! that knows something this pack does not may end the arrival short or move other people out of the
//! way, and this pack checks each answer as the owner and records only the result. With no resolver
//! registered — every world until a resolver's pack is installed — the constructors build exactly
//! the facts they built before the seam existed.
//!
//! # The two halves of this crate
//!
//! [`PresenceSystem`] is the installable half: it declares, installs and reduces, and the only state
//! it writes is its own. [`observe()`] is the query half — perception is a *read*, not an action,
//! so it is a function over a composed [`World`](mineworld_kernel::World) rather than a method on
//! the trait, and it produces the [`Observation`](mineworld_contracts::Observation) that `INV-13` requires a
//! controller to be given instead of the world.
//!
//! # Why this crate cannot name another pack's action
//!
//! An observation carries **affordances**: what this observer may attempt, each with the server's
//! verdict, so that no client ever computes availability
//! (`docs/ENGINEERING_RULES.md` §§7–9). The tempting implementation is a match on action names,
//! and it would make every future System Pack a change to this file — the change-amplification
//! failure `docs/ENGINEERING_STANDARDS.md` §8 exists to catch.
//!
//! So the question is asked rather than answered here. Each pack implements
//! [`PerceptionProvider`] and hands back [`Offer`]s: an action type read off its own
//! [`Action`](mineworld_contracts::Action) type, the
//! [`SpatialRequirement`](mineworld_contracts::SpatialRequirement) it declares, and the one
//! judgement only its owner can make — whether the target is available. This crate then does two
//! things with each offer, and neither involves knowing what the action is:
//!
//! ```text
//! is it provided here?   the kernel's route map answers — a disabled pack offers nothing (AC-2)
//! is it possible now?    SpatialRequirement::evaluate answers — one evaluator, every system
//! ```
//!
//! No action name belonging to another pack appears anywhere in these sources — that absence *is*
//! the architectural claim, so `tests/presence.rs` checks it structurally rather than trusting a
//! reviewer to notice the day it stops being true.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod component;
pub mod event;
pub mod interaction;
pub mod observe;
pub mod resolve;
pub mod system;

pub use component::Presence;
pub use event::{Arrived, PersonEnteredPlace, StoppedShort, admit, arrival, arrivals};
pub use interaction::{Offer, PerceptionProvider};
pub use observe::observe;
pub use resolve::{
    ArrivalResolver, Arriving, Resolution, register_resolvers, registered_resolvers,
    require_registered,
};
pub use system::{PresenceSystem, present_in, present_in_declaration};
