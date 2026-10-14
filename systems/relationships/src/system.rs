//! The installable system: what it declares, and the one thing it does — reduce.

use mineworld_contracts::{
    ComponentRecord, EntityId, EntityType, EntityTypeSet, Event, EventEnvelope, EventTypeId,
    PersonId, Rejection, RejectionCode, RelationTypeDeclaration, RelationTypeId, SystemId,
};
use mineworld_conversation::Spoke;
use mineworld_group_activity::{GroupActivityEnded, InvitationAccepted, InvitationDeclined};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::PerceptionProvider;
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions as section;
use serde_json::Value;

use crate::codec;
use crate::component::{Acquaintances, RelationshipValues};
use crate::decay::CurveName;
use crate::event::{AcquaintanceForgotten, BecameAcquainted, RelationshipChanged};
use crate::interactions::{
    RelationshipParameters, audience, decay_rule, increments, known_at_most, may_acquaint,
};

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
// Since S17's PR IL-e the five values above are the compiled defaults of the section's parameters
// (`spoke_familiarity` … `activity_regard`, `crate::interactions`), which a world may change.

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): all three (QFGA-5:
/// forgetting somebody is part of a life, as meeting them is).
pub const BIOGRAPHICAL: &[EventTypeId] = &[
    BecameAcquainted::EVENT_TYPE,
    RelationshipChanged::EVENT_TYPE,
    AcquaintanceForgotten::EVENT_TYPE,
];

/// Who knows whom.
///
/// A unit struct with no state (`INV-7`); `Clone`, `Debug` and `PartialEq` cost nothing and are what
/// its section's types require.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RelationshipsSystem;

impl SystemIdentity for RelationshipsSystem {
    const ID: SystemId = SystemId::from_static("relationships");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts, and its section of the World's Interaction List (`ARC-63`, since S17's PR
/// IL-e), which is its configuration. It owns no authored section.
impl SystemPack for RelationshipsSystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::interactions!();
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
    /// 2 since S17's PR IL-e: the declaration gained the section's component and configured fact.
    /// 3 since S24's PR FG-a: the section's parameters gained decay (the configured fact's payload
    /// grows) and the declaration the fact `acquaintance-forgotten` (D-FGA-9).
    const VERSION: SystemVersion = SystemVersion::new(3);

    /// No `depending_on`, no `providing`: it hears facts and writes only its own state.
    fn declaration(&self) -> SystemDeclaration {
        section::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .owning::<Acquaintances>()
                .emitting::<BecameAcquainted>()
                .emitting::<RelationshipChanged>()
                .emitting::<AcquaintanceForgotten>()
                .subscribing_to::<Spoke>()
                .subscribing_to::<InvitationAccepted>()
                .subscribing_to::<InvitationDeclined>()
                .subscribing_to::<GroupActivityEnded>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        section::install::<Self>(tables)?;
        tables.component::<Acquaintances>()?;
        tables.relation(knows_declaration())
    }

    /// Reduces a social fact another pack stated into the relationship state this pack owns, and
    /// states what a reader can name: a new acquaintance, a level crossed.
    ///
    /// The world's Interaction List decides, per direction, whether the pair may form a relationship
    /// at all (`acquaint`) and how much the contact moves it (the five increments), both at the place
    /// the causing fact happened.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if section::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let changes = changes(&world.read(), event)?;
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

/// The directed changes a fact implies (SD-6), decoded with the owner's published type. Each
/// direction's increments are the world's for that holder, counterpart and the fact's place — the
/// compiled constants above when it configures none.
fn changes(world: &WorldRead<'_>, event: &EventEnvelope) -> Result<Vec<Change>, KernelError> {
    let kind = event.event_type();
    let at = event.place();
    let by = |a: PersonId, b: PersonId| -> RelationshipParameters { increments(world, at, a, b) };
    let both = |a: PersonId, b: PersonId, make: &dyn Fn(PersonId, PersonId) -> Change| {
        vec![make(a, b), make(b, a)]
    };
    if *kind == Spoke::EVENT_TYPE {
        let spoke: Spoke = codec::event_payload(event.payload())?;
        return Ok(both(spoke.speaker(), spoke.listener(), &|a, b| Change {
            familiarity: by(a, b).spoke_familiarity,
            exchange: true,
            ..Change::new(a, b)
        }));
    }
    if *kind == InvitationAccepted::EVENT_TYPE {
        let accepted: InvitationAccepted = codec::event_payload(event.payload())?;
        return Ok(both(accepted.inviter(), accepted.invitee(), &|a, b| {
            Change {
                regard: by(a, b).accepted_regard,
                ..Change::new(a, b)
            }
        }));
    }
    if *kind == InvitationDeclined::EVENT_TYPE {
        let declined: InvitationDeclined = codec::event_payload(event.payload())?;
        let (inviter, invitee) = (declined.inviter(), declined.invitee());
        return Ok(vec![Change {
            regard: by(inviter, invitee).declined_regard,
            ..Change::new(inviter, invitee)
        }]);
    }
    if *kind == GroupActivityEnded::EVENT_TYPE {
        let ended: GroupActivityEnded = codec::event_payload(event.payload())?;
        let members = ended.members();
        let mut pairs = Vec::new();
        for person in members {
            for counterpart in members {
                if person != counterpart {
                    let increments = by(*person, *counterpart);
                    pairs.push(Change {
                        familiarity: increments.activity_familiarity,
                        regard: increments.activity_regard,
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

/// Applies one change: the edge and the entry together, then the facts a reader can name. A
/// direction the world's list forbids (`acquaint`) is skipped before anything is read or written:
/// no entry, no edge, no fact.
fn apply(
    world: &mut WorldView<'_, RelationshipsSystem>,
    event: &EventEnvelope,
    change: &Change,
) -> Result<Vec<Emission>, KernelError> {
    if change.person == change.counterpart
        || !may_acquaint(
            &world.read(),
            event.place(),
            change.person,
            change.counterpart,
        )
    {
        return Ok(Vec::new());
    }
    let holder = change.person.entity_id();
    let rule = decay_rule(&world.read(), change.person, change.counterpart)
        .map_err(|curve| unknown_curve(event, &curve))?;
    let mut known = world
        .read()
        .component::<Acquaintances>(holder)
        .cloned()
        .unwrap_or_default();
    let (values, new) = known.entry(
        change.counterpart,
        RelationshipValues::met(event.at(), event.id()),
    );
    // The level last stated for this pair is the stored values' (every crossing at a reduction is
    // stated); the contact adds to the values as they read now, faded (`ARC-86`, §5.2, §5.4).
    let before = values.level();
    values.rebase(&rule, event.at());
    values.adjust(change.familiarity, change.regard, event.at());
    if change.exchange {
        values.exchanged();
    }
    if change.activity {
        values.shared_an_activity();
    }
    let after = values.level();
    let forgotten = if new {
        beyond_the_cap(&world.read(), event, change, &mut known)?
    } else {
        Vec::new()
    };
    world.relate(&knows(), holder, change.counterpart.entity_id())?;
    for victim in &forgotten {
        unknow(world, event, holder, *victim)?;
    }
    world.insert(holder, known)?;

    let mut emissions = Vec::new();
    if new {
        emissions.push(fact(
            &world.read(),
            event,
            change,
            &BecameAcquainted::new(change.person, change.counterpart),
        ));
    }
    if before != after {
        emissions.push(fact(
            &world.read(),
            event,
            change,
            &RelationshipChanged::new(change.person, change.counterpart, before, after),
        ));
    }
    for victim in forgotten {
        emissions.push(fact(
            &world.read(),
            event,
            &Change::new(change.person, victim),
            &AcquaintanceForgotten::new(change.person, victim),
        ));
    }
    Ok(emissions)
}

/// The cap (§5.5): with a new acquaintance just formed, evicts from `known` the weakest ties beyond
/// the number the holder keeps, never the one just met; whom, in order.
fn beyond_the_cap(
    world: &WorldRead<'_>,
    event: &EventEnvelope,
    change: &Change,
    known: &mut Acquaintances,
) -> Result<Vec<PersonId>, KernelError> {
    let at_most = known_at_most(world, change.person);
    if known.len() <= at_most {
        return Ok(Vec::new());
    }
    known
        .evict_beyond(
            at_most,
            |counterpart| decay_rule(world, change.person, counterpart),
            event.at(),
            change.counterpart,
        )
        .map_err(|curve| unknown_curve(event, &curve))
}

/// Removes `holder → victim`'s `knows` edge, which every entry has (the edge and the entry are written
/// together); one that is not there is a broken invariant, named.
fn unknow(
    world: &mut WorldView<'_, RelationshipsSystem>,
    event: &EventEnvelope,
    holder: EntityId,
    victim: PersonId,
) -> Result<(), KernelError> {
    let edge = world
        .read()
        .relations_touching(holder)
        .find(|edge| {
            *edge.relation_type() == knows()
                && edge.from() == holder
                && edge.to() == victim.entity_id()
        })
        .cloned();
    match edge {
        Some(edge) if world.unrelate(&edge)? => Ok(()),
        _ => Err(KernelError::FactRefusedByOwner {
            system: RelationshipsSystem::ID,
            event_type: event.event_type().clone(),
            reason: Rejection::System {
                code: EDGELESS,
                detail: Some(format!(
                    "the acquaintance {holder} → {} had no knows edge to remove",
                    victim.entity_id()
                )),
            },
        }),
    }
}

/// The code of [`unknow`]'s refusal.
const EDGELESS: RejectionCode = RejectionCode::from_static("acquaintance-without-edge");

/// The fault a reduction raises for a curve this build does not know, naming it (§5.3): journaled as a
/// fault, never reduced with undecayed values.
fn unknown_curve(event: &EventEnvelope, curve: &CurveName) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: RelationshipsSystem::ID,
        event_type: event.event_type().clone(),
        reason: Rejection::System {
            code: UNKNOWN_CURVE,
            detail: Some(format!(
                "the decay curve '{curve}' is not bundled with relationships nor registered in \
                 this build"
            )),
        },
    }
}

/// The code of [`unknown_curve`]'s refusal.
const UNKNOWN_CURVE: RejectionCode = RejectionCode::from_static("unknown-decay-curve");

/// What `holder` holds about `counterpart`, read now — familiarity and regard faded by the world's
/// decay for the time since their last contact (`ARC-86`) — or [`None`] when `holder` does not know
/// them, or the world names a curve this build does not know (fail closed, as disclosure does).
///
/// The reading a disclosure shows, for Rust readers: tests, and an in-process decision model.
pub fn current(
    world: &WorldRead<'_>,
    holder: PersonId,
    counterpart: PersonId,
) -> Option<RelationshipValues> {
    let stored = world
        .component::<Acquaintances>(holder.entity_id())?
        .of(counterpart)?;
    let rule = decay_rule(world, holder, counterpart).ok()?;
    Some(stored.reading(&rule, world.at()))
}

/// One of this pack's facts about a directed pair: about the person, with both as participants, heard
/// by the two of them (as the world's list routes it), where the fact that caused it happened.
fn fact<E: Event>(
    world: &WorldRead<'_>,
    cause: &EventEnvelope,
    change: &Change,
    payload: &E,
) -> Emission {
    let heard_by = audience(
        world,
        cause.place(),
        &E::EVENT_TYPE,
        change.person,
        change.counterpart,
    );
    let emission = Emission::new::<E>(codec::encode(payload), heard_by)
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
    ///
    /// The same record as stored, with each familiarity and regard read at the world's instant —
    /// faded by the world's decay (`ARC-86`, D-FGA-2). A world that configures no decay discloses the
    /// stored values. A curve this build does not know withholds the record rather than show
    /// undecayed values (§5.3).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if subject != observer {
            return Vec::new();
        }
        let (Some(known), Ok(holder)) = (
            world.component::<Acquaintances>(observer),
            PersonId::new(observer, EntityType::Person),
        ) else {
            return Vec::new();
        };
        known
            .reading(
                |counterpart| decay_rule(world, holder, counterpart),
                world.at(),
            )
            .map(|now| {
                vec![ComponentRecord::new::<Acquaintances>(
                    observer,
                    codec::to_value(&now),
                )]
            })
            .unwrap_or_default()
    }
}
