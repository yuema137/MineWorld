//! A surveyor's price book: the asks it surveyed and the sales it perceived, and nothing else.
//!
//! A **System Pack** (`docs/decisions/ARC-116.md`; design
//! `.structured-coding/plans/mvp0/pr-s31c-price-survey.md`). It builds the notebook of the operator's
//! price clerk: a person whose backend may be a deterministic model or a language model, and which
//! must work only from what it can legitimately perceive. Because the book is world state disclosed
//! through the Observation, every backend driving the seat reads the same book.
//!
//! ```text
//! section     price-survey          a person's { kinds: { categories }, depth?, window?,
//!                                   resurvey_after? }: makes it a surveyor
//! provides    survey {}             one complete offer where tracked asks are on display; at most
//!                                   once per place per resurvey_after (surveyed-recently)
//! hears       sold                  economy's, through its type; noted for each surveyor that
//!                                   perceived it, by presence's audience rule (ARC-43)
//! emits       survey-book-opened    genesis; SystemInternal
//!             price-noted           a witnessed sale, or one survey's asks; SystemInternal
//! owns        PriceBook             on a Person (component type `survey-book`): one ring of notes
//!                                   per kind, ≤ depth, none older than window
//! discloses   survey-book           to the holder only, filtered to the window at the read's instant
//! depends on  economy, item, presence
//! ```
//!
//! **Integers only.** Times are world seconds, prices minor units; no float, no hash order.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod section;
mod witness;

pub mod action;
pub mod component;
pub mod event;
pub mod judge;
pub mod span;
pub mod system;

pub use action::{Survey, survey_requirement};
pub use component::{KindNotes, Note, PriceBook, Surveyed};
pub use event::{Noted, PriceNoted, Source, SurveyBookOpened};
pub use judge::{Plan, asks_here, codes, judge_survey};
pub use section::{
    AuthoredKinds, AuthoredSurvey, CATEGORIES_MAX, DEPTH_DEFAULT, DEPTH_MAX, RESURVEY_BOUNDS,
    RESURVEY_DEFAULT, WINDOW_BOUNDS, WINDOW_DEFAULT,
};
pub use span::Span;
pub use system::{BIOGRAPHICAL, PriceSurveySystem};
