//! Taking another system's declared name does not take its ownership.
//!
//! A pack can declare a system type whose `ID` is a name another system already uses — nothing
//! can stop it, because the name is a string literal in its own crate. It buys nothing:
//! ownership is a relationship between two *types*, so the impersonator's token is the token of
//! the impersonator's type, and the component is owned by the type that actually declared it.
//!
//! This is why `OwnedBy<S>` carries a system type rather than a `SystemId` value.

use mineworld_contracts::{EntityId, SystemId};
use mineworld_kernel::{ComponentStore, SystemIdentity, WriteAccess, owned_component};
use serde::{Deserialize, Serialize};

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

/// A different type claiming the same declared name.
struct PretendingToBeFirstStub;
impl SystemIdentity for PretendingToBeFirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

#[derive(Serialize, Deserialize)]
struct Measured {
    amount: u32,
}

owned_component! {
    component = Measured,
    owner = FirstStub,
    component_type = "measured",
    schema_version = 1,
}

fn main() {
    let mut access = WriteAccess::new();
    let pretender = access
        .grant(&PretendingToBeFirstStub)
        .expect("a first grant succeeds");
    let mut store = ComponentStore::new();

    store
        .insert(&pretender, EntityId::from_raw(1), Measured { amount: 1 })
        .expect("this line does not compile");
}
