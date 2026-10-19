//! Where people may sit, and who is seated.
//!
//! A **System Pack** (`docs/DECISIONS.md` `ARC-147`; `docs/CORE_CONCEPTS.md` §6.4). A world that installs it
//! has benches a person may sit on, one person to a sit point, and a seated person who stays where they sat.
//! A world that does not is still a valid world, with nobody sitting.
//!
//! ```text
//! section     sit_points           a place file's `sit_points:` list, validated by [`SitPoints`]
//! emits       place-sit-points     at genesis, from the section; public
//!             sat-down, stood-up   (see the actions)
//! owns        SitPoints            `sit-points`, on a Place, reduced from `place-sit-points` by this pack alone
//!             Posture              `posture-state`, on a Person, while seated; standing is its absence
//! depends on  presence, movement   where people are, and whether they are walking
//! ```
//!
//! A sit point is free floor, not furniture: the server knows where a person may sit, never what the bench is
//! made of or how high the seat is. A Presentation Pack draws the bench and the figure. The sit point's key is
//! the name a client sends; the server answers with the verdict.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod codec;
mod key;
mod point;
mod section;

pub mod component;
pub mod event;
pub mod system;

pub use component::Posture;
pub use event::PlaceSitPoints;
pub use key::{InvalidSitKey, KEY_MAX_BYTES, SitGroupKey, SitPointKey};
pub use point::{
    COORDINATE_BOUND, InvalidSitPoints, SIT_POINT_SPACING, SIT_POINTS_MAX, SitPoint, SitPoints,
};
pub use system::{BIOGRAPHICAL, PostureSystem};
