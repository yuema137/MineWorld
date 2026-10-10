//! The installable system: what it declares, how it decides, and how it reduces.

use mineworld_contracts::{
    Action, ActionIntent, EntityId, EntityType, Event, EventEnvelope, EventTypeId, LifecycleState,
    Location, PersonId, PlaceId, ProcessId, Rejection, RejectionCode, SystemId, WorldTime,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, Process, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{PersonEnteredPlace, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions::{self, Role, Roles};

use crate::action::{
    AcceptInvitation, DeclineInvitation, Invite, JoinGroupActivity, LeaveGroupActivity,
    accept_requirement, decline_requirement, invite_requirement, join_requirement,
};
use crate::codec;
use crate::component::{Invitation, Invitations, Participation};
use crate::event::{
    GroupActivityEnded, GroupActivityStarted, InvitationAccepted, InvitationDeclined, Invited,
    JoinedGroupActivity, LeftGroupActivity,
};
use crate::kind::ActivityKind;
use crate::process::{ACTIVITY_LENGTH, ActivityState, GroupActivity};

/// Doing things together.
///
/// A unit struct: its state is the components and processes it owns, held in the world (`INV-7`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GroupActivitySystem;

impl SystemIdentity for GroupActivitySystem {
    const ID: SystemId = SystemId::from_static("group-activity");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): its
/// biographical facts, and its section of the World's Interaction List (`ARC-63`), which is its
/// configuration. It owns no authored section.
impl SystemPack for GroupActivitySystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::interactions!();
}

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): beginning,
/// joining, leaving and ending something done together. The invitation facts are not — being asked
/// is an exchange, like speech, and what came of it is in the activity facts.
pub const BIOGRAPHICAL: &[EventTypeId] = &[
    GroupActivityStarted::EVENT_TYPE,
    JoinedGroupActivity::EVENT_TYPE,
    LeftGroupActivity::EVENT_TYPE,
    GroupActivityEnded::EVENT_TYPE,
];

/// This pack's own reason for refusing a request it cannot read (conversation's, for the same
/// reason: a malformed frame is not a fact about the world).
const MALFORMED_PAYLOAD: RejectionCode = RejectionCode::from_static("malformed-payload");

impl System for GroupActivitySystem {
    /// 2 since S17's PR IL-b: the section's component and configured fact, and `Invitations`' schema 2.
    const VERSION: SystemVersion = SystemVersion::new(2);

    fn declaration(&self) -> SystemDeclaration {
        interactions::declare::<Self>(
            SystemDeclaration::of::<Self>()
                .depending_on([PresenceSystem::ID])
                .owning::<Invitations>()
                .owning::<Participation>()
                .providing::<Invite>()
                .providing::<AcceptInvitation>()
                .providing::<DeclineInvitation>()
                .providing::<JoinGroupActivity>()
                .providing::<LeaveGroupActivity>()
                .emitting::<Invited>()
                .emitting::<InvitationAccepted>()
                .emitting::<InvitationDeclined>()
                .emitting::<GroupActivityStarted>()
                .emitting::<JoinedGroupActivity>()
                .emitting::<LeftGroupActivity>()
                .emitting::<GroupActivityEnded>()
                .subscribing_to::<Invited>()
                .subscribing_to::<InvitationAccepted>()
                .subscribing_to::<InvitationDeclined>()
                .subscribing_to::<GroupActivityStarted>()
                .subscribing_to::<JoinedGroupActivity>()
                .subscribing_to::<LeftGroupActivity>()
                .subscribing_to::<GroupActivityEnded>()
                .subscribing_to::<PersonEnteredPlace>(),
        )
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        interactions::install::<Self>(tables)?;
        tables.component::<Invitations>()?;
        tables.component::<Participation>()
    }

    /// Decides whether a request is admissible, reading state and writing nothing.
    ///
    /// Everything spatial is one call to [`SpatialRequirement::evaluate`] against the requirement this
    /// pack publishes, with the availability this pack judges — the same two values an affordance is
    /// priced from, so a client is shown the answer dispatch gives. The client's `actor_location` is
    /// ignored: positions are presence's.
    ///
    /// [`SpatialRequirement::evaluate`]: mineworld_contracts::SpatialRequirement::evaluate
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let actor = intent.actor();
        let here = located(world, actor).ok_or(Rejection::PreconditionFailed)?;
        if person(world, actor).is_none() {
            return Err(Rejection::NoSupportedInteraction);
        }
        let action = intent.action_type();
        if *action == LeaveGroupActivity::ACTION_TYPE {
            readable::<LeaveGroupActivity>(intent)?;
            return participation(world, actor)
                .map(|_| ())
                .ok_or(Rejection::PreconditionFailed);
        }

