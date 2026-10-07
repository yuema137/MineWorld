//! Jobs, and work as being there.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`). A job names an employer, a workplace, a shift and
//! a wage. Work is **attendance**: a person present at the workplace during the shift is working, and
//! no controller needs to know that work exists — people already walk to their agenda's place
//! (`ARC-32`), and a human player works by being there.
//!
//! ```text
//! section     job                  a person file's `job: { employer, workplace, from, until, wage,
//!                                  produces }`
//! emits       hired                at genesis, from the section; biographical
//!             shift-started        at the shift's start: whether the employee is there
//!             shift-ended          at its end: the seconds worked
//!             wage-due             at its end, when anything was worked — economy pays it (ARC-28)
//!             items-produced       inventory's fact, for the employer, through `produce` (ARC-26)
//! owns        Employment           on the employee; the `employed-by` edge (Person → Organization);
//!                                  one `shift` Process per job
//! hears       person-entered-place presence's: arriving at or leaving the workplace mid-shift
//! discloses   Employment           to the employee only
//! depends on  inventory, presence
//! ```
//!
//! **It never touches money.** It states that a wage is due; `economy` alone decides whether it is
//! paid and moves it. This pack has no write token for a wallet (`INV-7`) and does not name economy.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;

pub mod component;
pub mod error;
pub mod event;
pub mod process;
pub mod system;

pub use component::{Employment, Job, OnShift, Produces};
pub use error::EmploymentError;
pub use event::{Hired, ShiftEnded, ShiftStarted, WageDue};
pub use process::{ShiftProcess, ShiftState};
pub use section::AuthoredJob;
pub use system::{BIOGRAPHICAL, EmploymentSystem, employed_by, employed_by_declaration};
