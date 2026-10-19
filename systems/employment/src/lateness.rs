//! Lateness and what it costs (`docs/DECISIONS.md` `ARC-287`; `pr-le-leave-early.md` §0.6).
//!
//! ```text
//! late-for-shift      the first arrival at the workplace in a shift begun without the employee, later
//!                     than the shift's start plus the grace; or, at the shift's end, never having come
//!                     (absent, late_by = the shift's length). Always stated: a complaint to the employee
//!                     and to the employer's manager, if there is one and it is not the one late
//! count, level        latenesses within the window, this one included; notice < warning_at ≤ warning <
//!                     final_at ≤ final
//! deduction off       nothing more (the default: no money moves, I-LE2-4)
//! deduction on        no manager:   wage-deduction-due at once, by the author's formula
//!                     a manager:    the lateness is open; the manager is offered `penalize` up to the
//!                                   formula's amount until the window passes or one is accepted
//! the manager's own   complained about to nobody, never penalized
//! ```
//!
//! Every function here reads the world and builds facts; the writes are `system.rs`'s reductions.

use mineworld_contracts::{
    ActionIntent, EntityId, EntityType, OrganizationId, PersonId, Rejection, RejectionCode,
    Visibility, WorldTime,
};
use mineworld_kernel::{Emission, WorldRead};
use mineworld_presence::Offer;

use crate::action::{PENALIZE_REQUIREMENT, Penalize};
use crate::codec;
use crate::component::{Employment, LateStaff, OpenLateness};
use crate::event::{LateForShift, WageDeductionDue};
use crate::interactions::parameters;

/// The refusal of a penalty above the open lateness's bound, or of nothing.
pub const PENALTY_OUT_OF_BOUNDS: RejectionCode =
    RejectionCode::from_static("penalty-out-of-bounds");

/// The employer's manager: the one employee whose job says `manages`, if any.
pub(crate) fn manager_of(world: &WorldRead<'_>, employer: OrganizationId) -> Option<PersonId> {
    world
        .components::<Employment>()
        .find(|(_, employment)| {
            employment.job().manages() && employment.job().employer() == employer
        })
        .and_then(|(person, _)| PersonId::new(person, EntityType::Person).ok())
}

/// `late-for-shift` for `employee` in the shift begun at `shift`, `late_by` seconds late.
pub(crate) fn complaint(
    world: &WorldRead<'_>,
    employee: PersonId,
    employment: &Employment,
    shift: WorldTime,
    late_by: u64,
    absent: bool,
) -> Emission {
    let who = employee.entity_id();
    let job = employment.job();
    let parameters = parameters(world, who, job);
    let window = parameters.window();
    let earlier = employment
        .late()
        .iter()
        .filter(|at| at.seconds() < shift.seconds() && shift.seconds() - at.seconds() < window)
        .count();
    let count = u32::try_from(earlier + 1).unwrap_or(u32::MAX);
    let manager = manager_of(world, job.employer()).filter(|manager| *manager != employee);
    let audience: Vec<EntityId> = core::iter::once(who)
        .chain(manager.map(PersonId::entity_id))
        .collect();
    Emission::new::<LateForShift>(
        codec::encode(&LateForShift::new(
            employee,
            job.employer(),
            manager,
            shift,
            late_by,
            count,
            parameters.level(count),
            absent,
        )),
        Visibility::Entities(audience.iter().copied().collect()),
    )
    .about(vec![who])
    .with_participants(audience)
}

/// What a `late-for-shift` leaves behind, read at its reduction: the lateness the manager may still
/// penalize, and the deduction stated at once when there is no manager — each only when deduction is
/// on, the amount is at least 1, and the one late is not the manager.
pub(crate) fn consequences(
    world: &WorldRead<'_>,
    late: &LateForShift,
    employment: &Employment,
) -> (Option<OpenLateness>, Option<Emission>) {
    let who = late.employee().entity_id();
    let job = employment.job();
    let parameters = parameters(world, who, job);
    let amount = parameters.deduction(late.late_by());
    if !parameters.deduction_enabled || amount == 0 || job.manages() {
        return (None, None);
    }
    if late.manager().is_some() {
        let open = OpenLateness::new(
            late.shift(),
            late.late_by(),
            late.count(),
            late.level(),
            amount,
        );
        return (Some(open), None);
    }
    (
        None,
        Some(deduction(late.employee(), employment, amount, late.shift())),
    )
}

