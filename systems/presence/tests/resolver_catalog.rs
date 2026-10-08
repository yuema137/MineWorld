//! The resolver catalog's registration rules (`DECISIONS.md` `ARC-39` item 5; step-11 RS-8).
//!
//! The catalog is written once per process, and Cargo runs each file under `tests/` as its own
//! process, so this file holds exactly one `#[test]`, which walks the rules in sequence: what a
//! process that never registered sees, a first registration, the same list again, a different list,
//! and a list naming one id twice. A second test in this file would share the catalog and make the
//! order of tests part of the outcome.

use std::panic::{AssertUnwindSafe, catch_unwind};

use mineworld_contracts::{
    EntityKey, EntityType, LocalPosition, Location, Millimetres, PersonId, PlaceId, SystemId,
};
use mineworld_kernel::{World, WorldRead};
use mineworld_presence::{
    ArrivalResolver, Arriving, PresenceSystem, Resolution, arrival, arrivals, register_resolvers,
    registered_resolvers,
};

/// A resolver that changes nothing, under the id it is given.
struct Identity(&'static str);

impl ArrivalResolver for Identity {
    fn resolver_of(&self) -> SystemId {
        SystemId::from_static(self.0)
    }

    fn resolve(&self, _: &WorldRead<'_>, _: &Arriving, so_far: Resolution) -> Resolution {
        so_far
    }
}

fn list(ids: &[&'static str]) -> Vec<Box<dyn ArrivalResolver>> {
    ids.iter()
        .map(|id| Box::new(Identity(id)) as Box<dyn ArrivalResolver>)
        .collect()
}

fn ids(ids: &[&'static str]) -> Option<Vec<SystemId>> {
    Some(ids.iter().map(|id| SystemId::from_static(id)).collect())
}

/// The message a registration panicked with; fails the test if it did not panic.
fn panic_of(ids: &[&'static str]) -> String {
    let payload = catch_unwind(AssertUnwindSafe(|| register_resolvers(list(ids))))
        .expect_err("this registration must panic");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_owned()))
        .expect("a panic message")
}

#[test]
fn the_catalog_is_written_once_and_refuses_a_different_list() {
    // Never registered: no resolver, and the movers' constructor is the placement's one fact.
    assert_eq!(registered_resolvers(), None);
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    let alice = world
        .create_entity(EntityKey::new("alice").expect("a key"), EntityType::Person)
        .expect("a person");
    let cafe = world
        .create_entity(EntityKey::new("cafe").expect("a key"), EntityType::Place)
        .expect("a place");
    let alice = PersonId::new(alice, EntityType::Person).expect("a person");
    let to =
        Location::in_place(PlaceId::new(cafe, EntityType::Place).expect("a place")).with_local(
            LocalPosition::on_ground(Millimetres::new(1_000), Millimetres::new(2_000)),
        );
    let read = world.read();
    assert_eq!(
        arrivals(&read, alice, to).expect("accepted"),
        vec![arrival(&read, alice, to).expect("accepted")],
        "with nothing registered, arrivals is exactly arrival's one fact"
    );

    // A first registration is sorted by id, whatever order it is written in.
    register_resolvers(list(&["b", "a"]));
    assert_eq!(registered_resolvers(), ids(&["a", "b"]));

    // The same ids again — in either order — change nothing.
    register_resolvers(list(&["a", "b"]));
    assert_eq!(registered_resolvers(), ids(&["a", "b"]));

    // A different list panics, naming both.
    let message = panic_of(&["a"]);
    assert!(
        message.contains(r#"[SystemId("a")]"#)
            && message.contains(r#"[SystemId("a"), SystemId("b")]"#),
        "the panic names both lists: {message}"
    );

    // A list naming one id twice panics, naming it.
    let message = panic_of(&["c", "c"]);
    assert!(
        message.contains("two resolvers for 'c'"),
        "the panic names the id: {message}"
    );

    // Neither refusal touched the catalog.
    assert_eq!(registered_resolvers(), ids(&["a", "b"]));
}
