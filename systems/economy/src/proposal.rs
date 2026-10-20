//! Proposals: a buyer's offer to buy one unit of one kind from one seller at one price, kept on the
//! seller until it is answered, withdrawn or expires (`ARC-118`; `pr-s31h-haggling.md` §§5.3, 5.7,
//! 5.9, 5.10).
//!
//! Nothing here holds money or an item: a proposal reserves nothing. Expiry is lazy — no wake and no
//! fact; every reader compares the world's time with `valid_until`, and every reduction on a seller
//! first drops its dead entries.

use mineworld_contracts::{
    ActionId, EntityId, EntityType, Event, ItemId, LifecycleState, PersonId, PlaceId, Rejection,
    SimDuration, WorldTime,
};
use mineworld_kernel::{KernelError, WorldRead, WorldView, owned_component};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::event::{PriceProposed, ProposalAccepted, ProposalDeclined, ProposalWithdrawn};
use crate::money::price_in_range;
use crate::offer::{asking_price, asks_of, living_person, shop_here};
use crate::system::{EconomySystem, refused};

/// How long a proposal stays open: 15 world minutes (step-31 §5.9).
pub const PROPOSAL_VALID_FOR: SimDuration = SimDuration::from_seconds(900);

/// How many live proposals one seller may hold at once (step-31 §5.9).
pub const PROPOSALS_PER_SELLER: usize = 8;

/// A proposal's identity: the `ActionId` of the `propose-price` request that made it (SD-S31h-1) —
/// unique, allocated by the world and journaled, so economy keeps no counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProposalId(u64);

impl ProposalId {
    /// The proposal made by request `action`.
    pub const fn of(action: ActionId) -> Self {
        Self(action.raw())
    }

    /// Rebuilds an identity read from a record, a fact or a request.
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// The underlying value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Whether a live proposal still waits for an answer, or was declined and is kept so its buyer sees
/// the "no" until it expires (Q-S31h-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    /// Waiting for the seller's answer.
    Open,
    /// The seller said no.
    Declined,
}

/// One buyer's proposal to one seller, as the seller's [`Proposals`] keep it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    id: ProposalId,
    buyer: PersonId,
    item: ItemId,
    price: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    place: Option<PlaceId>,
    made_at: WorldTime,
    valid_until: WorldTime,
    status: ProposalStatus,
}

impl Proposal {
    /// Its identity.
    pub const fn id(&self) -> ProposalId {
        self.id
    }

    /// Who proposes.
    pub const fn buyer(&self) -> PersonId {
        self.buyer
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The proposed price for one unit, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// The shop's place, for a shop's proposal; none for a person seller's.
    pub const fn place(&self) -> Option<PlaceId> {
        self.place
    }

    /// When it was made.
    pub const fn made_at(&self) -> WorldTime {
        self.made_at
    }

    /// The first instant at which it is dead.
    pub const fn valid_until(&self) -> WorldTime {
        self.valid_until
    }

    /// Open or declined.
    pub const fn status(&self) -> ProposalStatus {
        self.status
    }

    /// Whether it still counts at `now`: `now < valid_until` (§5.10).
    pub fn is_live(&self, now: WorldTime) -> bool {
        now < self.valid_until
    }

    /// Whether it waits for an answer at `now`.
    pub fn is_open(&self, now: WorldTime) -> bool {
        self.is_live(now) && self.status == ProposalStatus::Open
    }
}

/// `proposals`: the proposals a seller — a Person, or the Organization that runs a shop — has
/// received, sorted by id (`ARC-118`). Written only by this pack's reductions of `price-proposed`,
/// `proposal-declined`, `proposal-withdrawn` and `proposal-accepted`. Disclosed only as the
/// [`ProposalsView`] about an observer itself.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposals {
    entries: Vec<Proposal>,
}

owned_component! {
    component = Proposals,
    owner = EconomySystem,
    component_type = "proposals",
    schema_version = 1,
}

impl Proposals {
    /// Every entry, live or dead, in id order.
    pub fn entries(&self) -> &[Proposal] {
        &self.entries
    }

    /// The live entries at `now`, in id order.
    pub fn live(&self, now: WorldTime) -> impl Iterator<Item = &Proposal> {
        self.entries.iter().filter(move |entry| entry.is_live(now))
    }

    /// The entry `id`, if it is open at `now`.
    pub fn open(&self, id: ProposalId, now: WorldTime) -> Option<&Proposal> {
        self.entries
            .binary_search_by_key(&id, Proposal::id)
            .ok()
            .map(|at| &self.entries[at])
            .filter(|entry| entry.is_open(now))
    }

    /// Why a new proposal from `buyer` for `item` would break a limit at `now`: a live entry for the
    /// same buyer and kind (`already-proposed`), or eight live entries (`seller-full`).
    pub(crate) fn limit(&self, buyer: EntityId, item: ItemId, now: WorldTime) -> Option<Limit> {
        if self
            .live(now)
            .any(|entry| entry.buyer.entity_id() == buyer && entry.item == item)
        {
            return Some(Limit::AlreadyProposed);
        }
        self.full(now).then_some(Limit::SellerFull)
    }

