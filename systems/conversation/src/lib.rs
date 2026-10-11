//! Speaking to somebody, and remembering that they spoke to you.
//!
//! A **System Pack**. A world that installs it has conversation in it; a world that does not has
//! people who cannot speak, and is not broken — it is a different world. That sentence is the whole
//! project's claim, and `tests/conversation_and_presence.rs` is where this pack demonstrates it
//! rather than asserting it.
//!
//! ```text
//! owns        ConversationHistory   who spoke to this person, when, and about what
//! provides    talk                  one person speaks to another, in reach of being heard
//! emits       conversation-started  the first exchange after a silence
//!             spoke                 every exchange
//! subscribes  spoke                 which is how the history above is a projection of the log
//! depends on  presence              because space is state that pack owns, not this one
//! ```
//!
//! # `ConversationHistory` is a projection, and must not grow into a memory system
//!
//! `docs/MVP.md` §9.2 is explicit: Alice remembering *"somebody spoke to me three minutes ago"* is
//! satisfied by a projection of the objective event log, and building episodic memory,
//! summarization, retrieval or forgetting first would be building the wrong thing first. So the
//! component here is a **bounded list, written only while reducing the pack's own `spoke` fact**,
//! and every sentence of its documentation is about keeping it that.
//!
//! What that buys is not modesty for its own sake. Because the history is a reduction of the log,
//! a world replayed from its events produces the same memories — which is the property a cognition
//! architecture built on top of it will need, and the property a hand-maintained cache would not
//! have.
//!
//! # Space is checked, and not by this pack's own arithmetic
//!
//! `talk` declares a [`SpatialRequirement`](mineworld_contracts::SpatialRequirement): the same
//! place, and within reach. It is *evaluated* by
//! [`SpatialRequirement::evaluate`](mineworld_contracts::SpatialRequirement::evaluate) — the one
//! implementation in the contract layer — and never by a distance check written here. That is the
//! difference between systems that agree about space and systems that each round differently
//! (`ENGINEERING_RULES.md` §§7–9), and it is also what lets the same value be *sent to a client* as
//! part of an affordance, so no renderer ever computes availability.
//!
//! The positions it is evaluated against are [`Presence`](mineworld_presence::Presence)'s, which
//! this pack reads and cannot write (`INV-7`). Hence the declared dependency: a world with
//! conversation and no presence would be a world where nobody has a location to be in reach of, and
//! the registry refuses to compose it rather than discovering it at run time.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod action;
pub mod component;
pub mod error;
pub mod event;
pub mod interactions;
pub mod system;
pub mod utterance;

pub use action::{INTERACTION_RANGE, Talk, talk_requirement, talk_requirement_within};
pub use component::{ConversationHistory, Heard, REMEMBERED_AT_MOST};
pub use error::ConversationError;
pub use event::{ConversationStarted, Spoke};
pub use system::{CONVERSATION_GAP, ConversationSystem};
pub use utterance::{UTTERANCE_MAX_BYTES, Utterance};
