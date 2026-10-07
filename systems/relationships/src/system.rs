//! The installable system: what it declares, and the one thing it does — reduce.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, EntityTypeSet, Event, EventEnvelope, EventTypeId,
    PersonId, RelationTypeDeclaration, RelationTypeId, SystemId, Visibility,
};
use mineworld_conversation::Spoke;
use mineworld_group_activity::{GroupActivityEnded, InvitationAccepted, InvitationDeclined};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::codec;
use crate::component::{Acquaintances, RelationshipValues};
use crate::event::{BecameAcquainted, RelationshipChanged};

/// What one exchange of words adds to familiarity, in both directions (SD-6 (1)).
pub const SPOKE_FAMILIARITY: i32 = 10;
/// What an accepted invitation adds to regard, in both directions (SD-6 (2)).
pub const ACCEPTED_REGARD: i32 = 50;
/// What a declined invitation does to the inviter's regard for the person who declined (SD-6 (3)).
pub const DECLINED_REGARD: i32 = -30;
/// What an activity done together adds to familiarity, for every ordered pair of members (SD-6 (4)).
pub const ACTIVITY_FAMILIARITY: i32 = 50;
/// What an activity done together adds to regard, for every ordered pair of members (SD-6 (4)).
pub const ACTIVITY_REGARD: i32 = 20;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): both of them.
pub const BIOGRAPHICAL: &[EventTypeId] = &[
    BecameAcquainted::EVENT_TYPE,
    RelationshipChanged::EVENT_TYPE,
];

/// Who knows whom.
#[derive(Default)]
pub struct RelationshipsSystem;

impl SystemIdentity for RelationshipsSystem {
    const ID: SystemId = SystemId::from_static("relationships");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts. It owns no authored section.
impl SystemPack for RelationshipsSystem {
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
}

/// The edge this pack declares.
pub fn knows() -> RelationTypeId {
    RelationTypeId::new("knows").expect("a legal relation type name")
}

/// `knows`: directed, from a person to a person, never to oneself.
pub fn knows_declaration() -> RelationTypeDeclaration {
    let people = || EntityTypeSet::new([EntityType::Person]).expect("a non-empty set");
    RelationTypeDeclaration::directed(knows(), RelationshipsSystem::ID, people(), people())
}

impl System for RelationshipsSystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// No `depending_on`, no `providing`: it hears facts and writes only its own state.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<Acquaintances>()
            .emitting::<BecameAcquainted>()
            .emitting::<RelationshipChanged>()
            .subscribing_to::<Spoke>()
            .subscribing_to::<InvitationAccepted>()
            .subscribing_to::<InvitationDeclined>()
            .subscribing_to::<GroupActivityEnded>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Acquaintances>()?;
        tables.relation(knows_declaration())
    }

    /// Reduces a social fact another pack stated into the relationship state this pack owns, and
    /// states what a reader can name: a new acquaintance, a level crossed.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let changes = changes(event)?;
        let mut emissions = Vec::new();
        for change in changes {
            emissions.extend(apply(world, event, &change)?);
        }
        Ok(emissions)
    }
}

/// One directed change: what `person` comes to hold about `counterpart`.
struct Change {
    person: PersonId,
    counterpart: PersonId,
    familiarity: i32,
    regard: i32,
    exchange: bool,
    activity: bool,
}

impl Change {
    const fn new(person: PersonId, counterpart: PersonId) -> Self {
        Self {
            person,
            counterpart,
            familiarity: 0,
            regard: 0,
            exchange: false,
            activity: false,
        }
    }
}