    /// Whether the seller holds the most live entries it may.
    pub(crate) fn full(&self, now: WorldTime) -> bool {
        self.live(now).count() >= PROPOSALS_PER_SELLER
    }

    /// These proposals without their dead entries (SD-S31h-9).
    fn pruned(&self, now: WorldTime) -> Self {
        Self {
            entries: self.live(now).copied().collect(),
        }
    }
}

/// Which limit a new proposal breaks (F-S31h-9): one code, `proposal-limit`, with this detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Limit {
    /// The buyer already has a live proposal for this kind to this seller.
    AlreadyProposed,
    /// The seller already holds eight live proposals.
    SellerFull,
}

impl Limit {
    /// The refusal, `proposal-limit` with its detail.
    pub(crate) fn refusal(self) -> Rejection {
        let detail = match self {
            Self::AlreadyProposed => "already-proposed",
            Self::SellerFull => "seller-full",
        };
        Rejection::System {
            code: crate::code::PROPOSAL_LIMIT,
            detail: Some(detail.to_owned()),
        }
    }
}

/// A seller's proposals; none when it has received none.
pub(crate) fn proposals_of(world: &WorldRead<'_>, seller: EntityId) -> Proposals {
    world
        .component::<Proposals>(seller)
        .cloned()
        .unwrap_or_default()
}

/// The seller holding the open proposal `id` at the read's instant, and the proposal.
pub(crate) fn find_open(world: &WorldRead<'_>, id: ProposalId) -> Option<(EntityId, Proposal)> {
    let now = world.at();
    world
        .components::<Proposals>()
        .find_map(|(seller, held)| held.open(id, now).map(|entry| (seller, *entry)))
}

// ---------------------------------------------------------------------------------------------
// reductions: the only writes of `Proposals`
// ---------------------------------------------------------------------------------------------

/// Whether `kind` is one of the four proposal facts [`reduce`] takes.
pub(crate) fn reduces(kind: &mineworld_contracts::EventTypeId) -> bool {
    [
        PriceProposed::EVENT_TYPE,
        ProposalDeclined::EVENT_TYPE,
        ProposalWithdrawn::EVENT_TYPE,
        ProposalAccepted::EVENT_TYPE,
    ]
    .contains(kind)
}

/// A proposal fact → the seller's [`Proposals`], after dropping its dead entries (§5.7). The owner
/// still decides (`ARC-26`): a `price-proposed` that breaks a limit, repeats an id, or carries a price
/// out of range or an expiry not after now, and an answer or withdrawal naming an entry that is not
/// open — or naming it with other terms — is refused, and nothing is written.
pub(crate) fn reduce(
    world: &mut WorldView<'_, EconomySystem>,
    event: &mineworld_contracts::EventEnvelope,
) -> Result<(), KernelError> {
    let kind = event.event_type();
    let now = world.at();
    let read = world.read();
    if *kind == PriceProposed::EVENT_TYPE {
        let proposed: PriceProposed = codec::event_payload(event.payload())?;
        let refusal = || refused(kind.clone(), Rejection::PreconditionFailed);
        let held = proposals_of(&read, proposed.seller()).pruned(now);
        let id = proposed.proposal();
        if held.entries.binary_search_by_key(&id, Proposal::id).is_ok()
            || held
                .limit(proposed.buyer().entity_id(), proposed.item(), now)
                .is_some()
            || !price_in_range(proposed.price())
            || proposed.valid_until() <= now
            || !is_holder_of_proposals(&read, proposed.seller())
        {
            return Err(refusal());
        }
        let mut entries = held.entries;
        let at = entries
            .binary_search_by_key(&id, Proposal::id)
            .expect_err("checked absent");
        entries.insert(
            at,
            Proposal {
                id,
                buyer: proposed.buyer(),
                item: proposed.item(),
                price: proposed.price(),
                place: proposed.place(),
                made_at: now,
                valid_until: proposed.valid_until(),
                status: ProposalStatus::Open,
            },
        );
        world.insert(proposed.seller(), Proposals { entries })?;
        return Ok(());
    }
    let (terms, decline) = if *kind == ProposalDeclined::EVENT_TYPE {
        let fact: ProposalDeclined = codec::event_payload(event.payload())?;
        (*fact.terms(), true)
    } else if *kind == ProposalWithdrawn::EVENT_TYPE {
        let fact: ProposalWithdrawn = codec::event_payload(event.payload())?;
        (*fact.terms(), false)
    } else {
        let fact: ProposalAccepted = codec::event_payload(event.payload())?;
        (*fact.terms(), false)
    };
    let mut held = proposals_of(&read, terms.seller()).pruned(now);
    let at = held
        .entries
        .binary_search_by_key(&terms.proposal(), Proposal::id)
        .ok()
        .filter(|at| {
            let entry = &held.entries[*at];
            entry.is_open(now)
                && entry.buyer == terms.buyer()
                && entry.item == terms.item()
                && entry.price == terms.price()
        })
        .ok_or_else(|| refused(kind.clone(), Rejection::PreconditionFailed))?;
    if decline {
        held.entries[at].status = ProposalStatus::Declined;
    } else {
        held.entries.remove(at);
    }
    if held.entries.is_empty() {
        world.remove::<Proposals>(terms.seller())?;
    } else {
        world.insert(terms.seller(), held)?;
    }
    Ok(())
}

