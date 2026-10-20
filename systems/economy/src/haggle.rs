//! Haggling (`ARC-118`; `pr-s31h-haggling.md` §§5.4–5.8): a buyer proposes a price for one unit, the
//! seller — or a keeper standing in the shop — accepts or declines it, and the buyer may withdraw it.
//! Acceptance is a sale at the proposed price, judged by the same [`judge_exchange`] a `buy` runs and
//! stated by the same [`settle`]. Nothing is reserved at any time.
//!
//! Each judge here is the one function both the offer and the dispatch call, in a fixed order, so two
//! reasons at once always report the same name.

use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, EntityId, EntityType, Event, ItemId, PersonId, PlaceId,
    Rejection, SpatialRequirement, SystemId, Visibility,
};
use mineworld_kernel::{Emission, KernelError, SystemIdentity, WorldRead};
use mineworld_presence::{Offer, Presence};
use serde::{Deserialize, Serialize};

use crate::action::person_buy_requirement;
use crate::code::{self, NO_SUCH_PROPOSAL, PRICE_OUT_OF_RANGE};
use crate::codec;
use crate::component::{Asks, Shop};
use crate::event::{
    PriceProposed, ProposalAccepted, ProposalDeclined, ProposalTerms, ProposalWithdrawn,
};
use crate::money::price_in_range;
use crate::offer::{asks_of, living_person, shop_here};
use crate::proposal::{PROPOSAL_VALID_FOR, Proposal, ProposalId, find_open, proposals_of};
use crate::sale::{
    Sale, Seller, judge_exchange, judge_goods, judge_self_trade, located, resolve_seller, settle,
};
use crate::system::{EconomySystem, refused};

/// Propose to buy one `item` at `price` (`ARC-118`): with no target from the shop the buyer stands
/// in, with a target from that person seller. Offered **incomplete**: the price is the buyer's choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposePrice {
    item: ItemId,
    price: u64,
}

impl Action for ProposePrice {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("propose-price");
    const OWNER: SystemId = EconomySystem::ID;
}

impl ProposePrice {
    /// Propose `price` for one `item`.
    pub const fn new(item: ItemId, price: u64) -> Self {
        Self { item, price }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The proposed price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }
}

/// Declares one of the three actions that name a proposal: accept, decline, withdraw.
macro_rules! proposal_action {
    ($(#[$doc:meta])* $name:ident, $action_type:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            proposal: ProposalId,
        }

        impl Action for $name {
            const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static($action_type);
            const OWNER: SystemId = EconomySystem::ID;
        }

        impl $name {
            /// The request naming `proposal`.
            pub const fn new(proposal: ProposalId) -> Self {
                Self { proposal }
            }

            /// The proposal it names.
            pub const fn proposal(&self) -> ProposalId {
                self.proposal
            }
        }
    };
}

proposal_action!(
    /// Accept an open proposal (`ARC-118`): targeted at its buyer, by the person seller or a keeper
    /// standing in the proposal's shop. Settles at once as a sale at the proposed price. Offered
    /// **incomplete** (Q-S31a-3).
    AcceptProposal,
    "accept-proposal"
);

proposal_action!(
    /// Decline an open proposal (`ARC-118`), by the same people who may accept it. Offered
    /// **incomplete** (Q-S31a-3).
    DeclineProposal,
    "decline-proposal"
);

proposal_action!(
    /// Take back one's own open proposal (`ARC-118`). Offered **incomplete** (Q-S31h-1).
    WithdrawProposal,
    "withdraw-proposal"
);

/// Whether `action` is one of the four haggling actions.
pub(crate) fn handles(action: &ActionTypeId) -> bool {
    [
        ProposePrice::ACTION_TYPE,
        AcceptProposal::ACTION_TYPE,
        DeclineProposal::ACTION_TYPE,
        WithdrawProposal::ACTION_TYPE,
    ]
    .contains(action)
}

// ---------------------------------------------------------------------------------------------
// the judges (§5.5)
// ---------------------------------------------------------------------------------------------

