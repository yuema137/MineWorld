//! Attempt `A8` of PR 03a's adversarial review, re-run against the sealed API.
//!
//! In 03a this **succeeded**, and the reviewer reproduced it from an external crate: construct your
//! own `WriteAccess`, grant yourself another system's token, write its component. Ownership was
//! therefore advisory — holding one issuer inside the kernel closes nothing while anyone can build a
//! second one.
//!
//! All three steps are written out below, in order, so that the case fails at the first one and the
//! `.stderr` file records exactly which door is shut. `WriteAccess::new` is `pub(crate)`, and no
//! `Default` is derived either, because a derived `Default` is a public second constructor and would
//! reopen the hole exactly.
//!
//! The only remaining way to obtain a write token is to install a system into a `World` and be
//! handed a `WorldView` while it runs, which is `BD-1`.

use mineworld_contracts::{EntityId, SystemId};
use mineworld_kernel::{ComponentStore, SystemIdentity, WriteAccess, owned_component};
use serde::{Deserialize, Serialize};

/// The system whose state is being stolen. In a real attempt this type lives in the victim's crate;
/// nothing about that changes the outcome, because the name is all the attacker needs.
struct Victim;
impl SystemIdentity for Victim {
    const ID: SystemId = SystemId::from_static("victim");
}

#[derive(Serialize, Deserialize)]
struct Guarded {
    amount: u32,
}

owned_component! {
    component = Guarded,
    owner = Victim,
    component_type = "guarded",
    schema_version = 1,
}

fn main() {
    // Step 1: build an issuer of write capability. Both constructors are gone.
    let mut access = WriteAccess::new();
    let _also_an_issuer = WriteAccess::default();

    // Step 2: grant myself the victim's token.
    let stolen = access.grant(&Victim).expect("this line does not compile");

    // Step 3: write the victim's state with it.
    let mut store = ComponentStore::new();
    store
        .insert(&stolen, EntityId::from_raw(1), Guarded { amount: 99 })
        .expect("this line does not compile");
}
