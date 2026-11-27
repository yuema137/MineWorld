//! The population sampler (`DECISIONS.md` `ARC-123`, `ARC-128`; `pr-s30d-sampler.md`;
//! `pr-s30j-runtime-sampling.md`): integer arithmetic only, the same bytes on every platform. `resolve` draws
//! a world's people at genesis; `sample_newcomer` draws one person who arrives later.

pub mod hash;
pub mod micro;
pub mod newcomer;
pub mod normal;
#[rustfmt::skip]
pub mod normal_table;
pub mod remainder;
pub mod resolve;
pub(crate) mod values;

pub use newcomer::{Arrival, Fixed, FixedBy, Newcomer, Resident, sample_newcomer};
pub use resolve::{Member, Origin, Resolution, Words, resolve};
