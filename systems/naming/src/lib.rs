//! What people and places are called.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-31`, `ARC-363`). A name is component state, a
//! component is owned by a system, and until this pack no system owned one — so no client ever showed
//! a name, and a controller could only say "person 4". This pack owns it:
//!
//! ```text
//! section     name                 a person's or place's `name: Alice Moreau`, or
//!                                  `name: { default: …, forms: { zh_Hans: … } }`, validated by [`NameSpec`]
//! emits       named, place-named   at genesis, from the section; public
//! owns        DisplayName          `display-name`, reduced from both facts by this pack alone
//! discloses   DisplayName          to every observer who perceives the subject — names are public
//!             DoorNames            `door-names`: about a place, the name of each place its doorways lead
//!                                  to, to whoever perceives it (the audience of movement's `to_tags`)
//! reads       Passages             movement's public component, when the world has it
//! ```
//!
//! It is called `naming` and not `identity`: identity is the kernel's word, for `EntityId`.
//!
//! Gating a name on acquaintance — strangers stay nameless — would couple this pack to relationships'
//! state; it is a later option, recorded in `ARC-31`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod component;
pub mod door;
pub mod event;
pub mod name;
pub mod system;

pub use component::DisplayName;
pub use door::{DoorName, DoorNames};
pub use event::{Named, PlaceNamed};
pub use name::{
    FORMS_MAX, Forms, InvalidLocale, InvalidName, Locale, NAME_MAX_BYTES, Name, NameSpec,
};
pub use system::{BIOGRAPHICAL, NamingSystem, names_in};
