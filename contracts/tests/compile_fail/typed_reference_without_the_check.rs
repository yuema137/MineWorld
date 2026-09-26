//! A typed entity reference must not be constructible from a raw identity: the only way in is
//! the checked constructor, which has to be told the entity's actual type.

use mineworld_contracts::{EntityId, PersonId};

fn main() {
    let _person = PersonId(EntityId::from_raw(1));
}
