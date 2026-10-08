//! A process that never registers resolvers (step-11 §16 RS-9, RS-7's second half; QB-15 bound 2;
//! `docs/DECISIONS.md` `ARC-39` item 7).
//!
//! Nothing in this file registers, and nothing composes through `worldpack`, so the catalog stays
//! unset for the whole process. Two things follow, and both are shown:
//!
//! ```text
//! RS-9   installing a resolver's pack fails loudly, at assembly, naming the pack,
//!        register_resolvers and ARC-39 — a resolver is never silently skipped
//! RS-7   a world with no resolver's pack runs exactly as before: the twelve-move script records the
//!        same per-move facts as in the registered process (arrival_resolvers.rs)
//! ```

mod resolvers;

use std::panic::{AssertUnwindSafe, catch_unwind};

use mineworld_kernel::World;
use mineworld_presence::{PresenceSystem, registered_resolvers};
use resolvers::{Fences, Pack, Plan, SCRIPT_START, Town, run_inert_script};

/// RS-9. A host that never registered cannot install a resolver's pack.
#[test]
fn installing_a_resolver_pack_in_an_unregistered_process_panics_naming_why() {
    assert_eq!(registered_resolvers(), None, "this process never registers");
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    let payload = catch_unwind(AssertUnwindSafe(|| world.install(Fences)))
        .expect_err("installing test-fences must panic");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .expect("a formatted panic message");
    for named in ["'test-fences'", "register_resolvers", "ARC-39"] {
        assert!(
            message.contains(named),
            "the panic names {named}: {message}"
        );
    }
}

/// RS-7, the never-registered half: no resolver, so the script runs exactly as before the seam.
#[test]
fn a_world_without_a_resolver_pack_runs_unchanged_when_nothing_is_registered() {
    let mut town = Town::begin(&Plan::new(
        &[Pack::Movement],
        &[("alice", Some(SCRIPT_START))],
    ));
    run_inert_script(&mut town);
    assert_eq!(registered_resolvers(), None);
}
