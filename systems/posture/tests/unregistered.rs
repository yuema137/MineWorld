//! AC-b-8: a process whose anchoring catalog lacks posture refuses to install posture, naming
//! `register_anchorings` and `ARC-62` — so a host that forgot the installed set's line never lets walkers
//! push seated people. Its own test binary: the catalog is written once per process.

use mineworld_kernel::World;
use mineworld_movement::MovementSystem;
use mineworld_posture::PostureSystem;
use mineworld_presence::PresenceSystem;

#[test]
#[should_panic(expected = "register_anchorings")]
fn posture_refuses_to_install_when_the_catalog_does_not_name_it() {
    mineworld_presence::register_resolvers(Vec::new());
    mineworld_movement::register_wayfinders(Vec::new());
    mineworld_bodies::register_anchorings(Vec::new());
    assert_eq!(mineworld_bodies::registered_anchorings(), Some(Vec::new()));
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    world.install(MovementSystem).expect("movement installs");
    let _ = world.install(PostureSystem);
}
