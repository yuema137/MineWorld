//! The installable system: what it declares, and the four things a world asks of it.

use mineworld_contracts::{
    ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope, LifecycleState,
    Location, PersonId, PlaceId, Rejection, RejectionCode, SystemId,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use mineworld_presence::{
    Arrived, Offer, PerceptionProvider, Presence, PresenceSystem, admit, arrival,
};

use crate::action::{Move, move_offer_requirement, stride_requirement};
use crate::codec;
use crate::component::{Passage, Passages};
use crate::event::PassageOpened;

/// Whether a person may walk where they ask — decided here, on the server, against where the world
/// says they are.
///
/// It decides and does not record positions: a person's position is presence's state, and this
/// system states presence's own `arrived` fact, built by presence's `arrival`, for presence to
/// reduce (`DECISIONS.md` `ARC-26`). That is why it depends on presence and presence knows nothing
/// of it, and why disabling it takes away `move` and nothing else (`AC-2`).
#[derive(Default)]
pub struct MovementSystem;

impl SystemIdentity for MovementSystem {
    const ID: SystemId = SystemId::from_static("movement");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing.
/// It declares no biographical fact and owns no authored section; a place's `passages` are a field
/// of the World Pack format, bound to this pack by the loader (`ARC-31` item 5).
impl SystemPack for MovementSystem {}

/// This pack's own reason for refusing a request it cannot read — the same code, for the same
/// reason, as the other packs' (a malformed payload is a fact about the request, not the world).
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for MovementSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<Passages>()
            .providing::<Move>()
            .emitting::<Arrived>()
            .emitting::<PassageOpened>()
            .subscribing_to::<PassageOpened>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Passages>()
    }

    /// Decides whether this person may move where they ask, reading state and writing nothing.
    ///
    /// ```text
    /// 1  payload readable as Move                 else malformed-payload
    /// 2  actor exists, is a person                else PreconditionFailed / NoSupportedInteraction
    /// 3  presence admits the destination          else PreconditionFailed  (a living person, a
    ///                                                                       living place)
    /// 4  the actor has a Presence to move from    else PreconditionFailed  (placement is genesis)
    /// 5  the destination is reachable in one move else TooFarAway          (see `reachable`)
    /// ```
    ///
    /// The position moved *from* is presence's record. The `actor_location` a client may put on
    /// the request is ignored, as every pack ignores it: it is a report, and the server decides
    /// (`ENGINEERING_RULES.md` §8).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let requested: Move =
            codec::action_payload(intent.payload()).map_err(|error| Rejection::System {
                code: MALFORMED_PAYLOAD,
                detail: Some(error.to_string()),
            })?;

        let actor = world
            .entity(intent.actor())
            .ok_or(Rejection::PreconditionFailed)?;
        let person = PersonId::new(actor.id(), actor.entity_type())
            .map_err(|_| Rejection::NoSupportedInteraction)?;
        let to = requested.to();
        admit(world, person, to)?;

        let from = world
            .component::<Presence>(person.entity_id())
            .map(Presence::location)
            .ok_or(Rejection::PreconditionFailed)?;
        reachable(world, &from, &to)
    }

    /// States the arrival, in presence's vocabulary and through presence's constructor, and writes
    /// nothing.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let requested: Move = codec::action_payload(intent.payload())?;
        let read = world.read();
        let actor = read.require_entity(intent.actor())?;
        let person = PersonId::new(actor.id(), actor.entity_type())?;
        let fact = arrival(&read, person, requested.to()).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: PresenceSystem::ID,
                event_type: Arrived::EVENT_TYPE,
                reason,
            }
        })?;
        Ok(vec![fact])
    }

    /// Reduces an opened passage into the [`Passages`] of both places.
    ///
    /// The owner decides here too: a passage from a place to itself, or to something that is not a
    /// place of this world, is refused with [`KernelError::FactRefusedByOwner`] and nothing is
    /// written.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != PassageOpened::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let opened: PassageOpened = codec::event_payload(event.payload())?;
        let (a, a_at) = opened.a();
        let (b, b_at) = opened.b();
        {
            let read = world.read();
            if a == b || !is_place(&read, a) || !is_place(&read, b) {
                return Err(KernelError::FactRefusedByOwner {
                    system: Self::ID,
                    event_type: PassageOpened::EVENT_TYPE,
                    reason: Rejection::PreconditionFailed,
                });
            }
        }

        for (from, passage) in [
            (a, Passage::new(b, a_at, b_at)),
            (b, Passage::new(a, b_at, a_at)),
        ] {
            let mut passages = world
                .read()
                .component::<Passages>(from.entity_id())
                .cloned()
                .unwrap_or_default();
            passages.open(passage);
            world.insert(from.entity_id(), passages)?;
        }
        Ok(Vec::new())
    }
}

