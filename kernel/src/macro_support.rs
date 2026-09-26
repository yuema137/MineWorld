//! The contract-layer items [`owned_component!`](crate::owned_component) names on the caller's
//! behalf.
//!
//! A macro expands in the caller's crate, where none of these may be in scope and the dependency
//! may not even be called `mineworld_contracts`, so the expansion reaches them through
//! `$crate`. Not part of this crate's interface: use
//! [`mineworld_contracts`](mineworld_contracts) directly.

pub use mineworld_contracts::{Component, ComponentSchemaVersion, ComponentTypeId, SystemId};