        let target = intent.target().ok_or(Rejection::PreconditionFailed)?;
        if target == actor {
            return Err(Rejection::NoSupportedInteraction);
        }
        let target_record = world.entity(target).ok_or(Rejection::TargetUnavailable)?;
        if target_record.entity_type() != EntityType::Person {
            return Err(Rejection::NoSupportedInteraction);
        }
        let there = located(world, target);
        let active = target_record.lifecycle() == LifecycleState::Active;

        if *action == Invite::ACTION_TYPE {
            readable::<Invite>(intent)?;
            return invite_requirement().evaluate(
                &here,
                there.as_ref(),
                active && participation(world, target).is_none(),
            );
        }
        if *action == AcceptInvitation::ACTION_TYPE || *action == DeclineInvitation::ACTION_TYPE {
            let accepting = *action == AcceptInvitation::ACTION_TYPE;
            if accepting {
                readable::<AcceptInvitation>(intent)?;
            } else {
                readable::<DeclineInvitation>(intent)?;
            }
            open_invitation(world, actor, target, intent.issued_at())
                .ok_or(Rejection::PreconditionFailed)?;
            if !accepting {
                return decline_requirement().evaluate(&here, there.as_ref(), active);
            }
            if participation(world, actor).is_some() {
                return Err(Rejection::Busy);
            }
            return accept_requirement().evaluate(&here, there.as_ref(), active);
        }
        if *action == JoinGroupActivity::ACTION_TYPE {
            readable::<JoinGroupActivity>(intent)?;
            if participation(world, actor).is_some() {
                return Err(Rejection::Busy);
            }
            let joinable = active
                && participation(world, target)
                    .is_some_and(|theirs| running(world, theirs.activity()).is_some());
            return join_requirement().evaluate(&here, there.as_ref(), joinable);
        }
        // Routing sends this system only the actions it declared; anything else is not one of them.
        Err(Rejection::NoSupportedInteraction)
    }

    /// States what happened. Writes only the process — starting it, changing its member list, ending
    /// it; the components are written while reducing the facts returned here.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let unresolvable = || KernelError::ActionNotResolvedBySystem {
            system: Self::ID,
            action_type: intent.action_type().clone(),
        };
        let (actor, place, target) = {
            let read = world.read();
            let actor = person(&read, intent.actor()).ok_or_else(unresolvable)?;
            let place = located(&read, intent.actor())
                .ok_or_else(unresolvable)?
                .place();
            let target = intent.target().and_then(|target| person(&read, target));
            (actor, place, target)
        };
        let action = intent.action_type().clone();

        if action == LeaveGroupActivity::ACTION_TYPE {
            let activity = participation(&world.read(), actor.entity_id())
                .map(Participation::activity)
                .ok_or_else(unresolvable)?;
            return depart(world, activity, actor);
        }
        let target = target.ok_or_else(unresolvable)?;
        if action == Invite::ACTION_TYPE {
            let invite: Invite = codec::action_payload(intent.payload())?;
            return Ok(vec![
                Invited::new(actor, target, invite.kind().clone()).emission(place),
            ]);
        }
        if action == DeclineInvitation::ACTION_TYPE || action == AcceptInvitation::ACTION_TYPE {
            let kind = open_invitation(
                &world.read(),
                actor.entity_id(),
                target.entity_id(),
                intent.issued_at(),
            )
            .map(|invitation| invitation.kind().clone())
            .ok_or_else(unresolvable)?;
            if action == DeclineInvitation::ACTION_TYPE {
                return Ok(vec![
                    InvitationDeclined::new(target, actor, kind).emission(place),
                ]);
            }
            let mut emissions =
                vec![InvitationAccepted::new(target, actor, kind.clone()).emission(place)];
            let theirs =
                participation(&world.read(), target.entity_id()).map(Participation::activity);
            match theirs {
                Some(activity) => emissions.push(join(world, activity, actor)?),
                None => emissions.push(begin(world, kind, place, target, actor)?),
            }
            return Ok(emissions);
        }
        if action == JoinGroupActivity::ACTION_TYPE {
            let activity = participation(&world.read(), target.entity_id())
                .map(Participation::activity)
                .ok_or_else(unresolvable)?;
            return Ok(vec![join(world, activity, actor)?]);
        }
        Err(unresolvable())
    }

    /// Reduces this pack's own facts into the components it owns, and answers presence's
    /// `person-entered-place` for a member who walked out of their activity's place.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if interactions::reduce(world, event)? {
            return Ok(Vec::new());
        }
        let now = event.at();
        let kind = event.event_type();
        if *kind == Invited::EVENT_TYPE {
            let invited: Invited = codec::event_payload(event.payload())?;
            let invitee = invited.invitee().entity_id();
            let until = WorldTime::from_seconds(now.seconds().saturating_add(lifetime(
                &world.read(),
                &invited,
                event.place(),
            )));
            let mut held = invitations(world, invitee);
            held.receive(
                Invitation::new(invited.inviter(), invited.kind().clone(), now, until),
                now,
            );
            world.insert(invitee, held)?;
        } else if *kind == InvitationAccepted::EVENT_TYPE {
            let answer: InvitationAccepted = codec::event_payload(event.payload())?;
            answered(world, answer.invitee(), answer.inviter(), now)?;
        } else if *kind == InvitationDeclined::EVENT_TYPE {
            let answer: InvitationDeclined = codec::event_payload(event.payload())?;
            answered(world, answer.invitee(), answer.inviter(), now)?;
        } else if *kind == GroupActivityStarted::EVENT_TYPE {
            let started: GroupActivityStarted = codec::event_payload(event.payload())?;
            for member in started.members() {
                world.insert(
                    member.entity_id(),
                    Participation::new(started.activity(), started.kind().clone(), now),
                )?;
            }
        } else if *kind == JoinedGroupActivity::EVENT_TYPE {
            let joined: JoinedGroupActivity = codec::event_payload(event.payload())?;
            world.insert(
                joined.person().entity_id(),
                Participation::new(joined.activity(), joined.kind().clone(), now),
            )?;
        } else if *kind == LeftGroupActivity::EVENT_TYPE {
            let left: LeftGroupActivity = codec::event_payload(event.payload())?;
            forget(world, left.person(), left.activity())?;
        } else if *kind == GroupActivityEnded::EVENT_TYPE {
            let ended: GroupActivityEnded = codec::event_payload(event.payload())?;
            for member in ended.members() {
                forget(world, *member, ended.activity())?;
            }
        } else if *kind == PersonEnteredPlace::EVENT_TYPE {
            let entered: PersonEnteredPlace = codec::event_payload(event.payload())?;
            let Some(activity) = participation(&world.read(), entered.person().entity_id())
                .map(Participation::activity)
            else {
                return Ok(Vec::new());
            };
            let elsewhere = world
                .read()
                .process(activity)
                .is_some_and(|process| process.place() != Some(entered.place()));
            if elsewhere {
                return depart(world, activity, entered.person());
            }
        }
        Ok(Vec::new())
    }

    /// The activity's hour is up: it ends, naming everyone who took part.
    fn wake(
        &self,
        world: &mut WorldView<'_, Self>,
        process: &Process,
    ) -> Result<Vec<Emission>, KernelError> {
        let (state, place) = activity(&world.read(), process.id())?;
        world.end_process::<GroupActivity>(process.id())?;
        Ok(vec![ended(process.id(), &state, place)])
    }
}

