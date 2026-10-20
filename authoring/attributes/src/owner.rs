//! The owner interface (`DECISIONS.md` `ARC-124`; `MODULE_SPEC.md` §3.1 "Owns"; Q-S30c-2).
//!
//! An owner declares its attributes in a file it ships (`owner:` itself), receives each person's
//! resolved values by subscribing to `attributes-handed` (no dependency, `ARC-28` item 2), keeps them in
//! its own state, projects them on a read, and discloses them within its own component through
//! [`disclose_owned`], the same filter and renderer `attributes` uses. Its attributes exist only where
//! `attributes` is enabled: elsewhere [`disclose_owned`] returns `None` and the owner runs as before.

use mineworld_contracts::EntityId;
use mineworld_kernel::{SystemIdentity, WorldRead};
use serde_json::Value;

use crate::facts::RunningCatalogue;
use crate::id::QualifiedId;
use crate::render::{View, disclose};
use crate::value::AttributeValue;

/// A System Pack that owns some of a person's attributes.
pub trait AttributeOwner: SystemIdentity {
    /// One owned attribute's value for `person`, projected from this pack's own state; `None` when
    /// absent.
    fn project(
        world: &WorldRead<'_>,
        person: EntityId,
        attribute: &QualifiedId,
    ) -> Option<AttributeValue>;
}

/// The disclosure of `O`'s attributes of `subject` to `observer`, as an observation carries it; `None`
/// when nothing is disclosable or the world does not enable `attributes`.
pub fn disclose_owned<O: AttributeOwner>(
    world: &WorldRead<'_>,
    observer: EntityId,
    subject: EntityId,
) -> Option<Value> {
    let catalogue = RunningCatalogue::in_world(world)?;
    let disclosed = disclose(
        catalogue.catalogue(),
        &O::ID,
        |declaration| O::project(world, subject, &declaration.id),
        View::of(observer, subject),
    )?;
    serde_json::to_value(disclosed).ok()
}
