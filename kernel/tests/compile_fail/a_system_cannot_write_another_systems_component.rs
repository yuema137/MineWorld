//! The attempt `INV-7` exists to stop: a system holding its own write token uses it on a
//! component type another system owns.
//!
//! This is the heart of the guarantee. The write is not refused at run time; it cannot be
//! written down.

use mineworld_contracts::{EntityId, SystemId};
use mineworld_kernel::{ComponentStore, SystemIdentity, WriteAccess, owned_component};
use serde::{Deserialize, Serialize};

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

struct SecondStub;
impl SystemIdentity for SecondStub {
    const ID: SystemId = SystemId::from_static("second-stub");
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
    let second = access.grant(&SecondStub).expect("a first grant succeeds");
    let mut store = ComponentStore::new();

    store
        .insert(&second, EntityId::from_raw(1), Measured { amount: 1 })
        .expect("this line does not compile");
}
