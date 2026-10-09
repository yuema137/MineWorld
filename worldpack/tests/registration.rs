//! Composing a world registers the build's arrival resolvers (`DECISIONS.md` `ARC-39` item 6; step-11
//! RS-10).
//!
//! The catalog is written once per process, and Cargo runs each file under `tests/` as its own
//! process, so this file holds exactly one `#[test]`: before anything composes nothing is registered;
//! composing registers the installed set's list; composing again changes nothing; and a host that then
//! registers a different list is refused, naming both.

use std::panic::{AssertUnwindSafe, catch_unwind};

use mineworld_contracts::SystemId;
use mineworld_kernel::WorldRead;
use mineworld_presence::{
    ArrivalResolver, Arriving, Resolution, register_resolvers, registered_resolvers,
};
use mineworld_worldpack::WorldPack;

const PACK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../worlds/social-cafe");

/// Presence's extension line in this build's installed set: one resolver (step-11 §17, QP-2; `ARC-62`).
const LISTED: &str = "bodies";

/// A resolver this build does not list.
struct Stray;

impl ArrivalResolver for Stray {
    fn resolver_of(&self) -> SystemId {
        SystemId::from_static("stray")
    }

    fn resolve(&self, _: &WorldRead<'_>, _: &Arriving, so_far: Resolution) -> Resolution {
        so_far
    }
}

#[test]
fn composing_registers_the_installed_set_once_and_a_different_list_is_refused() {
    assert_eq!(
        registered_resolvers(),
        None,
        "nothing is registered before a world is composed"
    );
    let pack = WorldPack::read(PACK).expect("the shipped pack reads");
    let listed = Some(vec![SystemId::from_static(LISTED)]);

    pack.compose().expect("the pack composes");
    assert_eq!(
        registered_resolvers(),
        listed,
        "composing registered the installed set's resolution: list"
    );

    pack.compose().expect("the pack composes a second time");
    assert_eq!(registered_resolvers(), listed, "a no-op");

    let payload = catch_unwind(AssertUnwindSafe(|| {
        register_resolvers(vec![Box::new(Stray) as Box<dyn ArrivalResolver>]);
    }))
    .expect_err("a different list after composing must panic");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .expect("a formatted panic message");
    assert!(
        message.contains(r#"[SystemId("stray")]"#)
            && message.contains(&format!(r#"already registered [SystemId("{LISTED}")]"#)),
        "the panic names both lists: {message}"
    );
    assert_eq!(registered_resolvers(), listed, "and changed nothing");
}
