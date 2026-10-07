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
//! emits       became-acquainted        when an edge first forms, once per direction
//!             relationship-changed     when a level is crossed, up or down — not per value change
//! depends on  nothing                  subscribing is not emitting (ARC-26); a world without
//!                                      conversation has relationships that hear no speech
//! ```
//!
//! # Decoding another pack's fact
//!
//! Through the owner's published type and [`EventRecord::payload_for`] — a Cargo dependency on the
//! vocabulary, never a registry dependency, and never a local struct shaped like another pack's
//! payload, which would drift silently and skip the schema-version refusals (`step-09-social.md` Q6).
//!
//! # Values never decay
//!
//! A pair that keeps meeting reaches its top level within weeks and stays there: a long world's social
//! graph saturates. That is a known living-world gap (`ARC-28`, `step-09-social.md` QB-2), not a
//! property this pack hides.
//!
//! [`EventRecord::payload_for`]: mineworld_contracts::EventRecord::payload_for

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;

pub mod component;
pub mod event;
pub mod level;
pub mod system;

pub use component::{Acquaintance, Acquaintances, RelationshipValues};
pub use event::{BecameAcquainted, RelationshipChanged};
pub use level::{Level, level};
pub use system::{
    ACCEPTED_REGARD, ACTIVITY_FAMILIARITY, ACTIVITY_REGARD, BIOGRAPHICAL, DECLINED_REGARD,
    RelationshipsSystem, SPOKE_FAMILIARITY, knows, knows_declaration,
};
