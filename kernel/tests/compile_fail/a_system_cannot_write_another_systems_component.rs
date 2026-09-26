//! The attempt `INV-7` exists to stop: a running system writes a component type another system
//! owns.
//!
//! This is the heart of the guarantee, and it is written here the way it would actually happen — in
//! a system's own `resolve`, through the [`WorldView`] dispatch hands it. That view is now the only
//! writable thing a system ever holds, so this is the whole surface the attempt can use.
//!
//! The write is not refused at run time; it cannot be written down.

use mineworld_contracts::{ActionIntent, EntityId, SystemId};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, WorldView,
    owned_component,
};
use serde::{Deserialize, Serialize};

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

struct SecondStub;
impl SystemIdentity for SecondStub {
    const ID: SystemId = SystemId::from_static("second-stub");
}

/// State the first system owns.
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

impl System for SecondStub {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        _intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        world.insert(EntityId::from_raw(1), Measured { amount: 1 })?;
        Ok(Vec::new())
    }
}

fn main() {}
