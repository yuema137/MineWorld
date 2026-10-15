//! `rule`: the paced rule controller as a decision model, the first and the default
//! (`pr-dl-a-decision-list.md` SD-DLa-2; `docs/MODULE_SPEC.md` §5.1).
//!
//! Zero-sized and parameterless. Each consult builds `PacedRuleController::new(seed, pace)` from the host's
//! [`Consult`] and asks it — the controller is `Copy` and two words, so building it costs nothing, and the
//! controller, its tests and its draws stay exactly as they were. With no decision list every seat is this
//! model, and every byte of every world is what it was before decision lists existed.

use mineworld_contracts::ActionRequest;
use mineworld_decision::{Consult, DecisionModel};

use crate::PacedRuleController;

/// The paced rule controller, `rule` in `decisions.yaml`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rule;

impl Rule {
    fn controller(consult: &Consult<'_>) -> PacedRuleController {
        PacedRuleController::new(consult.seed.raw(), consult.pace)
    }
}

impl DecisionModel for Rule {
    const ID: &'static str = "rule";
    const PACKAGE: mineworld_packages::Package = crate::PACKAGE;
    type Parameters = ();

    fn decide(&self, (): &(), consult: &Consult<'_>) -> Option<ActionRequest> {
        Self::controller(consult).decide(consult.observation)
    }

    fn step(&self, (): &(), consult: &Consult<'_>) -> Option<ActionRequest> {
        Self::controller(consult).step(consult.observation)
    }
}