/// Reads an action payload as `A`, or refuses the request as malformed.
fn readable<A: Action>(intent: &ActionIntent) -> Result<A, Rejection> {
    codec::action_payload::<A>(intent.payload()).map_err(|error| Rejection::System {
        code: MALFORMED_PAYLOAD,
        detail: Some(error.to_string()),
    })
}

/// That entity as a person, if this world holds a person by that identity.
pub(crate) fn person(world: &WorldRead<'_>, entity: EntityId) -> Option<PersonId> {
    let record = world.entity(entity)?;
    PersonId::new(record.id(), record.entity_type()).ok()
}

/// Where presence says that entity is; [`None`] is *unknown*, not *nowhere*.
pub(crate) fn located(world: &WorldRead<'_>, entity: EntityId) -> Option<Location> {
    world.component::<Presence>(entity).map(Presence::location)
}

/// The activity that person is part of, if any.
pub(crate) fn participation<'a>(
    world: &WorldRead<'a>,
    entity: EntityId,
) -> Option<&'a Participation> {
    world.component::<Participation>(entity)
}

/// The invitation `invitee` holds from `inviter`, if it can still be answered at `now`.
pub(crate) fn open_invitation<'a>(
    world: &WorldRead<'a>,
    invitee: EntityId,
    inviter: EntityId,
    now: WorldTime,
) -> Option<&'a Invitation> {
    let inviter = person(world, inviter)?;
    world
        .component::<Invitations>(invitee)?
        .from(inviter)
        .filter(|invitation| invitation.is_open_at(now))
}

/// The running activity behind that process, if it is one of this pack's.
pub(crate) fn running<'a>(world: &WorldRead<'a>, activity: ProcessId) -> Option<&'a Process> {
    world
        .process(activity)
        .filter(|process| process.state_for::<GroupActivity>().is_ok())
}

