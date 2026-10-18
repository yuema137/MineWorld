//! A person's character: how they are, and how they speak.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-74`).
//!
//! ```text
//! section     persona              a person file's `persona: { summary, traits, speech }`, validated by [`Persona`]
//! emits       persona-authored     at genesis, from the section; private to the person it describes
//! owns        PersonaComponent     `persona`, reduced from `persona-authored` by this pack alone
//! discloses   PersonaComponent     to its holder only
//! depends on  nothing
//! ```
//!
//! A persona shapes speech and grants nothing. No action, process or reaction reads it, and a rule
//! controller ignores it; a controller that speaks for the person may read it through the person's
//! own observation (`ARC-74`, `R-S10-9`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod event;
mod section;

pub mod component;
pub mod persona;
pub mod system;

pub use component::PersonaComponent;
pub use event::PersonaAuthored;
pub use persona::{
    Persona, PersonaError, SPEECH_MAX_BYTES, SUMMARY_MAX_BYTES, TRAIT_MAX_BYTES, TRAITS_MAX,
};
pub use system::{BIOGRAPHICAL, PersonaSystem};
