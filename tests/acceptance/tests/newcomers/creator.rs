//! Test-only arrival source with its own System Pack declaration, outside the installed registry.
use mineworld_attributes_model::{ATTRIBUTES, NewcomerAdmitted, admit_newcomer, check_newcomer};
use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, EntityType, Event, EventEnvelope, EventSchemaVersion,
    EventTypeId, PersonId, Rejection, SystemId, Visibility,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, WorldRead,
    WorldView,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default)]
pub struct Creator {
    pub mode: Mode,
}
#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)] // Each integration binary selects only the relevant adversarial modes.
pub enum Mode {
    #[default]
    Checked,
    Unchecked,
    Missing,
    Duplicate,
    FutureCause,
}
impl SystemIdentity for Creator {
    const ID: SystemId = SystemId::from_static("test-arrivals");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arrive;
impl Action for Arrive {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("test-arrive");
    const OWNER: SystemId = Creator::ID;
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Arrived {
    pub person: PersonId,
}
impl Event for Arrived {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("test-arrived");
    const OWNER: SystemId = Creator::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}
impl System for Creator {
    const VERSION: SystemVersion = SystemVersion::new(1);
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([ATTRIBUTES])
            .providing::<Arrive>()
            .emitting::<Arrived>()
            .emitting::<NewcomerAdmitted>()
            .subscribing_to::<Arrived>()
    }
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        if matches!(self.mode, Mode::Unchecked) {
            return Ok(());
        }
        let person = PersonId::new(intent.actor(), EntityType::Person)
            .map_err(|_| Rejection::PreconditionFailed)?;
        check_newcomer(world, &Self::ID, person)
    }
    fn resolve(
        &self,
        _: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let person = PersonId::new(intent.actor(), EntityType::Person).expect("validated person");
        Ok(vec![Emission::new::<Arrived>(
            serde_json::to_vec(&Arrived { person }).expect("encodes"),
            Visibility::SystemInternal,
        )])
    }
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let arrived: Arrived = serde_json::from_slice(event.payload().payload()).expect("own fact");
        if matches!(self.mode, Mode::Missing) {
            return Ok(vec![]);
        }
        if matches!(self.mode, Mode::Unchecked | Mode::FutureCause) {
            return Ok(vec![Emission::new::<NewcomerAdmitted>(
                serde_json::to_vec(&NewcomerAdmitted {
                    person: arrived.person,
                    because: if matches!(self.mode, Mode::FutureCause) {
                        mineworld_contracts::EventId::from_raw(u64::MAX)
                    } else {
                        event.id()
                    },
                })
                .unwrap(),
                Visibility::SystemInternal,
            )]);
        }
        admit_newcomer(&world.read(), &Self::ID, arrived.person, event.id())
            .map(|fact| {
                if matches!(self.mode, Mode::Duplicate) {
                    vec![fact.clone(), fact]
                } else {
                    vec![fact]
                }
            })
            .map_err(|reason| KernelError::FactRefusedByOwner {
                system: Self::ID,
                event_type: Arrived::EVENT_TYPE,
                reason,
            })
    }
}
