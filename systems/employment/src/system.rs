//! The installable system: what it declares, the facts it reduces, the wake that keeps a job's
//! shifts, and the one action it provides, `penalize` (`ARC-287`).

use mineworld_contracts::{
    ActionIntent, ComponentRecord, EntityId, EntityType, EntityTypeSet, Event, EventEnvelope,
    EventTypeId, PersonId, Rejection, RejectionCode, RelationTypeDeclaration, RelationTypeId,
    SystemId, Visibility, WorldTime,
};
use mineworld_inventory::{InventorySystem, ItemsProduced};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, PersonEnteredPlace, Presence, PresenceSystem};
use mineworld_schedule::{DAY, TimeOfDay};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions;
use serde_json::Value;

use mineworld_attributes_model::value::BoundedText;
use mineworld_attributes_model::{AttributeOwner, AttributeValue, QualifiedId, disclose_owned};

use crate::action::Penalize;
use crate::codec;
use crate::component::{DisclosedEmployment, Employment, OnShift};
use crate::event::{Hired, LateForShift, ShiftEnded, ShiftStarted, WageDeductionDue, WageDue};
use crate::interactions::parameters;
use crate::lateness;
use crate::process::{ShiftProcess, ShiftState};

/// Which of this pack's facts belong in a person's biography (`ARC-29`): being hired — once per job, a
/// life event — and being late for a shift, a few a month at most, which a person remembers
/// (`ARC-287`). Shifts and wages are thousands of facts that would bury a biography (step-10 QS-49).
pub const BIOGRAPHICAL: &[EventTypeId] = &[Hired::EVENT_TYPE, LateForShift::EVENT_TYPE];

/// The refusal of a second manager for one employer, at genesis (`ARC-287`).
pub const SECOND_MANAGER: RejectionCode = RejectionCode::from_static("second-manager");

/// Jobs, and work as being there.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct EmploymentSystem;

impl SystemIdentity for EmploymentSystem {
    const ID: SystemId = SystemId::from_static("employment");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts, the `job:` section of a person's file, which its `AuthoredSection` impl
/// (`src/section.rs`) describes, and its section of the World's Interaction List (`ARC-63`,
/// `src/interactions.rs`), which is its configuration.
impl SystemPack for EmploymentSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
    mineworld_sdk::interactions!();
    mineworld_sdk::ships_data!("attributes/occupation.yaml");
}

/// `employment.occupation` (S30-f §5.10, the named exception of step-30 §9.4): projected from the held
/// job's title; absent when unemployed or untitled. Exists only where the world enables `attributes`.
impl AttributeOwner for EmploymentSystem {
    fn project(
        world: &WorldRead<'_>,
        person: EntityId,
        attribute: &QualifiedId,
    ) -> Option<AttributeValue> {
        if attribute.as_str() != OCCUPATION {
            return None;
        }
        let title = world.component::<Employment>(person)?.job().title()?;
        BoundedText::new(title, OCCUPATION_MAX_BYTES)
            .ok()
            .map(AttributeValue::Text)
    }
}

/// The attribute this pack owns.
pub const OCCUPATION: &str = "employment.occupation";

/// `employment.occupation`'s declared `max_bytes`.
const OCCUPATION_MAX_BYTES: u16 = 40;

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

/// A fact this pack may not take: refused by its owner (`ARC-26`).
fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: EmploymentSystem::ID,
        event_type,
        reason,
    }
}

impl System for EmploymentSystem {
    /// 2 since PR LE-b (`ARC-287`): the Interaction List section, `late-for-shift`,
    /// `wage-deduction-due`, `penalize`, and the lateness `Employment` keeps.
    const VERSION: SystemVersion = SystemVersion::new(2);

