//! Knowing somebody, and how well — changed only because something happened between two people.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-28`). It provides no action and runs no process: its
//! state changes only by reducing facts other packs record. That is `CLAUDE.md` §4 rule 1 in its
//! plainest form — conversation states that somebody spoke, group-activity that an invitation was
//! answered or an activity ended, and this pack, the owner of relationship state, reacts.
//!
//! ```text
//! declares    knows                    a directed edge, Person → Person
//! owns        Acquaintances            on the `from` Person: per counterpart, the values below
//! subscribes  spoke                    conversation's: familiarity +10 both ways, an exchange counted
//!             invitation-accepted      group-activity's: regard +50 both ways
//!             invitation-declined      group-activity's: the inviter's regard for the decliner −30
//!             group-activity-ended     group-activity's: per ordered pair of members, familiarity +50,
//!                                      regard +20, an activity shared
//! emits       became-acquainted        when an edge forms, once per formation
//!             relationship-changed     when a level is crossed, up or down — not per value change
//!             acquaintance-forgotten   when a new acquaintance takes a person past known_at_most and
//!                                      the weakest tie is evicted, its edge removed
//! depends on  nothing                  subscribing is not emitting (ARC-26); a world without
//!                                      conversation has relationships that hear no speech
//! section     configure/relationships.yaml   the rule `acquaint`, the five increments above, decay and
//!                                      the cap (`interactions`, `ARC-63`, `ARC-86`); compiled
//!                                      defaults when unconfigured
//! catalog     DecayCurve               user curves, on this pack's ARC-62 extension line
//! ```
//!
//! # Decoding another pack's fact
//!
//! Through the owner's published type and [`EventRecord::payload_for`] — a Cargo dependency on the
//! vocabulary, never a registry dependency, and never a local struct shaped like another pack's
//! payload, which would drift silently and skip the schema-version refusals (`step-09-social.md` Q6).
//!
//! # Forgetting, by configuration
//!
//! Without configuration nothing fades: a pair that keeps meeting reaches its top level within weeks
//! and stays there (`ARC-28`'s saturation). A world may make familiarity and regard fade by a named,
//! pure, integer [`DecayCurve`] with time since the last contact, read lazily wherever a value is read
//! — no Process, no timer, no fact — and bound how many people a person keeps (`ARC-86`; the
//! [`decay`] module, `README.md`). A curve of one's own is a pack on this pack's `ARC-62` extension
//! line ([`register_curves`]).
//!
//! [`EventRecord::payload_for`]: mineworld_contracts::EventRecord::payload_for

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod component;
pub mod decay;
pub mod event;
pub mod interactions;
pub mod level;
pub mod system;

pub use component::{Acquaintance, Acquaintances, RelationshipValues};
pub use decay::{CurveCatalog, CurveName, DecayCurve, register_curves, registered_curves};
pub use event::{AcquaintanceForgotten, BecameAcquainted, RelationshipChanged};
pub use level::{Level, level};
pub use system::{
    ACCEPTED_REGARD, ACTIVITY_FAMILIARITY, ACTIVITY_REGARD, BIOGRAPHICAL, DECLINED_REGARD,
    RelationshipsSystem, SPOKE_FAMILIARITY, current, knows, knows_declaration,
};
