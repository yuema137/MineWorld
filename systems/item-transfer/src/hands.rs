//! Putting a thing down in the place one stands in, and picking one up from it (S35 N-h R4;
//! `pr-s35-nh-hands-and-errands.md` §3.2 (b), Q-Nh-1).
//!
//! A person carries at most a few things (`mineworld_inventory::PERSON_CAPACITY`). A bound on carrying
//! with no way to put a thing down leaves a full person stuck for ever: every `buy` is refused
//! `cannot-carry`, and the only way out of one's hands was a `give` to somebody present with room
//! (F-Ng-2). Every carrying model the design compared has a way to put things down; this is ours.
//!
//! ```text
//! put-down { item }   one of a held kind, from me to the place I stand in; offered once per held kind
//! pick-up { item }    one from the place I stand in, to me; offered once per kind the place holds,
//!                     refused `cannot-carry` when I cannot take one more
//! requirement         in that place
//! states              inventory's items-transferred { from, to }, heard in the place
//! ```
//!
//! Items are conserved: nothing is created or destroyed here, so a town's total never grows.

use mineworld_contracts::{
    Action, ActionTypeId, EntityId, ItemId, PlaceId, Rejection, RejectionCode, SpatialRequirement,
    SystemId,
};
use mineworld_inventory::Holdings;
use mineworld_kernel::{Emission, SystemIdentity, WorldRead};
use mineworld_presence::{Offer, Presence};
use serde::{Deserialize, Serialize};

use crate::offer::living_person;
use crate::system::ItemTransferSystem;

/// Put one `item` down in the place I stand in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutDown {
    item: ItemId,
}

impl Action for PutDown {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("put-down");
    const OWNER: SystemId = ItemTransferSystem::ID;
}

impl PutDown {
    /// Put one `item` down.
    pub const fn new(item: ItemId) -> Self {
        Self { item }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}

/// Pick one `item` up from the place I stand in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickUp {
    item: ItemId,
}

impl Action for PickUp {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("pick-up");
    const OWNER: SystemId = ItemTransferSystem::ID;
}

impl PickUp {
    /// Pick one `item` up.
    pub const fn new(item: ItemId) -> Self {
        Self { item }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }
}

/// The picker cannot carry one more.
pub const CANNOT_CARRY: RejectionCode = RejectionCode::from_static("cannot-carry");
/// The place holds none of the kind.
pub const NOT_HERE: RejectionCode = RejectionCode::from_static("not-here");

const fn refusal(code: RejectionCode) -> Rejection {
    Rejection::System { code, detail: None }
}

/// What putting down or picking up requires of space: standing in `place`.
pub const fn hands_requirement(place: PlaceId) -> SpatialRequirement {
    SpatialRequirement::at_place(place)
}

/// The place a living person stands in.
fn standing_in(world: &WorldRead<'_>, person: EntityId) -> Result<PlaceId, Rejection> {
    if !living_person(world, person) {
        return Err(Rejection::NoSupportedInteraction);
    }
    world
        .component::<Presence>(person)
        .map(|presence| presence.location().place())
        .ok_or(Rejection::PreconditionFailed)
}

fn count(world: &WorldRead<'_>, holder: EntityId, item: ItemId) -> u32 {
    world
        .component::<Holdings>(holder)
        .map_or(0, |held| held.count(item))
}

/// The one judgement of a `put-down`, for the offer and the dispatch: a living person standing in a
/// place, holding one of `item`; then inventory's own rule. The place it goes to.
pub(crate) fn judge_put_down(
    world: &WorldRead<'_>,
    actor: EntityId,
    item: ItemId,
) -> Result<PlaceId, Rejection> {
    let place = standing_in(world, actor)?;
    if count(world, actor, item) == 0 {
        return Err(Rejection::PreconditionFailed);
    }
    mineworld_inventory::admit_transfer(world, actor, place.entity_id(), item, 1)?;
    Ok(place)
}

/// The one judgement of a `pick-up`: a living person standing in a place that holds one of `item`
/// (`not-here` otherwise), who can take one more (`cannot-carry`); then inventory's own rule. The
/// place it comes from.
pub(crate) fn judge_pick_up(
    world: &WorldRead<'_>,
    actor: EntityId,
    item: ItemId,
) -> Result<PlaceId, Rejection> {
    let place = standing_in(world, actor)?;
    if count(world, place.entity_id(), item) == 0 {
        return Err(refusal(NOT_HERE));
    }
    if !mineworld_inventory::can_take(world, actor, 1) {
        return Err(refusal(CANNOT_CARRY));
    }
    mineworld_inventory::admit_transfer(world, place.entity_id(), actor, item, 1)?;
    Ok(place)
}

/// The fact a judged `put-down` or `pick-up` states: inventory's `items-transferred`, heard in the
/// place, through its checked constructor.
pub(crate) fn moved(
    world: &WorldRead<'_>,
    from: EntityId,
    to: EntityId,
    item: ItemId,
    place: PlaceId,
) -> Result<Emission, Rejection> {
    mineworld_inventory::transfer_in_place(world, from, to, item, 1, place)
}

/// What `observer` is offered with no target: one complete `put-down` per kind held, then one complete
/// `pick-up` per kind the place holds, each refused with its reason, in item order (`AC-12`).
pub(crate) fn offers(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if target.is_some() {
        return Vec::new();
    }
    let Ok(place) = standing_in(world, observer) else {
        return Vec::new();
    };
    let requirement = hands_requirement(place);
    let mut offered = Vec::new();
    if let Some(held) = world.component::<Holdings>(observer) {
        for line in held.held() {
            let offer = Offer::complete(&PutDown::new(line.item()), requirement)
                .expect("a put-down encodes");
            offered.push(match judge_put_down(world, observer, line.item()) {
                Ok(_) => offer,
                Err(reason) => offer.refused(reason),
            });
        }
    }
    if let Some(lying) = world.component::<Holdings>(place.entity_id()) {
        for line in lying.held() {
            let offer =
                Offer::complete(&PickUp::new(line.item()), requirement).expect("a pick-up encodes");
            offered.push(match judge_pick_up(world, observer, line.item()) {
                Ok(_) => offer,
                Err(reason) => offer.refused(reason),
            });
        }
    }
    offered
}
