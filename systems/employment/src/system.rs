//! The installable system: what it declares, the facts it reduces, and the wake that keeps a job's
//! shifts.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, EntityTypeSet, Event, EventEnvelope, EventTypeId,
    PersonId, RelationTypeDeclaration, RelationTypeId, SystemId, Visibility, WorldTime,
};
use mineworld_inventory::{InventorySystem, ItemsProduced};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{PerceptionProvider, PersonEnteredPlace, Presence, PresenceSystem};
use mineworld_schedule::{DAY, TimeOfDay};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::{Employment, OnShift};
use crate::event::{Hired, ShiftEnded, ShiftStarted, WageDue};
use crate::process::{ShiftProcess, ShiftState};

/// Which of this pack's facts belong in a person's biography (`ARC-29`): being hired — once per job, a
/// life event. Shifts and wages are thousands of facts that would bury a biography (step-10 QS-49).
pub const BIOGRAPHICAL: &[EventTypeId] = &[Hired::EVENT_TYPE];

/// Jobs, and work as being there.
#[derive(Default)]
pub struct EmploymentSystem;

impl SystemIdentity for EmploymentSystem {
    const ID: SystemId = SystemId::from_static("employment");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts, and the `job:` section of a person's file, which its `AuthoredSection` impl
/// (`src/section.rs`) describes.
impl SystemPack for EmploymentSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

/// The edge this pack declares: a person is employed by an organization.
pub fn employed_by() -> RelationTypeId {
    RelationTypeId::new("employed-by").expect("a legal relation type name")
}

/// `employed-by`: directed, from a Person to an Organization.
pub fn employed_by_declaration() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        employed_by(),
        EmploymentSystem::ID,
        EntityTypeSet::new([EntityType::Person]).expect("a non-empty set"),
        EntityTypeSet::new([EntityType::Organization]).expect("a non-empty set"),
    )
}

/// The first instant strictly after `now` whose time of day is `at`.
fn next(now: WorldTime, at: TimeOfDay) -> WorldTime {
    let midnight = now.seconds() - TimeOfDay::of(now);
    let today = midnight + at.seconds();
    WorldTime::from_seconds(if today > now.seconds() {
        today
    } else {
        today + DAY
    })
}

fn employment_of(world: &WorldRead<'_>, person: EntityId) -> Option<Employment> {
    world.component::<Employment>(person).cloned()
}

