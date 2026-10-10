//! The facts this pack records.
//!
//! Every payload names its people, as `Spoke` does, and every envelope lists them as subjects and
//! participants. Both matter: a reducer reads the typed payload, never a position in a list, and a
//! biography selects by the envelope (`ARC-29`), so a fact that left somebody out of its envelope would
//! be missing from their life.

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, PersonId, PlaceId, ProcessId, SystemId,
    Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::kind::ActivityKind;
use crate::system::GroupActivitySystem;

macro_rules! fact {
    ($type:ty, $name:literal) => {
        impl Event for $type {
            const EVENT_TYPE: EventTypeId = EventTypeId::from_static($name);
            const OWNER: SystemId = GroupActivitySystem::ID;
            const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
        }
    };
}

/// The three facts about an invitation share one shape: who invited whom to do what.
macro_rules! invitation_fact {
    ($(#[$doc:meta])* $type:ident, $name:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $type {
            inviter: PersonId,
            invitee: PersonId,
            kind: ActivityKind,
        }

        fact!($type, $name);

        impl $type {
            /// States it.
            pub const fn new(inviter: PersonId, invitee: PersonId, kind: ActivityKind) -> Self {
                Self {
                    inviter,
                    invitee,
                    kind,
                }
            }

            /// Who invited.
            pub const fn inviter(&self) -> PersonId {
                self.inviter
            }

            /// Who was invited.
            pub const fn invitee(&self) -> PersonId {
                self.invitee
            }

            /// What was proposed.
            pub const fn kind(&self) -> &ActivityKind {
                &self.kind
            }

            /// The fact, as this pack records it, at `place`, heard by `audience` — the two of them
            /// unless the world's list says otherwise (it can only narrow).
            pub(crate) fn emission(&self, place: PlaceId, audience: Visibility) -> Emission {
                let both = vec![self.inviter.entity_id(), self.invitee.entity_id()];
                Emission::new::<Self>(codec::encode(self), audience)
                    .about(vec![self.invitee.entity_id()])
                    .with_participants(both)
                    .at_place(place)
            }
        }
    };
}

invitation_fact!(
    /// Somebody asked somebody to do something together.
    Invited,
    "invited"
);
invitation_fact!(
    /// An invitation was accepted.
    InvitationAccepted,
    "invitation-accepted"
);
invitation_fact!(
    /// An invitation was declined.
    InvitationDeclined,
    "invitation-declined"
);

/// The two facts about a whole activity share one shape: which, what, where, and who.
macro_rules! activity_fact {
    ($(#[$doc:meta])* $type:ident, $name:literal, $members_doc:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $type {
            activity: ProcessId,
            kind: ActivityKind,
            place: PlaceId,
            members: Vec<PersonId>,
        }

        fact!($type, $name);

        impl $type {
            /// States it.
            pub const fn new(
                activity: ProcessId,
                kind: ActivityKind,
                place: PlaceId,
                members: Vec<PersonId>,
            ) -> Self {
                Self {
                    activity,
                    kind,
                    place,
                    members,
                }
            }

            /// The activity's process.
            pub const fn activity(&self) -> ProcessId {
                self.activity
            }

            /// What it is.
            pub const fn kind(&self) -> &ActivityKind {
                &self.kind
            }

            /// Where.
            pub const fn place(&self) -> PlaceId {
                self.place
            }

            #[doc = $members_doc]
            pub fn members(&self) -> &[PersonId] {
                &self.members
            }

            /// The fact, as this pack records it: about every member, seen by `audience` — anyone in
            /// the place unless the world's list narrows it.
            pub(crate) fn emission(&self, audience: Visibility) -> Emission {
                let people: Vec<EntityId> =
                    self.members.iter().map(|member| member.entity_id()).collect();
                Emission::new::<Self>(codec::encode(self), audience)
                    .about(people.clone())
                    .with_participants(people)
                    .at_place(self.place)
            }
        }
    };
}

activity_fact!(
    /// Two people began doing something together.
    GroupActivityStarted,
    "group-activity-started",
    "Who began it."
);
activity_fact!(
    /// An activity ended — at its expected end, or because fewer than two remained.
    GroupActivityEnded,
    "group-activity-ended",
    "Everyone who took part, in the order they came — not only who was left at the end."
);

/// The two facts about one person and an activity share one shape.
macro_rules! membership_fact {
    ($(#[$doc:meta])* $type:ident, $name:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct $type {
            activity: ProcessId,
            kind: ActivityKind,
            person: PersonId,
        }

        fact!($type, $name);

        impl $type {
            /// States it.
            pub const fn new(activity: ProcessId, kind: ActivityKind, person: PersonId) -> Self {
                Self {
                    activity,
                    kind,
                    person,
                }
            }

            /// The activity's process.
            pub const fn activity(&self) -> ProcessId {
                self.activity
            }

            /// What it is.
            pub const fn kind(&self) -> &ActivityKind {
                &self.kind
            }

            /// Who.
            pub const fn person(&self) -> PersonId {
                self.person
            }

            /// The fact, as this pack records it, at the activity's place, seen by `audience` — anyone
            /// there unless the world's list narrows it.
            pub(crate) fn emission(&self, place: PlaceId, audience: Visibility) -> Emission {
                Emission::new::<Self>(codec::encode(self), audience)
                    .about(vec![self.person.entity_id()])
                    .with_participants(vec![self.person.entity_id()])
                    .at_place(place)
            }
        }
    };
}

membership_fact!(
    /// Somebody joined an activity already under way.
    JoinedGroupActivity,
    "joined-group-activity"
);
membership_fact!(
    /// Somebody stopped taking part — by choice, or by walking into another place.
    LeftGroupActivity,
    "left-group-activity"
);
