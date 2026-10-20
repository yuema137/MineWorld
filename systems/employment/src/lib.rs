//! Jobs, and work as being there.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-38`). A job names an employer, a workplace, a shift and
//! a wage. Work is **attendance**: a person present at the workplace during the shift is working, and
//! no controller needs to know that work exists — people already walk to their agenda's place
//! (`ARC-32`), and a human player works by being there.
//!
//! ```text
//! section     job                  a person file's `job: { employer, workplace, from, until, wage,
//!                                  produces, manages }`
//! configure   employment           the Interaction List section: lateness patience and the optional
//!                                  wage deduction (ARC-63, ARC-287)
//! emits       hired                at genesis, from the section; biographical
//!             shift-started        at the shift's start: whether the employee is there
//!             shift-ended          at its end: the seconds worked
//!             wage-due             at its end, when anything was worked — economy pays it (ARC-28)
//!             late-for-shift       the first arrival past the grace, or absence at the end; to the
//!                                  employee and the manager; biographical (ARC-287)
//!             wage-deduction-due   deduction on: at once without a manager, or from an accepted
//!                                  `penalize` — economy answers it
//!             items-produced       inventory's fact, for the employer, through `produce` (ARC-26)
//! provides    penalize             a manager's, offered complete per open lateness, up to its bound
//! owns        Employment           on the employee; the `employed-by` edge (Person → Organization);
//!                                  one `shift` Process per job
//! hears       person-entered-place presence's: arriving at or leaving the workplace mid-shift
//! discloses   Employment           to the employee only; a manager's carries `late_staff`
//! depends on  inventory, presence
//! ```
//!
//! **It never touches money.** It states that a wage, or a deduction, is due; `economy` alone decides
//! whether it is paid and moves it. This pack has no write token for a wallet (`INV-7`) and does not
//! name economy.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod lateness;
mod section;

pub mod action;
pub mod component;
pub mod error;
pub mod event;
pub mod interactions;
pub mod process;
pub mod system;

pub use action::{PENALIZE_REQUIREMENT, Penalize};
pub use component::{
    DisclosedEmployment, Employment, Job, LATE_KEPT, LateStaff, Level, OnShift, OpenLateness,
    Produces,
};
pub use error::EmploymentError;
pub use event::{Hired, LateForShift, ShiftEnded, ShiftStarted, WageDeductionDue, WageDue};
pub use interactions::{EmploymentParameters, EmploymentPartial};
pub use lateness::PENALTY_OUT_OF_BOUNDS;
pub use process::{ShiftProcess, ShiftState};
pub use section::AuthoredJob;
pub use system::{
    BIOGRAPHICAL, EmploymentSystem, SECOND_MANAGER, employed_by, employed_by_declaration,
};
