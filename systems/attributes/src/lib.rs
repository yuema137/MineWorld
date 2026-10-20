//! A Person's attributes (`docs/DECISIONS.md` `ARC-120` … `ARC-124`; `docs/MODULE_SPEC.md` §§3.1, 4.1).
//!
//! A **System Pack**.
//!
//! ```text
//! data        attributes/, vocabularies/   decoded with line and column; one frozen Catalogue
//! context     attributes-composed          the catalogue, stated once at genesis (ARC-121 item 5)
//! section     attributes                   values and restrictions, decoded against the catalogue
//! resolution  every person: authored, else the declaration's default; an owner's values handed to it
//! owns        Traits                       plain stored traits only — never derived, never an owner's
//! reacts      traits-stated, trait-changed → Traits; attribute-change-proposed → checked
//! discloses   traits                       through the model crate's one filter and renderer
//! depends on  nothing
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod data;
mod section;
pub mod system;

pub use system::AttributesSystem;