impl System for EmploymentSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on inventory, whose `items-produced` it states (`ARC-26`), and on presence, whose
    /// positions it reads and whose `person-entered-place` it hears. Provides no action: work is
    /// attendance. Never names a wallet or economy.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID, PresenceSystem::ID])
            .owning::<Employment>()
            .emitting::<Hired>()
            .emitting::<ShiftStarted>()
            .emitting::<ShiftEnded>()
            .emitting::<WageDue>()
            .emitting::<ItemsProduced>()
            .subscribing_to::<Hired>()
            .subscribing_to::<ShiftStarted>()
            .subscribing_to::<ShiftEnded>()
            .subscribing_to::<PersonEnteredPlace>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Employment>()?;
        tables.relation(employed_by_declaration())
    }

    /// `hired` → Employment, the `employed-by` edge and the shift process; `shift-started` and
    /// `shift-ended` → the shift in progress, begun and cleared; presence's `person-entered-place` for
    /// an employee on shift → arriving at the workplace opens a present span, leaving it closes one.
    /// These are the only writes of [`Employment`].
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == Hired::EVENT_TYPE {
            let hired: Hired = codec::event_payload(event.payload())?;
            let employee = hired.employee().entity_id();
            let first = next(world.at(), hired.job().from());
            world.start_process(
                ProcessStart::<ShiftProcess>::new(codec::encode(&ShiftState::new(
                    hired.employee(),
                )))
                .with_participants(vec![employee])
                .ending_at(first)
                .uninterruptible(),
            )?;
            world.relate(&employed_by(), employee, hired.job().employer().entity_id())?;
            world.insert(employee, Employment::hired(hired.job().clone()))?;
        } else if *kind == ShiftStarted::EVENT_TYPE {
            let started: ShiftStarted = codec::event_payload(event.payload())?;
            let employee = started.employee().entity_id();
            if let Some(employment) = employment_of(&world.read(), employee) {
                let shift = OnShift::begun(event.at(), started.present());
                world.insert(employee, employment.with_shift(Some(shift)))?;
            }
        } else if *kind == ShiftEnded::EVENT_TYPE {
            let ended: ShiftEnded = codec::event_payload(event.payload())?;
            let employee = ended.employee().entity_id();
            if let Some(employment) = employment_of(&world.read(), employee) {
                world.insert(employee, employment.with_shift(None))?;
            }
        } else if *kind == PersonEnteredPlace::EVENT_TYPE {
            let entered: PersonEnteredPlace = codec::event_payload(event.payload())?;
            let employee = entered.person().entity_id();
            let Some(employment) = employment_of(&world.read(), employee) else {
                return Ok(Vec::new());
            };
            let Some(shift) = employment.shift() else {
                return Ok(Vec::new());
            };
            let workplace = employment.job().workplace();
            let next = if entered.place() == workplace {
                shift.arrived(event.at())
            } else if entered.from() == workplace {
                shift.left(event.at())
            } else {
                return Ok(Vec::new());
            };
            world.insert(employee, employment.with_shift(Some(next)))?;
        }
        Ok(Vec::new())
    }

    /// A shift's start or end, whichever is due. The process is rescheduled to the next one and never
    /// ends — a job goes on as long as the world does.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let missing = || KernelError::ProcessNotRunning {
            process: process.id(),
        };
        let state: ShiftState =
            codec::state(process.state_for::<ShiftProcess>()?).ok_or_else(missing)?;
        let employee = state.employee();
        let employment = employment_of(&world.read(), employee.entity_id()).ok_or_else(missing)?;
        let now = world.at();
        let job = employment.job();
        match employment.shift() {
            None => {
                let present = world
                    .read()
                    .component::<Presence>(employee.entity_id())
                    .is_some_and(|presence| presence.location().place() == job.workplace());
                world.reschedule_process::<ShiftProcess>(
                    process.id(),
                    Some(next(now, job.until())),
                )?;
                Ok(vec![started(employee, present)])
            }
            Some(shift) => {
                let worked = shift.worked_until(now);
                let ended = ended(&world.read(), employee, &employment, worked);
                world.reschedule_process::<ShiftProcess>(
                    process.id(),
                    Some(next(now, job.from())),
                )?;
                Ok(ended)
            }
        }
    }
}

/// `shift-started`, visible to the employee.
fn started(employee: PersonId, present: bool) -> Emission {
    Emission::new::<ShiftStarted>(
        codec::encode(&ShiftStarted::new(employee, present)),
        Visibility::Entities([employee.entity_id()].into_iter().collect()),
    )
    .about(vec![employee.entity_id()])
    .with_participants(vec![employee.entity_id()])
}

/// What the end of a shift states, in order: `shift-ended`; then, if anything was worked, `wage-due`
/// and one `items-produced` per kind whose prorated share is at least one.
///
/// Production goes through inventory's checked constructor. An organization can always take more,
/// so a refusal here means a kind that is no longer declared; that line is skipped rather than
/// failing the shift, and the wage is still due.
fn ended(
    world: &WorldRead<'_>,
    employee: PersonId,
    employment: &Employment,
    worked: u64,
) -> Vec<Emission> {
    let job = employment.job();
    let who = employee.entity_id();
    let employer = job.employer();
    let mut out = vec![
        Emission::new::<ShiftEnded>(
            codec::encode(&ShiftEnded::new(employee, worked)),
            Visibility::Entities([who].into_iter().collect()),
        )
        .about(vec![who])
        .with_participants(vec![who]),
    ];
    let amount = job.wage_for(worked);
    if amount > 0 {
        out.push(
            Emission::new::<WageDue>(
                codec::encode(&WageDue::new(employee, employer, amount)),
                Visibility::Participants,
            )
            .about(vec![who, employer.entity_id()])
            .with_participants(vec![who, employer.entity_id()]),
        );
    }
    out.extend(
        job.produced_for(worked)
            .into_iter()
            .filter_map(|(item, count)| {
                mineworld_inventory::produce(world, employer.entity_id(), item, count).ok()
            }),
    );
    out
}

impl PerceptionProvider for EmploymentSystem {
    /// A person's [`Employment`], **to that person and nobody else** (`INV-13`): one's job and how the
    /// shift is going are one's own to know.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if subject != observer {
            return Vec::new();
        }
        world
            .component::<Employment>(subject)
            .map(|employment| {
                vec![ComponentRecord::new::<Employment>(
                    subject,
                    codec::to_value(employment),
                )]
            })
            .unwrap_or_default()
    }
}
