//! A System Pack that does not state its package identity is not a pack at all: `PACKAGE` has no
//! default, so the compiler refuses the `impl` (`DECISIONS.md` `ARC-53`). Everything else a pack
//! declares has a safe default; an identity cannot have one, because a default identity would be the
//! wrong pack's.

use mineworld_contracts::SystemId;
use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion};
use mineworld_sdk::SystemPack;

#[derive(Default)]
struct Anonymous;

impl SystemIdentity for Anonymous {
    const ID: SystemId = SystemId::from_static("anonymous");
}

impl System for Anonymous {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl SystemPack for Anonymous {}

fn main() {}
