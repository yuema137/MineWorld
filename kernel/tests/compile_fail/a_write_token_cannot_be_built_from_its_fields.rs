//! The other way to forge write access: skip the constructor and build the token out of its
//! fields. A token holds nothing but a `PhantomData`, so there is nothing to supply — which is
//! exactly why the field is private.

use mineworld_contracts::SystemId;
use mineworld_kernel::{SystemIdentity, WriteToken};
use std::marker::PhantomData;

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

fn main() {
    let _forged: WriteToken<FirstStub> = WriteToken {
        system: PhantomData,
    };
}
