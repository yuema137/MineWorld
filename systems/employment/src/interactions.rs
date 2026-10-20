//! This pack's section of the World's Interaction List (`docs/DECISIONS.md` `ARC-63`, `ARC-287`;
//! `configure/employment.yaml`): how patient an employer is about lateness, and whether lateness costs
//! money.
//!
//! ```yaml
//! parameters:
//!   - { grace_minutes: 0, window_days: 7, warning_at: 2, final_at: 3 }
//!   - { deduction_enabled: true, per_late_minute: 10, cap_per_shift: 600 }
//!   - { target: lenient-employer, grace_minutes: 10 }     # scoped by the employer's class
//! ```
//!
//! ```text
//! parameters   grace_minutes       minutes after the shift begins before arriving is late, 0 … 60, default 0
//!              window_days         days a lateness counts towards the next one's level, 1 … 60, default 7
//!              warning_at          the count at which a lateness is a warning, 1 … 20, default 2
//!              final_at            the count at which it is final, 1 … 20, default 3 (read as at least
//!                                  warning_at)
//!              deduction_enabled   whether lateness costs money, default false (`ARC-287`, I-LE2-4)
//!              per_late_minute     minor units per minute late, 0 … 10 000, default 10
//!              cap_per_shift       minor units at most per shift, 0 … 1 000 000, default 600
//! roles        actor = the employee, target = the employer, place = the workplace
//! ```
//!
//! With nothing configured every default holds: complaints, and no money moves.

use mineworld_contracts::{ComponentTypeId, EntityId, EventTypeId, PlaceId};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{self, InteractionSection, Resolved, Role, Roles};

use crate::component::{Job, Level};
use crate::system::EmploymentSystem;

mineworld_sdk::parameters! {
    /// What a world may choose about lateness and what it costs.
    pub struct EmploymentParameters, partial EmploymentPartial {
        /// Minutes after a shift begins before arriving counts as late.
        grace_minutes: u32 = 0, 0 ..= 60;
        /// Days a lateness counts towards the level of the next one.
        window_days: u32 = 7, 1 ..= 60;
        /// How many latenesses within the window make one a warning.
        warning_at: u32 = 2, 1 ..= 20;
        /// How many make one final; read as at least `warning_at`.
        final_at: u32 = 3, 1 ..= 20;
        /// Whether lateness costs the employee money. Off, it is a complaint only and no money moves.
        deduction_enabled: bool = false, false ..= true;
        /// Minor units deducted per started minute late.
        per_late_minute: u64 = 10, 0 ..= 10_000;
        /// The most minor units deducted for one shift.
        cap_per_shift: u64 = 600, 0 ..= 1_000_000;
    }
}

/// Seconds in a minute and in a day.
const MINUTE: u64 = 60;
const DAY: i64 = 86_400;

impl EmploymentParameters {
    /// Seconds after a shift's start before arriving is late.
    pub fn grace(&self) -> u64 {
        u64::from(self.grace_minutes) * MINUTE
    }

    /// The window, in seconds.
    pub fn window(&self) -> i64 {
        i64::from(self.window_days) * DAY
    }

    /// The level of the `count`th lateness within the window.
    pub fn level(&self, count: u32) -> Level {
        if count >= self.final_at.max(self.warning_at) {
            Level::Final
        } else if count >= self.warning_at {
            Level::Warning
        } else {
            Level::Notice
        }
    }

    /// What `late_by` seconds cost: `min(cap_per_shift, per_late_minute × ⌈late_by ÷ 60⌉)`.
    pub fn deduction(&self, late_by: u64) -> u64 {
        self.per_late_minute
            .saturating_mul(late_by.div_ceil(MINUTE))
            .min(self.cap_per_shift)
    }
}

/// The roles every parameter entry may scope by.
const ROLES: &[Role] = &[Role::Actor, Role::Target, Role::Place];

impl InteractionSection for EmploymentSystem {
    type Parameters = EmploymentParameters;
    type Knobs = ();
    const CONFIGURED: EventTypeId = EventTypeId::from_static("employment-interactions-configured");
    const COMPONENT: ComponentTypeId = ComponentTypeId::from_static("employment-interactions");
    const PARAMETER_ROLES: &'static [Role] = ROLES;

    fn encode(resolved: &Resolved<Self>) -> Vec<u8> {
        serde_json::to_vec(resolved).expect("a resolved section encodes")
    }

    fn decode(payload: &[u8]) -> Result<Resolved<Self>, String> {
        serde_json::from_slice(payload).map_err(|error| error.to_string())
    }
}

/// The parameters in force for `employee`'s `job`, at its workplace.
pub(crate) fn parameters(
    world: &WorldRead<'_>,
    employee: EntityId,
    job: &Job,
) -> EmploymentParameters {
    let roles = Roles::new()
        .with(Role::Actor, employee)
        .with(Role::Target, job.employer().entity_id());
    let place: PlaceId = job.workplace();
    interactions::parameters::<EmploymentSystem>(world, place, &roles)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every default is the one §0.6 fixed before any run; deduction is off (I-LE2-4).
    #[test]
    fn the_defaults_complain_and_move_no_money() {
        let defaults = EmploymentParameters::default();
        assert!(!defaults.deduction_enabled);
        assert_eq!(
            (
                defaults.grace(),
                defaults.window(),
                defaults.warning_at,
                defaults.final_at
            ),
            (0, 7 * DAY, 2, 3)
        );
        assert_eq!(
            [1, 2, 3, 4].map(|count| defaults.level(count)),
            [Level::Notice, Level::Warning, Level::Final, Level::Final]
        );
        // 10 per started minute, capped at 600.
        assert_eq!(defaults.deduction(1), 10);
        assert_eq!(defaults.deduction(60), 10);
        assert_eq!(defaults.deduction(61), 20);
        assert_eq!(defaults.deduction(720), 120);
        assert_eq!(defaults.deduction(4 * 3_600), 600);
    }

    /// A final threshold below the warning threshold is read as the warning threshold.
    #[test]
    fn a_final_threshold_below_the_warning_threshold_is_the_warning_threshold() {
        let parameters = EmploymentParameters {
            warning_at: 4,
            final_at: 2,
            ..EmploymentParameters::default()
        };
        assert_eq!(parameters.level(3), Level::Notice);
        assert_eq!(parameters.level(4), Level::Final);
    }
}
