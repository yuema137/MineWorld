//! An entity's lifecycle must not be reachable by assignment: destruction is final only if the
//! checked transition is the only way to reach it.

use mineworld_contracts::{Entity, EntityId, EntityKey, EntityType, LifecycleState};

fn main() {
    let mut entity = Entity::new(
        EntityId::from_raw(1),
        EntityKey::new("alice").unwrap(),
        EntityType::Person,
    );
    entity.lifecycle = LifecycleState::Destroyed;
}
