//! Taking another system's declared name does not take its ownership.
//!
//! A pack can declare a system type whose `ID` is a name another system already uses — nothing can
//! stop it, because the name is a string literal in its own crate. It buys nothing: ownership is a
//! relationship between two *types*, so the impersonator's view is a view for the impersonator's
//! type, and the component belongs to the type that actually declared it.
//!
//! This is why `OwnedBy<S>` carries a system type rather than a `SystemId` value. (What the *name*
//! collision does cause is a refused installation, because a world holds one system per name — but
//! that is a runtime refusal, and this is the earlier one.)

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

impl System for PretendingToBeFirstStub {
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