/// `propose-price`'s judgement: [`resolve_seller`] (1–3); the price in range (4); no self-trade (5);
/// the limits (6); funds, stock, carry and the owners' rules at the proposed price (7–9). The
/// requirement is the caller's, after.
fn judge_proposal(
    world: &WorldRead<'_>,
    buyer: EntityId,
    target: Option<EntityId>,
    propose: &ProposePrice,
) -> Result<Sale, Rejection> {
    let (seller, _ask) = resolve_seller(world, buyer, target, propose.item)?;
    if !price_in_range(propose.price) {
        return Err(code::refusal(PRICE_OUT_OF_RANGE));
    }
    let sale = Sale {
        seller,
        buyer,
        item: propose.item,
        price: propose.price,
    };
    judge_self_trade(world, &sale)?;
    if let Some(limit) = proposals_of(world, seller.entity()).limit(buyer, propose.item, world.at())
    {
        return Err(limit.refusal());
    }
    judge_goods(world, &sale)?;
    Ok(sale)
}

/// Who answers a proposal, and the sale acceptance would be.
struct Answer {
    seller: Seller,
    proposal: Proposal,
}

impl Answer {
    fn sale(&self) -> Sale {
        Sale {
            seller: self.seller,
            buyer: self.proposal.buyer().entity_id(),
            item: self.proposal.item(),
            price: self.proposal.price(),
        }
    }

    fn terms(&self) -> ProposalTerms {
        ProposalTerms::new(
            self.proposal.id(),
            self.seller.entity(),
            self.proposal.buyer(),
            self.proposal.item(),
            self.proposal.price(),
        )
    }

    /// What accepting requires of space: the buyer within reach of a person seller, or in the shop
    /// with its keeper.
    fn requirement(&self) -> SpatialRequirement {
        match self.seller {
            Seller::Person(_) => person_buy_requirement(),
            Seller::Shop { .. } => SpatialRequirement::same_place().requiring_target_available(),
        }
    }
}

/// Steps 1–3 of an answer: the actor a living Person; `proposal` open now; the actor the person
/// seller, or a keeper standing in the proposal's shop.
fn judge_authority(
    world: &WorldRead<'_>,
    actor: EntityId,
    proposal: ProposalId,
) -> Result<Answer, Rejection> {
    if !living_person(world, actor) {
        return Err(Rejection::NoSupportedInteraction);
    }
    let (seller, entry) =
        find_open(world, proposal).ok_or_else(|| code::refusal(NO_SUCH_PROPOSAL))?;
    authority(world, actor, seller, entry)
}

/// Step 3 for one open entry held by `seller`.
fn authority(
    world: &WorldRead<'_>,
    actor: EntityId,
    seller: EntityId,
    entry: Proposal,
) -> Result<Answer, Rejection> {
    let seller = match entry.place() {
        None => {
            if actor != seller {
                return Err(Rejection::PermissionDenied);
            }
            Seller::Person(seller)
        }
        Some(place) => {
            let (here, shop) = shop_here(world, actor).ok_or(Rejection::NoSupportedInteraction)?;
            if here != place || shop.operator().entity_id() != seller {
                return Err(Rejection::NoSupportedInteraction);
            }
            if !shop.kept_by(actor) {
                return Err(Rejection::PermissionDenied);
            }
            Seller::Shop {
                place,
                operator: seller,
            }
        }
    };
    Ok(Answer {
        seller,
        proposal: entry,
    })
}

/// `accept-proposal`'s judgement, steps 1–5: authority; the target the proposal's buyer; and every
/// `buy` check at the proposed price through [`judge_exchange`]. The current ask is not read
/// (SD-S31h-7).
fn judge_acceptance(
    world: &WorldRead<'_>,
    actor: EntityId,
    target: Option<EntityId>,
    proposal: ProposalId,
) -> Result<Answer, Rejection> {
    let answer = judge_authority(world, actor, proposal)?;
    if target != Some(answer.proposal.buyer().entity_id()) {
        return Err(Rejection::PreconditionFailed);
    }
    judge_exchange(world, &answer.sale())?;
    Ok(answer)
}

