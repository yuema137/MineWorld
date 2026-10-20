//! The population sampler (`DECISIONS.md` `ARC-123`; `pr-s30d-sampler.md`): integer arithmetic only, the
//! same bytes on every platform.

pub mod hash;
pub mod micro;
pub mod normal;
#[rustfmt::skip]
pub mod normal_table;
pub mod remainder;
pub mod resolve;

pub use resolve::{Member, Origin, Resolution, Words, resolve};