    /// Depends on inventory, whose `items-produced` it states (`ARC-26`), and on presence, whose
    /// positions it reads and whose `person-entered-place` it hears. Provides `penalize`, a manager's
    /// only; work itself is attendance. Never names a wallet or economy.
    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .depending_on([InventorySystem::ID, PresenceSystem::ID])
                .owning::<Employment>()
                .providing::<Penalize>()
                .emitting::<Hired>()
                .emitting::<ShiftStarted>()
                .emitting::<ShiftEnded>()
                .emitting::<WageDue>()
                .emitting::<LateForShift>()
                .emitting::<WageDeductionDue>()
                .emitting::<ItemsProduced>()
                .subscribing_to::<Hired>()
                .subscribing_to::<ShiftStarted>()
                .subscribing_to::<ShiftEnded>()
                .subscribing_to::<LateForShift>()
                .subscribing_to::<WageDeductionDue>()
                .subscribing_to::<PersonEnteredPlace>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<Employment>()?;
        tables.relation(employed_by_declaration())
    }

    /// Whether this `penalize` may happen (`src/lateness.rs`): the payload is one; the actor is the
    /// employee's manager (`PermissionDenied`); a lateness is open (`NoSupportedInteraction`); the
    /// amount is within its bound (`penalty-out-of-bounds`). Nothing spatial: a boss docks pay from
    /// anywhere.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let penalize = lateness::read_penalize(intent)?;
        lateness::admit(world, intent.actor(), &penalize).map(|_| ())
    }

    /// An accepted `penalize` states `wage-deduction-due` for its amount, caused by it — and nothing
    /// else; its reduction closes the lateness.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let deduction = |reason| refused(WageDeductionDue::EVENT_TYPE, reason);
        let penalize = lateness::read_penalize(intent).map_err(deduction)?;
        let read = world.read();
        let (employment, open) =
            lateness::admit(&read, intent.actor(), &penalize).map_err(deduction)?;
        Ok(vec![lateness::deduction(
            penalize.employee(),
            &employment,
            penalize.amount(),
            open.shift(),
        )])
    }

    /// `hired` → Employment, the `employed-by` edge and the shift process (a second manager of one
    /// employer is refused by name); `shift-started` and `shift-ended` → the shift in progress, begun
    /// and cleared; presence's `person-entered-place` for an employee on shift → arriving at the
    /// workplace opens a present span, leaving it closes one, and a first arrival past the grace states
    /// `late-for-shift`; `late-for-shift` → the lateness kept, and the deduction or the open lateness it
    /// leaves; `wage-deduction-due` → that lateness closed. These are the only writes of [`Employment`].
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let kind = event.event_type();
        if *kind == Hired::EVENT_TYPE {
            let hired: Hired = codec::event_payload(event.payload())?;
            hire(world, &hired)?;
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
            return entered_place(world, &entered, event.at());
        } else if *kind == LateForShift::EVENT_TYPE {
            let late: LateForShift = codec::event_payload(event.payload())?;
            let employee = late.employee().entity_id();
            if let Some(employment) = employment_of(&world.read(), employee) {
                let (open, deduction) = lateness::consequences(&world.read(), &late, &employment);
                world.insert(employee, employment.with_lateness(late.shift(), open))?;
                return Ok(deduction.into_iter().collect());
            }
        } else if *kind == WageDeductionDue::EVENT_TYPE {
            let due: WageDeductionDue = codec::event_payload(event.payload())?;
            let employee = due.employee().entity_id();
            if let Some(employment) = employment_of(&world.read(), employee)
                && employment
                    .open()
                    .is_some_and(|open| open.shift() == due.shift())
            {
                world.insert(employee, employment.closed())?;
            }
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
                let mut ended = ended(&world.read(), employee, &employment, worked);
                if worked == 0 && shift.present_since().is_none() {
                    ended.push(lateness::complaint(
                        &world.read(),
                        employee,
                        &employment,
                        job.shift_start(now),
                        job.shift_seconds(),
                        true,
                    ));
                }
                world.reschedule_process::<ShiftProcess>(
                    process.id(),
                    Some(next(now, job.from())),
                )?;
                Ok(ended)
            }
        }
    }
}

/// `hired`: the shift process, the `employed-by` edge and the Employment — after refusing a second
/// manager for the same employer, by name.
fn hire(world: &mut WorldView<'_, EmploymentSystem>, hired: &Hired) -> Result<(), KernelError> {
    let employee = hired.employee().entity_id();
    let job = hired.job();
    if job.manages()
        && lateness::manager_of(&world.read(), job.employer())
            .is_some_and(|manager| manager != hired.employee())
    {
        return Err(refused(
            Hired::EVENT_TYPE,
            Rejection::System {
                code: SECOND_MANAGER,
                detail: Some(format!(
                    "organization #{} already has a manager; at most one employee per employer \
                     may say `manages: true`",
                    job.employer().entity_id().raw()
                )),
            },
        ));
    }
    let first = next(world.at(), job.from());
    world.start_process(
        ProcessStart::<ShiftProcess>::new(codec::encode(&ShiftState::new(hired.employee())))
            .with_participants(vec![employee])
            .ending_at(first)
            .uninterruptible(),
    )?;
    world.relate(&employed_by(), employee, job.employer().entity_id())?;
    world.insert(employee, Employment::hired(job.clone()))?;
    Ok(())
}

/// Presence's `person-entered-place` for an employee on shift: arriving at the workplace opens a
/// present span — the first arrival of a shift begun without them, past the grace, also states
/// `late-for-shift` — and leaving it closes one.
fn entered_place(
    world: &mut WorldView<'_, EmploymentSystem>,
    entered: &PersonEnteredPlace,
    at: WorldTime,
) -> Result<Vec<Emission>, KernelError> {
    let employee = entered.person();
    let who = employee.entity_id();
    let read = world.read();
    let Some(employment) = employment_of(&read, who) else {
        return Ok(Vec::new());
    };
    let Some(shift) = employment.shift() else {
        return Ok(Vec::new());
    };
    let job = employment.job();
    let workplace = job.workplace();
    let mut facts = Vec::new();
    let next = if entered.place() == workplace {
        let first = shift.present_since().is_none() && shift.worked_until(at) == 0;
        let start = job.shift_start(at);
        let due = start.seconds() + i64::try_from(parameters(&read, who, job).grace()).unwrap_or(0);
        if first && at.seconds() > due {
            let late_by = u64::try_from(at.seconds() - due).unwrap_or(0);
            facts.push(lateness::complaint(
                &read,
                employee,
                &employment,
                start,
                late_by,
                false,
            ));
        }
        shift.arrived(at)
    } else if entered.from() == workplace {
        shift.left(at)
    } else {
        return Ok(Vec::new());
    };
    world.insert(who, employment.with_shift(Some(next)))?;
    Ok(facts)
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
    /// To a manager, one complete `penalize` per member of staff whose lateness may still be penalized
    /// (`src/lateness.rs`); to anybody else, nothing.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        lateness::offers(world, observer, target)
    }

    /// A person's [`Employment`], **to that person and nobody else** (`INV-13`): one's job and how the
    /// shift is going are one's own to know. A manager's carries `late_staff`, the staff whose lateness
    /// may still be penalized, which is a manager's to know (`ARC-287`).
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
                let disclosed = DisclosedEmployment::new(
                    employment.clone(),
                    lateness::late_staff(world, subject),
                    disclose_owned::<Self>(world, observer, subject),
                );
                vec![ComponentRecord::new::<Employment>(
                    subject,
                    codec::to_value(&disclosed),
                )]
            })
            .unwrap_or_default()
    }
}
