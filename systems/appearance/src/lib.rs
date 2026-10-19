//! What people look like.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-31`, `ARC-91`). A look is component state, a component
//! is owned by a system, and until this pack no system owned one — so no client knew how a person
//! should be drawn, and each invented a figure from a hash. This pack owns it:
//!
//! ```text
//! section     appearance           a person file's `appearance: { look: alice-moreau }`, validated by [`Appearance`]
//! emits       appeared             at genesis, from the section; public
//! owns        Look                 `look`, reduced from `appeared` by this pack alone
//! discloses   Look                 to every observer who perceives the person — what someone looks like is visible
//! depends on  nothing
//! ```
//!
//! A look is a key, not art. The world says that Alice has the appearance called `alice-moreau`; a
//! Presentation Pack says what that looks like (`ARC-92`). A look is never a name, and no client shows
//! it as text.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod component;
pub mod event;
pub mod look;
pub mod system;

pub use component::Look;
pub use event::Appeared;
pub use look::{InvalidLook, LOOK_MAX_BYTES, LookKey};
pub use section::Appearance;
pub use system::{AppearanceSystem, BIOGRAPHICAL};