/// `wage-deduction-due`, heard by the employee and the employer (economy answers it).
pub(crate) fn deduction(
    employee: PersonId,
    employment: &Employment,
    amount: u64,
    shift: WorldTime,
) -> Emission {
    let employer = employment.job().employer();
    let who = employee.entity_id();
    Emission::new::<WageDeductionDue>(
        codec::encode(&WageDeductionDue::new(employee, employer, amount, shift)),
        Visibility::Participants,
    )
    .about(vec![who, employer.entity_id()])
    .with_participants(vec![who, employer.entity_id()])
}

/// The staff of the manager `manager` whose lateness may still be penalized at the read's instant, in
/// entity order; empty for anybody who manages nobody.
pub(crate) fn late_staff(world: &WorldRead<'_>, manager: EntityId) -> Vec<LateStaff> {
    let Some(mine) = world
        .component::<Employment>(manager)
        .filter(|employment| employment.job().manages())
    else {
        return Vec::new();
    };
    let employer = mine.job().employer();
    world
        .components::<Employment>()
        .filter(|(person, employment)| {
            *person != manager && employment.job().employer() == employer
        })
        .filter_map(|(person, employment)| {
            let window = parameters(world, person, employment.job()).window();
            let open = employment.open_at(world.at(), window)?;
            let employee = PersonId::new(person, EntityType::Person).ok()?;
            Some(LateStaff::new(employee, open))
        })
        .collect()
}

/// One complete `penalize` per open lateness, its amount the bound, to the manager only; target-less,
/// so a manager is offered it from anywhere.
pub(crate) fn offers(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() {
        return Vec::new();
    }
    late_staff(world, observer)
        .into_iter()
        .map(|late| {
            Offer::complete(
                &Penalize::new(late.employee(), late.lateness().bound()),
                PENALIZE_REQUIREMENT,
            )
            .expect("a penalty encodes")
        })
        .collect()
}

/// Reads a request's `penalize`; a payload that is not one is a precondition the request failed.
pub(crate) fn read_penalize(intent: &ActionIntent) -> Result<Penalize, Rejection> {
    let payload = intent
        .payload()
        .payload_for::<Penalize>()
        .map_err(|_| Rejection::PreconditionFailed)?;
    serde_json::from_slice(payload).map_err(|_| Rejection::PreconditionFailed)
}

/// Whether this penalty may happen, in order: the actor is the employee's employer's manager and not
/// the employee (`PermissionDenied`); an unpenalized lateness is open (`NoSupportedInteraction`);
/// `1 ≤ amount ≤ bound` (`penalty-out-of-bounds`, naming the bound). Requires nothing of space.
pub(crate) fn admit(
    world: &WorldRead<'_>,
    actor: EntityId,
    penalize: &Penalize,
) -> Result<(Employment, OpenLateness), Rejection> {
    let employee = penalize.employee().entity_id();
    let theirs = world
        .component::<Employment>(employee)
        .ok_or(Rejection::PermissionDenied)?;
    let manages = world.component::<Employment>(actor).is_some_and(|mine| {
        mine.job().manages() && mine.job().employer() == theirs.job().employer()
    });
    if !manages || actor == employee {
        return Err(Rejection::PermissionDenied);
    }
    let window = parameters(world, employee, theirs.job()).window();
    let open = theirs
        .open_at(world.at(), window)
        .ok_or(Rejection::NoSupportedInteraction)?;
    let amount = penalize.amount();
    if amount == 0 || amount > open.bound() {
        return Err(Rejection::System {
            code: PENALTY_OUT_OF_BOUNDS,
            detail: Some(format!("amount is 1 … {}, not {amount}", open.bound())),
        });
    }
    Ok((theirs.clone(), open))
}