/// The directed changes a fact implies (SD-6), decoded with the owner's published type.
fn changes(event: &EventEnvelope) -> Result<Vec<Change>, KernelError> {
    let kind = event.event_type();
    let both = |a: PersonId, b: PersonId, make: &dyn Fn(PersonId, PersonId) -> Change| {
        vec![make(a, b), make(b, a)]
    };
    if *kind == Spoke::EVENT_TYPE {
        let spoke: Spoke = codec::event_payload(event.payload())?;
        return Ok(both(spoke.speaker(), spoke.listener(), &|a, b| Change {
            familiarity: SPOKE_FAMILIARITY,
            exchange: true,
            ..Change::new(a, b)
        }));
    }
    if *kind == InvitationAccepted::EVENT_TYPE {
        let accepted: InvitationAccepted = codec::event_payload(event.payload())?;
        return Ok(both(accepted.inviter(), accepted.invitee(), &|a, b| {
            Change {
                regard: ACCEPTED_REGARD,
                ..Change::new(a, b)
            }
        }));
    }
    if *kind == InvitationDeclined::EVENT_TYPE {
        let declined: InvitationDeclined = codec::event_payload(event.payload())?;
        return Ok(vec![Change {
            regard: DECLINED_REGARD,
            ..Change::new(declined.inviter(), declined.invitee())
        }]);
    }
    if *kind == GroupActivityEnded::EVENT_TYPE {
        let ended: GroupActivityEnded = codec::event_payload(event.payload())?;
        let members = ended.members();
        let mut pairs = Vec::new();
        for person in members {
            for counterpart in members {
                if person != counterpart {
                    pairs.push(Change {
                        familiarity: ACTIVITY_FAMILIARITY,
                        regard: ACTIVITY_REGARD,
                        activity: true,
                        ..Change::new(*person, *counterpart)
                    });
                }
            }
        }
        return Ok(pairs);
    }
    Ok(Vec::new())
}

/// Applies one change: the edge and the entry together, then the facts a reader can name.
fn apply(
    world: &mut WorldView<'_, RelationshipsSystem>,
    event: &EventEnvelope,
    change: &Change,
) -> Result<Vec<Emission>, KernelError> {
    if change.person == change.counterpart {
        return Ok(Vec::new());
    }
    let holder = change.person.entity_id();
    let mut known = world
        .read()
        .component::<Acquaintances>(holder)
        .cloned()
        .unwrap_or_default();
    let (values, new) = known.entry(
        change.counterpart,
        RelationshipValues::met(event.at(), event.id()),
    );
    let before = values.level();
    values.adjust(change.familiarity, change.regard, event.at());
    if change.exchange {
        values.exchanged();
    }
    if change.activity {
        values.shared_an_activity();
    }
    let after = values.level();
    world.relate(&knows(), holder, change.counterpart.entity_id())?;
    world.insert(holder, known)?;

    let mut emissions = Vec::new();
    if new {
        emissions.push(fact(
            event,
            change,
            &BecameAcquainted::new(change.person, change.counterpart),
        ));
    }
    if before != after {
        emissions.push(fact(
            event,
            change,
            &RelationshipChanged::new(change.person, change.counterpart, before, after),
        ));
    }
    Ok(emissions)
}

/// One of this pack's facts about a directed pair: about the person, with both as participants, heard
/// by the two of them, where the fact that caused it happened.
fn fact<E: Event>(cause: &EventEnvelope, change: &Change, payload: &E) -> Emission {
    let emission = Emission::new::<E>(codec::encode(payload), Visibility::Participants)
        .about(vec![change.person.entity_id()])
        .with_participants(vec![
            change.person.entity_id(),
            change.counterpart.entity_id(),
        ]);
    match cause.place() {
        Some(place) => emission.at_place(place),
        None => emission,
    }
}

impl PerceptionProvider for RelationshipsSystem {
    /// A person's `Acquaintances`, **to that person and nobody else** (SD-9, `INV-13`): how Alice
    /// regards Bob is Alice's to know.
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
            .component::<Acquaintances>(observer)
            .map(|known| {
                vec![ComponentRecord::new::<Acquaintances>(
                    observer,
                    codec::to_value(known),
                )]
            })
            .unwrap_or_default()
    }
}
