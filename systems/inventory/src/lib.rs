//! What people and organizations hold.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-37`). An authored Item is a kind (`ARC-36`), so what
//! anybody holds is a count of each kind. This pack owns those counts, and it is the **only** writer
//! of them: a pack that decides a give, a purchase, a shift's production or a meal states this pack's
//! fact through its checked constructor, and this pack still decides (`ARC-26`, `ARC-38`):
//!
//! ```text
//! section     holdings             a person's or organization's `holdings: { coffee: 2 }`
//! emits       stocked              at genesis, from the section; visible to the holder
//!             items-transferred    stated by a deciding pack through [`transfer`]; visible to both
//!             items-produced       stated by a producing pack through [`produce`]; to the holder
//!             items-consumed       stated by a consuming pack through [`consume`]; to the holder
//! owns        Holdings             `holdings`, reduced from these four facts by this pack alone
//! discloses   Holdings             to the holder only (INV-13)
//! depends on  item                 a kind must be declared before anybody holds it
//! ```
//!
//! [`admit_transfer`] is the whole of what holdings refuse about a transfer, asked by a deciding pack's
//! `validate`, by [`transfer`], and again at reduction — one function, so the three cannot disagree. A
//! person carries at most [`PERSON_CAPACITY`] items, all kinds together; an organization is not
//! bounded ([`can_take`]).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod admit;
pub mod component;
pub mod event;
pub mod system;

pub use admit::{
    PERSON_CAPACITY, admit_consumption, admit_production, admit_transfer, can_take, consume,
    produce, transfer,
};
pub use component::{Held, Holdings};
pub use event::{ItemsConsumed, ItemsProduced, ItemsTransferred, Stocked};
pub use section::AuthoredHoldings;
pub use system::InventorySystem;