/// The requirement of an acceptance, evaluated from where the actor and the buyer stand.
fn accept_reach(world: &WorldRead<'_>, actor: EntityId, answer: &Answer) -> Result<(), Rejection> {
    let here = located(world, actor).ok_or(Rejection::PreconditionFailed)?;
    let there = located(world, answer.proposal.buyer().entity_id());
    let available = living_person(world, answer.proposal.buyer().entity_id());
    answer
        .requirement()
        .evaluate(&here, there.as_ref(), available)
}

/// `withdraw-proposal`'s judgement: a living Person; the proposal open; the actor its buyer.
fn judge_withdrawal(
    world: &WorldRead<'_>,
    actor: EntityId,
    proposal: ProposalId,
) -> Result<(EntityId, Proposal), Rejection> {
    if !living_person(world, actor) {
        return Err(Rejection::NoSupportedInteraction);
    }
    let (seller, entry) =
        find_open(world, proposal).ok_or_else(|| code::refusal(NO_SUCH_PROPOSAL))?;
    if entry.buyer().entity_id() != actor {
        return Err(Rejection::PermissionDenied);
    }
    Ok((seller, entry))
}

// ---------------------------------------------------------------------------------------------
// validate and resolve
// ---------------------------------------------------------------------------------------------

/// Whether a haggling request may happen: its judge, then its requirement.
pub(crate) fn validate(world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
    let (action, actor, target) = (intent.action_type(), intent.actor(), intent.target());
    if *action == ProposePrice::ACTION_TYPE {
        let propose: ProposePrice = codec::read(intent)?;
        let sale = judge_proposal(world, actor, target, &propose)?;
        let here = located(world, actor).ok_or(Rejection::PreconditionFailed)?;
        let there = target.and_then(|target| located(world, target));
        return sale.requirement().evaluate(&here, there.as_ref(), true);
    }
    if *action == AcceptProposal::ACTION_TYPE {
        let accept: AcceptProposal = codec::read(intent)?;
        let answer = judge_acceptance(world, actor, target, accept.proposal)?;
        return accept_reach(world, actor, &answer);
    }
    if target.is_some() {
        return Err(Rejection::NoSupportedInteraction);
    }
    if *action == DeclineProposal::ACTION_TYPE {
        let decline: DeclineProposal = codec::read(intent)?;
        return judge_authority(world, actor, decline.proposal).map(|_| ());
    }
    let withdraw: WithdrawProposal = codec::read(intent)?;
    judge_withdrawal(world, actor, withdraw.proposal).map(|_| ())
}

