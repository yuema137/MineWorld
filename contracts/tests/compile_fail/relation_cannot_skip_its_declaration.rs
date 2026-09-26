//! An edge must not be constructible without its declaration: the endpoint check and the
//! canonical ordering of an undirected edge are only guarantees if there is no way around them.

use mineworld_contracts::{EntityId, Relation, RelationTypeId};

fn main() {
    let _edge = Relation {
        relation_type: RelationTypeId::new("connected_to").unwrap(),
        from: EntityId::from_raw(4),
        to: EntityId::from_raw(3),
    };
}