/// Whether `seller` may hold proposals: a living Person or Organization.
fn is_holder_of_proposals(world: &WorldRead<'_>, seller: EntityId) -> bool {
    world.entity(seller).is_some_and(|record| {
        matches!(
            record.entity_type(),
            EntityType::Person | EntityType::Organization
        ) && record.lifecycle() != LifecycleState::Destroyed
    })
}

// ---------------------------------------------------------------------------------------------
// disclosure: the `proposals` record, about the observer itself (§5.9)
// ---------------------------------------------------------------------------------------------

/// One line of the `proposals` record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalLine {
    proposal: ProposalId,
    seller: EntityId,
    buyer: PersonId,
    item: ItemId,
    price: u64,
    ask: Option<u64>,
    valid_until: WorldTime,
    status: ProposalStatus,
    for_shop: bool,
}

impl ProposalLine {
    /// The proposal.
    pub const fn proposal(&self) -> ProposalId {
        self.proposal
    }

    /// Who would sell: a Person, or the Organization running the shop.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// Who proposes.
    pub const fn buyer(&self) -> PersonId {
        self.buyer
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// The proposed price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// The seller's asking price for the kind now (a shop's after its adjustments), if it still
    /// asks one.
    pub const fn ask(&self) -> Option<u64> {
        self.ask
    }

    /// The first instant at which the proposal is dead.
    pub const fn valid_until(&self) -> WorldTime {
        self.valid_until
    }

    /// Open or declined.
    pub const fn status(&self) -> ProposalStatus {
        self.status
    }

    /// Whether it is a shop's proposal.
    pub const fn for_shop(&self) -> bool {
        self.for_shop
    }
}

/// The `proposals` record as an observer is told it: the live proposals it may answer (`incoming`)
/// and the live proposals it made (`outgoing`), each in id order. A view, built when disclosed, never
/// stored; disclosed about the observer itself and absent when both lists are empty.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalsView {
    incoming: Vec<ProposalLine>,
    outgoing: Vec<ProposalLine>,
}

impl ProposalsView {
    /// The live proposals the observer may answer, in id order.
    pub fn incoming(&self) -> &[ProposalLine] {
        &self.incoming
    }

    /// The live proposals the observer made, in id order.
    pub fn outgoing(&self) -> &[ProposalLine] {
        &self.outgoing
    }
}

/// The `proposals` record for `observer` at the read's instant, if it has any line.
pub(crate) fn view(world: &WorldRead<'_>, observer: EntityId) -> Option<ProposalsView> {
    if !living_person(world, observer) {
        return None;
    }
    let now = world.at();
    let line = |seller: EntityId, entry: &Proposal| ProposalLine {
        proposal: entry.id,
        seller,
        buyer: entry.buyer,
        item: entry.item,
        price: entry.price,
        ask: ask_now(world, seller, entry),
        valid_until: entry.valid_until,
        status: entry.status,
        for_shop: entry.place.is_some(),
    };
    let mut incoming: Vec<ProposalLine> = world
        .component::<Proposals>(observer)
        .into_iter()
        .flat_map(|held| held.live(now))
        .filter(|entry| entry.place.is_none())
        .map(|entry| line(observer, entry))
        .collect();
    if let Some((place, shop)) = shop_here(world, observer)
        && shop.kept_by(observer)
    {
        let operator = shop.operator().entity_id();
        incoming.extend(
            world
                .component::<Proposals>(operator)
                .into_iter()
                .flat_map(|held| held.live(now))
                .filter(|entry| entry.place == Some(place))
                .map(|entry| line(operator, entry)),
        );
    }
    incoming.sort_by_key(ProposalLine::proposal);
    let mut outgoing: Vec<ProposalLine> = world
        .components::<Proposals>()
        .flat_map(|(seller, held)| {
            held.live(now)
                .filter(|entry| entry.buyer.entity_id() == observer)
                .map(move |entry| (seller, entry))
        })
        .map(|(seller, entry)| line(seller, entry))
        .collect();
    outgoing.sort_by_key(ProposalLine::proposal);
    (!incoming.is_empty() || !outgoing.is_empty()).then_some(ProposalsView { incoming, outgoing })
}

/// What the seller asks for the proposal's kind now: the shop's adjusted price for a shop's
/// proposal, the person's own ask otherwise.
fn ask_now(world: &WorldRead<'_>, seller: EntityId, entry: &Proposal) -> Option<u64> {
    match entry.place {
        Some(place) => world
            .component::<crate::component::Shop>(place.entity_id())
            .and_then(|shop| asking_price(world, shop, entry.item)),
        None => asks_of(world, seller).price(entry.item),
    }
}
