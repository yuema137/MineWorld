//! `INV-7` for processes: a phone system that hears a call during dinner tries to end the dinner
//! itself, and to start one of the dining system's processes.
//!
//! Both are written in the phone system's own `react`, through the only writable thing it holds.
//! Neither compiles: a process kind names its owner as a type, and every typed process operation
//! requires `P: ProcessKind<Owner = Self>`. What the phone system may do is ask
//! (`request_interrupt`), and the dining system decides.

use mineworld_contracts::{EventEnvelope, ProcessId, ProcessTypeId, SystemId};
use mineworld_kernel::{
    Emission, KernelError, ProcessKind, ProcessStart, System, SystemDeclaration, SystemIdentity,
    SystemVersion, WorldView,
};

struct Dining;
impl SystemIdentity for Dining {
    const ID: SystemId = SystemId::from_static("dining");
}

struct Phone;
impl SystemIdentity for Phone {
    const ID: SystemId = SystemId::from_static("phone");
}

/// Having dinner, owned by `Dining`.
struct Dinner;
impl ProcessKind for Dinner {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("having-dinner");
    type Owner = Dining;
}

impl System for Phone {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        _event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        world.end_process::<Dinner>(ProcessId::from_raw(1))?;
        world.start_process(ProcessStart::<Dinner>::new(Vec::new()))?;
        Ok(Vec::new())
    }
}

fn main() {}
