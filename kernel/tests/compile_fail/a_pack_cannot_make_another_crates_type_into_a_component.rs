//! Why a dishonest ownership claim can never reach another pack's state.
//!
//! `OwnedBy` is a public trait, so a pack can implement it for a component type it defines — that
//! is how a component declares its owner — and nothing stops it from implementing it dishonestly.
//! The question is what a dishonest claim can reach. The answer is: only the liar's own types,
//! and this case is the reason.
//!
//! `Component` is a foreign trait, and Rust's coherence rules forbid implementing a foreign trait
//! for a foreign type. So the only component types a pack can bring into existence are the ones it
//! defines itself, and since `OwnedBy` requires `Component`, a claim about another pack's
//! component type cannot be written at all. A dishonest claim can therefore only widen who may
//! write the liar's own state — never narrow anyone else's — and
//! `ComponentStore::declare` refuses it when the type is declared.
//!
//! `String` stands in for another crate's type, which is the position every pack is in with
//! respect to somebody else's component.

use mineworld_contracts::{Component, ComponentSchemaVersion, ComponentTypeId, SystemId};
use mineworld_kernel::SystemIdentity;

struct FirstStub;
impl SystemIdentity for FirstStub {
    const ID: SystemId = SystemId::from_static("first-stub");
}

impl Component for String {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("stolen");
    const OWNER: SystemId = <FirstStub as SystemIdentity>::ID;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

fn main() {}
