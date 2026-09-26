//! A reference to a place must not be usable where a reference to a person is required.

use mineworld_contracts::{EntityId, EntityType, PersonId, PlaceId};

fn requires_a_person(_person: PersonId) {}

fn main() {
    let place = PlaceId::new(EntityId::from_raw(1), EntityType::Place).unwrap();
    requires_a_person(place);
}
