//! Installing a system requires a *value* of the system type, and that is the mitigation a pack has
//! against another crate installing something under its name: a system type with a private field can
//! only be built by the code that defines it.
//!
//! The module boundary here stands in for the crate boundary — the privacy rule is the same one — so
//! the case shows what code outside `pack` can and cannot do: it can name `Guarded`, and it cannot
//! construct one.
//!
//! This mitigation is real but weak, and the documentation in `access.rs` says so: most packs export
//! `pub struct InventorySystem;`, which gives it away. The load-bearing protection since PR 03b is
//! that installation is the world assembler's act and a `WriteAccess` cannot be built at all.

use mineworld_contracts::SystemId;
use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion, World};

mod pack {
    use super::*;

    /// A system that keeps its own constructor.
    pub struct Guarded {
        _private: (),
    }

    impl SystemIdentity for Guarded {
        const ID: SystemId = SystemId::from_static("guarded");
    }

    impl System for Guarded {
        const VERSION: SystemVersion = SystemVersion::new(1);

        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>()
        }
    }
}

fn main() {
    let mut world = World::new();
    let _ = world.install(pack::Guarded { _private: () });
}
