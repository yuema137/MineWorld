//! What people are called.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-31`). A name is component state, a component is owned
//! by a system, and until this pack no system owned one — so no client ever showed a name, and a
//! controller could only say "person 4". This pack owns it:
//!
//! ```text
//! section     name                 a person file's `name: Alice Moreau`, validated by [`Name`]
//! emits       named                at genesis, from the section; public
//! owns        DisplayName          `display-name`, reduced from `named` by this pack alone
//! discloses   DisplayName          to every observer who perceives the person — names are public
//! depends on  nothing
//! ```
//!
//! It is called `naming` and not `identity`: identity is the kernel's word, for `EntityId`.
//!
//! Gating a name on acquaintance — strangers stay nameless — would couple this pack to relationships'
//! state; it is a later option, recorded in `ARC-31`, and so are names for places.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod component;
pub mod event;
pub mod name;
pub mod system;

pub use component::DisplayName;
pub use event::Named;
pub use name::{InvalidName, NAME_MAX_BYTES, Name};
pub use system::{BIOGRAPHICAL, NamingSystem, names_in};