impl PerceptionProvider for MovementSystem {
    /// Offers `move` to any living person, once, against nobody.
    ///
    /// With [`move_offer_requirement`] — nothing required of a target — because an offer's
    /// requirement is evaluated against its target and `move` has none. Whether one particular
    /// destination is reachable is answered when it is asked, by [`MovementSystem::validate`].
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let is_living_person = world.entity(observer).is_some_and(|entity| {
            entity.entity_type() == EntityType::Person
                && entity.lifecycle() != LifecycleState::Destroyed
        });
        if !is_living_person {
            return Vec::new();
        }
        vec![Offer::new::<Move>(move_offer_requirement())]
    }

    /// Discloses a place's [`Passages`] — where its doorways are — to whoever perceives the place.
    ///
    /// Perception asks only about entities the observation already lists, and it lists only the
    /// observer's own place, so this tells a person the ways out of the room they stand in and of no
    /// other. It states *where* a doorway is, never whether one may pass: that stays
    /// [`MovementSystem::validate`]'s judgement, made when a `move` is asked (`ENGINEERING_RULES.md`
    /// §8). A controller that cannot see the door cannot walk out of the room, which is why this
    /// exists (step-08 §10.1 Q4). A person, and a place with no passages, disclose nothing — absence
    /// rather than an empty record.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        _observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let is_place = world
            .entity(subject)
            .is_some_and(|entity| entity.entity_type() == EntityType::Place);
        if !is_place {
            return Vec::new();
        }
        world
            .component::<Passages>(subject)
            .map(|passages| {
                vec![ComponentRecord::new::<Passages>(
                    subject,
                    codec::to_value(passages),
                )]
            })
            .unwrap_or_default()
    }
}

/// Whether `to` is one move from `from`. Every distance is decided by the contract layer's one
/// evaluator with the destination or a doorway as the target — there is no distance arithmetic in
/// this crate.
///
/// ```text
/// same place      a stride: `to` within MAX_STRIDE of `from`
/// another place   through the passage from `from`'s place to `to`'s:
///                   none                    TooFarAway — not adjoining; that is travel
///                   doorway here known      `from` within MAX_STRIDE of it, else TooFarAway
///                   doorway there known     `to` within MAX_STRIDE of it, else TooFarAway
/// ```
///
/// Where a position is not modelled — on either side of a request or a doorway — the evaluator's
/// own degeneracy applies (`CORE_CONCEPTS.md` §6.3): the range becomes *same place*, so a semantic
/// world moves freely within a place and through its passages.
fn reachable(world: &WorldRead<'_>, from: &Location, to: &Location) -> Result<(), Rejection> {
    let stride = stride_requirement();
    if from.place() == to.place() {
        return stride.evaluate(from, Some(to), true);
    }
    let passage = world
        .component::<Passages>(from.place().entity_id())
        .and_then(|passages| passages.to(to.place()))
        .ok_or(Rejection::TooFarAway)?;
    if let Some(here) = passage.here() {
        let doorway = Location::in_place(from.place()).with_local(here);
        stride.evaluate(from, Some(&doorway), true)?;
    }
    if let Some(there) = passage.there() {
        let doorway = Location::in_place(to.place()).with_local(there);
        stride.evaluate(to, Some(&doorway), true)?;
    }
    Ok(())
}

/// Whether this world holds a living place by that identity.
fn is_place(world: &WorldRead<'_>, place: PlaceId) -> bool {
    world.entity(place.entity_id()).is_some_and(|record| {
        record.entity_type() == EntityType::Place && record.lifecycle() != LifecycleState::Destroyed
    })
}