/// A haggling request's facts, re-judged from the state at resolve; a refusal here is the owner's
/// `FactRefusedByOwner`.
pub(crate) fn resolve(
    world: &WorldRead<'_>,
    intent: &ActionIntent,
) -> Result<Vec<Emission>, KernelError> {
    let (action, actor, target) = (intent.action_type(), intent.actor(), intent.target());
    if *action == ProposePrice::ACTION_TYPE {
        let refusal = |reason| refused(PriceProposed::EVENT_TYPE, reason);
        let propose: ProposePrice = codec::read(intent).map_err(refusal)?;
        let sale = judge_proposal(world, actor, target, &propose).map_err(refusal)?;
        let buyer = PersonId::new(actor, EntityType::Person)
            .map_err(|_| refusal(Rejection::PreconditionFailed))?;
        let terms = ProposalTerms::new(
            ProposalId::of(intent.action_id()),
            sale.seller.entity(),
            buyer,
            sale.item,
            sale.price,
        );
        let place = sale.seller.shop_place();
        let valid_until = world
            .at()
            .seconds()
            .checked_add(PROPOSAL_VALID_FOR.seconds())
            .map(mineworld_contracts::WorldTime::from_seconds)
            .ok_or_else(|| refusal(Rejection::PreconditionFailed))?;
        let fact = PriceProposed::new(terms, place, valid_until);
        let emission = heard_by::<PriceProposed>(world, &fact, &terms, sale.seller);
        return Ok(vec![emission]);
    }
    if *action == AcceptProposal::ACTION_TYPE {
        let refusal = |reason| refused(ProposalAccepted::EVENT_TYPE, reason);
        let accept: AcceptProposal = codec::read(intent).map_err(refusal)?;
        let answer = judge_acceptance(world, actor, target, accept.proposal).map_err(refusal)?;
        let terms = answer.terms();
        let mut facts = vec![heard_by::<ProposalAccepted>(
            world,
            &ProposalAccepted::new(terms),
            &terms,
            answer.seller,
        )];
        facts.extend(settle(world, &answer.sale())?);
        return Ok(facts);
    }
    if *action == DeclineProposal::ACTION_TYPE {
        let refusal = |reason| refused(ProposalDeclined::EVENT_TYPE, reason);
        let decline: DeclineProposal = codec::read(intent).map_err(refusal)?;
        let answer = judge_authority(world, actor, decline.proposal).map_err(refusal)?;
        let terms = answer.terms();
        return Ok(vec![heard_by::<ProposalDeclined>(
            world,
            &ProposalDeclined::new(terms),
            &terms,
            answer.seller,
        )]);
    }
    let refusal = |reason| refused(ProposalWithdrawn::EVENT_TYPE, reason);
    let withdraw: WithdrawProposal = codec::read(intent).map_err(refusal)?;
    let (seller, entry) = judge_withdrawal(world, actor, withdraw.proposal).map_err(refusal)?;
    let seller = match entry.place() {
        Some(place) => Seller::Shop {
            place,
            operator: seller,
        },
        None => Seller::Person(seller),
    };
    let terms = ProposalTerms::new(
        entry.id(),
        seller.entity(),
        entry.buyer(),
        entry.item(),
        entry.price(),
    );
    Ok(vec![heard_by::<ProposalWithdrawn>(
        world,
        &ProposalWithdrawn::new(terms),
        &terms,
        seller,
    )])
}

/// A proposal fact, heard by its participants only (§5.7): the buyer, and the person seller or the
/// keepers standing in the shop now; about the buyer and the seller.
fn heard_by<E: Event>(
    world: &WorldRead<'_>,
    fact: &impl Serialize,
    terms: &ProposalTerms,
    seller: Seller,
) -> Emission {
    let buyer = terms.buyer().entity_id();
    let mut participants = vec![buyer];
    match seller {
        Seller::Person(person) => participants.push(person),
        Seller::Shop { place, .. } => participants.extend(keepers_in(world, place)),
    }
    participants.sort_unstable();
    participants.dedup();
    let emission = Emission::new::<E>(codec::encode(fact), Visibility::Participants)
        .about(vec![buyer, seller.entity()])
        .with_participants(participants);
    match seller.shop_place() {
        Some(place) => emission.at_place(place),
        None => emission,
    }
}

/// The keepers of the shop at `place` standing in it now, in id order.
fn keepers_in(world: &WorldRead<'_>, place: PlaceId) -> Vec<EntityId> {
    let Some(shop) = world.component::<Shop>(place.entity_id()) else {
        return Vec::new();
    };
    let mut here: Vec<EntityId> = shop
        .keepers()
        .iter()
        .map(|keeper| keeper.entity_id())
        .filter(|keeper| {
            world
                .component::<Presence>(*keeper)
                .is_some_and(|presence| presence.location().place() == place)
        })
        .collect();
    here.sort_unstable();
    here
}

// ---------------------------------------------------------------------------------------------
// offers (§5.8): every one incomplete
// ---------------------------------------------------------------------------------------------

