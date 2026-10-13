//! What this pack puts in an observation: the actions it offers, and the state it lets be known.
//!
//! ```text
//! offers      against nobody   leave-group-activity, when the observer is part of an activity
//!             against a person invite (available when they are part of no activity);
//!                              accept and decline, when the observer holds an invitation from them
//!                              (accept only while the observer is part of nothing);
//!                              join, when they are part of an activity and the observer is not
//! discloses   to the observer  its own Invitations and its own Participation
//!             about a member   that member's Participation — "they are together" is visible to
//!                              anyone who can see them (SD-11)
//! ```
//!
//! An offer carries the availability only this pack can judge; perception prices space. An offer
//! cannot see the clock (`PerceptionProvider` is handed no instant), so an invitation past its
//! [`Invitation::until`](crate::Invitation::until) that has not been written over yet is still offered
//! and is refused `PreconditionFailed` at dispatch. A controller reads the invitation's own `until` —
//! the lifetime the world's section gave it — rather than finding out by asking (`step-09-social.md`
//! §9 E-B2; S17's PR IL-b).

use mineworld_contracts::{
    Action, ActionTypeId, ComponentRecord, EntityId, EntityType, LifecycleState,
};
use mineworld_kernel::WorldRead;
use mineworld_presence::{Offer, PerceptionProvider};
use serde_json::Value;

use crate::action::{
    AcceptInvitation, DeclineInvitation, Invite, JoinGroupActivity, LeaveGroupActivity,
    accept_requirement, decline_requirement, invite_requirement, join_requirement,
    leave_requirement,
};
use crate::codec;
use crate::component::{Invitations, Participation};
use crate::interactions::{answered, invite_terms, pair, permits};
use crate::system::{GroupActivitySystem, located, participation, person, running};

impl PerceptionProvider for GroupActivitySystem {
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        let Some(me) = person(world, observer) else {
            return Vec::new();
        };
        let mine = participation(world, observer);
        let Some(target) = target else {
            return if mine.is_some() {
                vec![Offer::new::<LeaveGroupActivity>(leave_requirement())]
            } else {
                Vec::new()
            };
        };
        if target == observer {
            return Vec::new();
        }
        let Some(record) = world.entity(target) else {
            return Vec::new();
        };
        if record.entity_type() != EntityType::Person {
            return Vec::new();
        }
        let active = record.lifecycle() == LifecycleState::Active;
        let theirs = participation(world, target);
        // The world's list, asked as dispatch asks it: at the observer's place, with the observer
        // acting on the target. An observer with no place is answered with the compiled default, as
        // its dispatch is refused `PreconditionFailed` before the list is asked.
        let roles = pair(observer, target);
        let here = located(world, observer).map(|here| here.place());
        let (invite_permitted, invite) = here.map_or_else(
            || (Ok(()), invite_requirement()),
            |place| invite_terms(world, place, &roles),
        );
        let permitted = |action: &ActionTypeId| {
            here.map_or(Ok(()), |place| permits(world, place, action, &roles))
        };

        let mut offers = vec![answered(
            Offer::new::<Invite>(invite).with_target_available(active && theirs.is_none()),
            invite_permitted,
        )];
        let invited_by_them = person(world, target).is_some_and(|inviter| {
            world
                .component::<Invitations>(me.entity_id())
                .is_some_and(|held| held.from(inviter).is_some())
        });
        if invited_by_them {
            if mine.is_none() {
                offers.push(answered(
                    Offer::new::<AcceptInvitation>(accept_requirement())
                        .with_target_available(active),
                    permitted(&AcceptInvitation::ACTION_TYPE),
                ));
            }
            offers.push(
                Offer::new::<DeclineInvitation>(decline_requirement())
                    .with_target_available(active),
            );
        }
        if mine.is_none()
            && let Some(theirs) = theirs
        {
            offers.push(answered(
                Offer::new::<JoinGroupActivity>(join_requirement())
                    .with_target_available(active && running(world, theirs.activity()).is_some()),
                permitted(&JoinGroupActivity::ACTION_TYPE),
            ));
        }
        offers
    }

    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let mut records = Vec::new();
        if subject == observer
            && let Some(held) = world.component::<Invitations>(observer)
            && !held.is_empty()
        {
            records.push(ComponentRecord::new::<Invitations>(
                observer,
                codec::to_value(held),
            ));
        }
        if let Some(theirs) = world.component::<Participation>(subject) {
            records.push(ComponentRecord::new::<Participation>(
                subject,
                codec::to_value(theirs),
            ));
        }
        records
    }
}
