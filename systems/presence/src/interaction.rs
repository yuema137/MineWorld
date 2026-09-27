//! The seam that lets an observation carry what a player may do without this pack knowing what any
//! of it means.
//!
//! `ENGINEERING_RULES.md` §8 requires the server to answer whether an interaction is possible, and
//! §9 requires the 2D and the 3D client to get that answer from the same place. An
//! [`Observation`](mineworld_contracts::Observation) therefore carries
//! [`Affordance`](mineworld_contracts::Affordance)s, and something has to produce them.
//!
//! The wrong producer is this crate. A `match` on action names here would mean every new System
//! Pack edits perception — the change-amplification failure `ENGINEERING_STANDARDS.md` §8 names, and
//! the difference between a framework and a hardcoded game. So the producer is **each pack, about
//! its own actions**, through this trait; and what comes back is deliberately not an affordance but
//! an [`Offer`], because the two halves of the answer belong to different owners:
//!
//! ```text
//! the pack knows      which of its actions apply to this observer and this target,
//!                     what each requires of space, and whether the target is available
//! perception knows    where everybody is, and whether the world still provides the action
//! ```
//!
//! Neither half can produce an affordance alone, which is why the type system is arranged so that
//! neither tries.

use mineworld_contracts::{Action, ActionTypeId, EntityId, SpatialRequirement};
use mineworld_kernel::WorldRead;

/// What a System Pack says about its own actions, so that perception can price them.
///
/// Implemented by the pack that provides the actions, and called once per candidate target — plus
/// once with no target, for actions that are directed at nobody. A pack that offers nothing in
/// either case returns an empty list, which is the normal answer for most packs and most targets.
///
/// The provider is handed a [`WorldRead`] and nothing else, so an implementation can consult any
/// state it needs and can write none of it: deciding what is *possible* must not change the world,
/// for the same reason [`System::validate`](mineworld_kernel::System::validate) is handed a
/// read-only view (`BD-6`).
///
/// The value implementing this is the pack's own system type, which a world has already taken by
/// value at installation. Holding a second one to ask it questions is safe by construction rather
/// than by convention: a system's mutable state is the components it owns, held in the world, so
/// there is nothing in the value for two copies to disagree about (`INV-7`).
pub trait InteractionProvider {
    /// Which of this pack's actions `observer` may attempt against `target`, and what each needs.
    ///
    /// `target` is [`None`] for an action directed at nobody. The order of the returned offers is
    /// the order they reach the observation, so an implementation returns them in a fixed order —
    /// perception adds no sorting of its own, because an observation that reordered between two runs
    /// of the same world would break replay comparison (`AC-12`).
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer>;
}

/// One action a pack offers, before perception decides whether it is possible right now.
///
/// Not an [`Affordance`](mineworld_contracts::Affordance): an affordance carries a verdict, and the
/// pack that makes this offer cannot reach the positions the verdict depends on. What it carries is
/// exactly the three things only the owning pack knows.
///
/// The action type is read off `A` rather than passed as a value, so an offer cannot be
/// mislabelled — the same reason [`Emission::new`](mineworld_kernel::Emission::new) takes its event
/// type as a parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    action_type: ActionTypeId,
    requirement: SpatialRequirement,
    target_available: bool,
}

impl Offer {
    /// Offers `A`, with what it requires of space.
    ///
    /// The target is assumed available: an offer whose requirement does not ask about availability
    /// is unaffected by it, and a pack that does ask says so with
    /// [`Offer::with_target_available`]. Defaulting the other way would make every pack that does
    /// not model availability report its targets as unavailable.
    pub fn new<A: Action>(requirement: SpatialRequirement) -> Self {
        Self {
            action_type: A::ACTION_TYPE,
            requirement,
            target_available: true,
        }
    }

    /// States whether the target is available to be acted upon.
    ///
    /// Only the owning pack can answer this, and that is the point: what "available" means is a
    /// domain question — engaged, closed, asleep, mid-process — and the kernel must never learn any
    /// of those. The flag is consulted only when the requirement declares
    /// [`SpatialRequirement::requires_target_available`].
    #[must_use]
    pub const fn with_target_available(mut self, available: bool) -> Self {
        self.target_available = available;
        self
    }

    /// Which action is offered.
    pub const fn action_type(&self) -> &ActionTypeId {
        &self.action_type
    }

    /// What it requires of space, unevaluated — the value a client is shown as well as the one
    /// perception checks.
    pub const fn requirement(&self) -> SpatialRequirement {
        self.requirement
    }

    /// Whether the owning pack considers the target available.
    pub const fn target_available(&self) -> bool {
        self.target_available
    }
}
