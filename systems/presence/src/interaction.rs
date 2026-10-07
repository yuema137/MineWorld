//! The seam that lets an observation carry what a player may do and what a player may know, without
//! this pack knowing what any of it means.
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
//!
//! # State is the same question with a different noun
//!
//! An observation also carries **component records**, and the tempting implementation of *that* is
//! worse than a match on action names: walk the component stores and serialize what is there. It
//! would defeat `INV-13` for every pack written afterwards, because exposing a component on the
//! grounds that it exists is precisely the omniscience the invariant forbids, and this crate cannot
//! judge what any other pack's state means or who may read it.
//!
//! So the same seam asks the same kind of question about state, and the owning pack answers it per
//! observer:
//!
//! ```text
//! the pack knows      which of its components this observer may know about this subject, and
//!                     therefore what to encode
//! perception knows    which entities this observer perceives at all, and asks about those only
//! ```
//!
//! What follows from that division is the property `AC-2` wants: a disabled pack's state leaves
//! every observation in the world with no edit anywhere, exactly as its affordances do.
//!
//! The division is not symmetrical, and deliberately: **what** may be known is the owning pack's
//! judgement, while **whom a record may be about** is not. Perception bounds the second, because it
//! can do so without knowing what any component is and because an invariant that rests on every
//! future pack being honest is a convention rather than a construction. See
//! [`PerceptionProvider::discloses`] and [`observe`](crate::observe).

use mineworld_contracts::{Action, ActionTypeId, ComponentRecord, EntityId, SpatialRequirement};
use mineworld_kernel::WorldRead;
use serde_json::Value;

/// What a System Pack says about itself, so that perception can put it in an observation.
///
/// Two questions, both of which only the owning pack can answer, and neither of which this crate
/// could answer without learning what another pack's vocabulary means:
///
/// ```text
/// offers      which of my actions may this observer attempt against that target
/// discloses   which of my components may this observer know about that subject
/// ```
///
/// Both have a default returning nothing, because most packs answer one of them and a pack that
/// answers neither is still a legitimate pack — it simply contributes nothing to an observation.
/// Defaulting to *nothing* rather than to *everything* is the same safe direction
/// [`Observation::new`](mineworld_contracts::Observation::new) takes.
///
/// The provider is handed a [`WorldRead`] and nothing else, so an implementation can consult any
/// state it needs and can write none of it: deciding what is *possible*, or what may be *known*,
/// must not change the world, for the same reason
/// [`System::validate`](mineworld_kernel::System::validate) is handed a read-only view (`BD-6`).
///
/// The value implementing this is the pack's own system type, which a world has already taken by
/// value at installation. Holding a second one to ask it questions is safe by construction rather
/// than by convention: a system's mutable state is the components it owns, held in the world, so
/// there is nothing in the value for two copies to disagree about (`INV-7`).
pub trait PerceptionProvider {
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
    ) -> Vec<Offer> {
        let _ = (world, observer, target);
        Vec::new()
    }

    /// Which of this pack's components `observer` is entitled to know about `subject`, encoded.
    ///
    /// Called once per entity the observer already perceives, including the observer itself, so an
    /// implementation never has to decide whether the subject is visible — only whether this
    /// observer may know *this* state about it. A pack that discloses state about a stranger and
    /// state about oneself differently says so by comparing the two arguments, which is the whole
    /// reason both are passed.
    ///
    /// The payload is a [`Value`] rather than bytes because its reader is a controller or a client
    /// that does not have the Rust type: an observation is read, while a log is replayed, and
    /// `spike/FINDINGS.md` F8.2 measured what opaque bytes reach a client as. Encoding it is the
    /// owning pack's own business, as every payload in this workspace is.
    ///
    /// # Every record must be about `subject`
    ///
    /// A record naming anybody else is **dropped**, silently and per record. That is not a courtesy
    /// check on a well-behaved implementation: without it, *whom* a record may be about would rest
    /// on every provider choosing to be honest, and this trait is extension surface — `ARC-8` makes
    /// a Tier 1 pack a WASM component, which is not code this repository wrote. So an implementation
    /// that is asked about one person and answers about another discloses nothing at all, rather
    /// than leaking a third party's state to a client (`INV-13`).
    ///
    /// Returning nothing is the normal answer, and the default.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let _ = (world, observer, subject);
        Vec::new()
    }
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
///
/// An offer may also be **complete** ([`Offer::complete`], `DECISIONS.md` `ARC-34`): it carries the
/// exact request the pack would accept, so a requester that was never compiled against the pack can
/// still attempt it. Perception carries that payload into the affordance and decides nothing about
/// what it means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    action_type: ActionTypeId,
    requirement: SpatialRequirement,
    target_available: bool,
    payload: Option<Value>,
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
            payload: None,
        }
    }

    /// Offers exactly this request: a **complete** offer, whose payload a requester may submit
    /// unchanged without knowing the action.
    ///
    /// The action type is read off the value's own type, exactly as [`Offer::new`] reads it, so a
    /// payload of one action cannot be attached to an offer of another — there is no second
    /// argument to get wrong. The payload is encoded as the observation carries payloads, a JSON
    /// value; the only failure is a value JSON cannot represent, which no action in this workspace
    /// is.
    ///
    /// A pack offers one complete offer per choice it would accept — which is why only a pack whose
    /// choices it can enumerate can make one. Whatever is submitted is still validated by the pack
    /// at dispatch.
    pub fn complete<A: Action>(
        action: &A,
        requirement: SpatialRequirement,
    ) -> Result<Self, serde_json::Error> {
        let payload = serde_json::to_value(action)?;
        Ok(Self {
            payload: Some(payload),
            ..Self::new::<A>(requirement)
        })
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

    /// The complete request, when this is a complete offer.
    pub const fn payload(&self) -> Option<&Value> {
        self.payload.as_ref()
    }
}