/// The haggling actions `observer` is offered toward `target`, each **incomplete** (Q-S31a-3,
/// Q-S31h-1): with no target, a `propose-price` to the shop it stands in, one `decline-proposal` when
/// it may answer an open proposal, one `withdraw-proposal` when it has one open; with a person target,
/// a `propose-price` to that seller and an `accept-proposal` for that buyer's open proposals.
pub(crate) fn offers(
    world: &WorldRead<'_>,
    observer: EntityId,
    target: Option<EntityId>,
) -> Vec<Offer> {
    if !living_person(world, observer) {
        return Vec::new();
    }
    let mut offers = Vec::new();
    match target {
        None => {
            if let Some((place, shop)) = shop_here(world, observer) {
                let operator = shop.operator().entity_id();
                if !asks_of(world, operator).asks().is_empty() {
                    let offer = Offer::new::<ProposePrice>(crate::action::buy_requirement(place));
                    let refusal = if shop.kept_by(observer) || operator == observer {
                        Some(Rejection::PreconditionFailed)
                    } else {
                        full(world, operator)
                    };
                    offers.push(with_refusal(offer, refusal));
                }
            }
            if !answerable(world, observer).is_empty() {
                offers.push(Offer::new::<DeclineProposal>(SpatialRequirement::NONE));
            }
            let now = world.at();
            let has_open = world
                .components::<crate::proposal::Proposals>()
                .any(|(_, held)| {
                    held.live(now)
                        .any(|entry| entry.is_open(now) && entry.buyer().entity_id() == observer)
                });
            if has_open {
                offers.push(Offer::new::<WithdrawProposal>(SpatialRequirement::NONE));
            }
        }
        Some(person) => {
            if person == observer || !living_person(world, person) {
                return offers;
            }
            if world
                .component::<Asks>(person)
                .is_some_and(|asks| !asks.asks().is_empty())
            {
                let offer = Offer::new::<ProposePrice>(person_buy_requirement());
                offers.push(with_refusal(offer, full(world, person)));
            }
            if let Some(offer) = accept_offer(world, observer, person) {
                offers.push(offer);
            }
        }
    }
    offers
}

fn with_refusal(offer: Offer, refusal: Option<Rejection>) -> Offer {
    match refusal {
        Some(reason) => offer.refused(reason),
        None => offer,
    }
}

/// `seller-full` when `seller` holds eight live proposals.
fn full(world: &WorldRead<'_>, seller: EntityId) -> Option<Rejection> {
    proposals_of(world, seller)
        .full(world.at())
        .then(|| crate::proposal::Limit::SellerFull.refusal())
}

/// The open proposals `observer` may answer now (steps 1–3), in id order.
fn answerable(world: &WorldRead<'_>, observer: EntityId) -> Vec<Answer> {
    let now = world.at();
    let mut answers: Vec<Answer> = world
        .components::<crate::proposal::Proposals>()
        .flat_map(|(seller, held)| {
            held.live(now)
                .filter(move |entry| entry.is_open(now))
                .map(move |entry| (seller, *entry))
        })
        .filter_map(|(seller, entry)| authority(world, observer, seller, entry).ok())
        .collect();
    answers.sort_by_key(|answer| answer.proposal.id());
    answers
}

/// One `accept-proposal` toward `buyer` when `observer` may answer one of its open proposals:
/// available when one passes steps 1–5, else refused with the code of the lowest-id one.
fn accept_offer(world: &WorldRead<'_>, observer: EntityId, buyer: EntityId) -> Option<Offer> {
    let theirs: Vec<Answer> = answerable(world, observer)
        .into_iter()
        .filter(|answer| answer.proposal.buyer().entity_id() == buyer)
        .collect();
    let first = theirs.first()?;
    let offer = Offer::new::<AcceptProposal>(first.requirement());
    let verdicts: Vec<Result<(), Rejection>> = theirs
        .iter()
        .map(|answer| judge_exchange(world, &answer.sale()))
        .collect();
    if verdicts.iter().any(Result::is_ok) {
        return Some(offer);
    }
    let reason = verdicts
        .into_iter()
        .find_map(Result::err)
        .expect("at least one verdict");
    Some(offer.refused(reason))
}
