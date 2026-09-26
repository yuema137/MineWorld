//! `WriteAccess::grant` requires a *value* of the system type, and that is the mitigation a pack
//! has against another crate asking for its token: a system type with a private field can only be
//! built by the code that defines it.
//!
//! The module boundary here stands in for the crate boundary — the privacy rule is the same one —
//! so the case shows what a pack outside `pack` can and cannot do: it can name `Guarded`, and it
//! cannot construct one.

use mineworld_contracts::SystemId;
use mineworld_kernel::{SystemIdentity, WriteAccess};

mod pack {
    use super::*;

    /// A system that keeps its own constructor.
    pub struct Guarded {
        _private: (),
    }

    impl SystemIdentity for Guarded {
        const ID: SystemId = SystemId::from_static("guarded");
    }
}

fn main() {
    let mut access = WriteAccess::new();
    let _token = access.grant(&pack::Guarded { _private: () });
}