/// This pack's state for a running activity, and where it is happening.
fn activity(world: &WorldRead<'_>, id: ProcessId) -> Result<(ActivityState, PlaceId), KernelError> {
    let missing = || KernelError::ProcessNotRunning { process: id };
    let process = world.process(id).ok_or_else(missing)?;
    let state = codec::state(process.state_for::<GroupActivity>()?).ok_or_else(missing)?;
    Ok((state, process.place().ok_or_else(missing)?))
}

/// The invitations a person holds, or none yet.
/// The seconds an invitation lasts: the world's section for this inviter, invitee and place, or the
/// compiled [`INVITATION_LIFETIME`](crate::INVITATION_LIFETIME) when the world configures none (and for
/// a fact with no place, which this pack never states).
fn lifetime(world: &WorldRead<'_>, invited: &Invited, place: Option<PlaceId>) -> i64 {
    let Some(place) = place else {
        return crate::INVITATION_LIFETIME.seconds();
    };
    let roles = Roles::new()
        .with(Role::Actor, invited.inviter().entity_id())
        .with(Role::Target, invited.invitee().entity_id());
    i64::from(
        interactions::parameters::<GroupActivitySystem>(world, place, &roles).invitation_lifetime,
    )
}

fn invitations(world: &WorldView<'_, GroupActivitySystem>, invitee: EntityId) -> Invitations {
    world
        .read()
        .component::<Invitations>(invitee)
        .cloned()
        .unwrap_or_default()
}

/// Forgets an answered invitation.
fn answered(
    world: &mut WorldView<'_, GroupActivitySystem>,
    invitee: PersonId,
    inviter: PersonId,
    now: WorldTime,
) -> Result<(), KernelError> {
    let mut held = invitations(world, invitee.entity_id());
    held.answered(inviter, now);
    world.insert(invitee.entity_id(), held)?;
    Ok(())
}

/// Removes a person's participation, if it names that activity.
fn forget(
    world: &mut WorldView<'_, GroupActivitySystem>,
    person: PersonId,
    activity: ProcessId,
) -> Result<(), KernelError> {
    let names_it = participation(&world.read(), person.entity_id())
        .is_some_and(|theirs| theirs.activity() == activity);
    if names_it {
        world.remove::<Participation>(person.entity_id())?;
    }
    Ok(())
}

/// Starts an activity of `kind` at `place`, founded by the inviter and the person who accepted.
fn begin(
    world: &mut WorldView<'_, GroupActivitySystem>,
    kind: ActivityKind,
    place: PlaceId,
    inviter: PersonId,
    invitee: PersonId,
) -> Result<Emission, KernelError> {
    let founders = vec![inviter, invitee];
    let state = ActivityState::begun(kind.clone(), founders.clone());
    let until = WorldTime::from_seconds(world.at().seconds() + ACTIVITY_LENGTH.seconds());
    let id = world.start_process(
        ProcessStart::<GroupActivity>::new(codec::encode(&state))
            .with_participants(founders.iter().map(|founder| founder.entity_id()).collect())
            .at_place(place)
            .ending_at(until),
    )?;
    Ok(GroupActivityStarted::new(id, kind, place, founders).emission())
}

/// Adds a member to a running activity.
fn join(
    world: &mut WorldView<'_, GroupActivitySystem>,
    id: ProcessId,
    person: PersonId,
) -> Result<Emission, KernelError> {
    let (mut state, place) = activity(&world.read(), id)?;
    state.join(person);
    world.set_process_state::<GroupActivity>(id, codec::encode(&state))?;
    Ok(JoinedGroupActivity::new(id, state.kind().clone(), person).emission(place))
}

/// A member leaves; below two members, the activity ends.
fn depart(
    world: &mut WorldView<'_, GroupActivitySystem>,
    id: ProcessId,
    person: PersonId,
) -> Result<Vec<Emission>, KernelError> {
    let (mut state, place) = activity(&world.read(), id)?;
    state.leave(person);
    let left = LeftGroupActivity::new(id, state.kind().clone(), person).emission(place);
    if state.is_over() {
        world.end_process::<GroupActivity>(id)?;
        return Ok(vec![left, ended(id, &state, place)]);
    }
    world.set_process_state::<GroupActivity>(id, codec::encode(&state))?;
    Ok(vec![left])
}

/// `group-activity-ended`, naming everyone who took part.
fn ended(id: ProcessId, state: &ActivityState, place: PlaceId) -> Emission {
    GroupActivityEnded::new(id, state.kind().clone(), place, state.took_part().to_vec()).emission()
}
