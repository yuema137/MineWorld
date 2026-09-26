//! Write access is granted, never made. If a token could be constructed, every other guarantee
//! in the kernel would be decoration: a pack would simply build the token of whichever system
//! owns the state it wanted to write.
//!
//! Two ways to try it, neither of which needs `unsafe`: the constructor and `Default`. The third
//! way, building it out of its fields, is its own case — the compiler reports that one in a later
//! pass, so it has to be compiled on its own to be pinned.

use mineworld_contracts::SystemId;
use mineworld_kernel::{SystemIdentity, WriteToken};

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

fn main() {
    let _from_constructor = WriteToken::<FirstStub>::new();
    let _from_default = WriteToken::<FirstStub>::default();
}
