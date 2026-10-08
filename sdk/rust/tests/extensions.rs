//! IA-5: extension catalogs (`DECISIONS.md` `ARC-62`), through a test-local installed set with two
//! extension lines.
//!
//! ```text
//! test-relay      owns the trait Relay and its catalog, register_relays (write-once, as presence's)
//! test-relay-a    a pack implementing Relay
//! test-relay-b    a pack implementing Relay
//! Echo            a second, test-local catalog with nothing listed (deviation D-2: the SDK cannot
//!                 depend on presence, whose line the real set carries)
//! ```
//!
//! A catalog is written once per process and Cargo runs each file under `tests/` as its own process,
//! so this file holds one `#[test]` that registers through the generated function and observes it.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

use mineworld_contracts::SystemId;
use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion};
use mineworld_sdk::SystemPack;

/// The perception trait this test set answers through: nothing, here.
pub trait Perceives {}

/// A trait test-relay owns and other packs implement.
pub trait Relay: Send + Sync {
    /// The pack that implements it.
    fn relay_of(&self) -> SystemId;
}

/// test-relay's catalog: the relays' ids in registration order, written once per process.
static RELAYS: OnceLock<Vec<SystemId>> = OnceLock::new();

/// test-relay's register function, with presence's rules: the same list again is a no-op, a
/// different one panics naming both.
pub fn register_relays(relays: Vec<Box<dyn Relay>>) {
    let offered: Vec<SystemId> = relays.iter().map(|relay| relay.relay_of()).collect();
    let current = RELAYS.get_or_init(|| offered.clone());
    assert!(
        *current == offered,
        "register_relays was called with {offered:?}, but this process already registered {current:?}"
    );
}

/// A second test-local trait, whose line lists nothing.
pub trait Echo: Send + Sync {}

/// How many times the second catalog was registered, and with how many values.
static ECHOES: OnceLock<usize> = OnceLock::new();

/// The second catalog's register function.
pub fn register_echoes(echoes: Vec<Box<dyn Echo>>) {
    let count = echoes.len();
    assert_eq!(*ECHOES.get_or_init(|| count), count);
}

macro_rules! pack {
    ($Type:ident, $id:literal) => {
        #[derive(Default)]
        pub struct $Type;

        impl SystemIdentity for $Type {
            const ID: SystemId = SystemId::from_static($id);
        }

        impl System for $Type {
            const VERSION: SystemVersion = SystemVersion::new(1);

            fn declaration(&self) -> SystemDeclaration {
                SystemDeclaration::of::<Self>()
            }
        }

        impl Perceives for $Type {}

        impl SystemPack for $Type {
            const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
        }
    };
}

pack!(TestRelay, "test-relay");
pack!(TestRelayA, "test-relay-a");
pack!(TestRelayB, "test-relay-b");

impl Relay for TestRelayA {
    fn relay_of(&self) -> SystemId {
        Self::ID
    }
}

impl Relay for TestRelayB {
    fn relay_of(&self) -> SystemId {
        Self::ID
    }
}

/// A relay this set does not list.
#[derive(Default)]
struct Other;

impl Relay for Other {
    fn relay_of(&self) -> SystemId {
        SystemId::from_static("other")
    }
}

mod set {
    mineworld_sdk::installed! {
        perception: super::Perceives;
        extension super::Relay => super::register_relays: [super::TestRelayA, super::TestRelayB,];
        extension super::Echo => super::register_echoes: [];
        TestRelay => super::TestRelay,
        TestRelayA => super::TestRelayA,
        TestRelayB => super::TestRelayB,
    }
}

#[test]
fn both_lines_register_each_list_in_order_once_and_a_different_list_is_refused() {
    assert_eq!(
        RELAYS.get(),
        None,
        "nothing is registered before a host registers"
    );
    assert_eq!(ECHOES.get(), None);

    set::Capability::register_extensions();
    let listed = vec![
        SystemId::from_static("test-relay-a"),
        SystemId::from_static("test-relay-b"),
    ];
    assert_eq!(
        RELAYS.get(),
        Some(&listed),
        "the first line, in its listed order"
    );
    assert_eq!(
        ECHOES.get(),
        Some(&0),
        "the second line, with its empty list"
    );

    set::Capability::register_extensions();
    assert_eq!(RELAYS.get(), Some(&listed), "registering again is a no-op");

    let lines = set::Capability::extension_types();
    assert_eq!(lines.len(), 2, "one entry per line, in the listed order");
    assert!(lines[0].0.ends_with("Relay") && lines[1].0.ends_with("Echo"));
    assert_eq!(
        lines[0].1,
        [
            std::any::type_name::<TestRelayA>(),
            std::any::type_name::<TestRelayB>(),
        ]
    );
    assert!(lines[1].1.is_empty());

    let payload = catch_unwind(AssertUnwindSafe(|| {
        register_relays(vec![Box::new(Other) as Box<dyn Relay>]);
    }))
    .expect_err("a different list for one trait must panic");
    let message = payload
        .downcast_ref::<String>()
        .cloned()
        .expect("a formatted panic message");
    assert!(
        message.contains(r#"[SystemId("other")]"#)
            && message.contains(
                r#"already registered [SystemId("test-relay-a"), SystemId("test-relay-b")]"#
            ),
        "the panic names both lists: {message}"
    );
    assert_eq!(RELAYS.get(), Some(&listed), "and changed nothing");
}
