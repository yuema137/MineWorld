//! One consult of the `motivated` model (`pr-dl-b-motivated.md` §5.2, §5.5, §5.6).
//!
//! ```text
//! 1  invitation: an open one I may accept  → accept or decline, by a scored yes/no (draw 2)
//! 2  reply: an in-window line I may answer → reply or not, by a scored yes/no with the floor (draw 1)
//! 3  score every offered choice; choose by a seeded draw in the top band; a door by its window
//! 4  the chosen choice's request, built by `options::request` from the draws it needs
//! ```
//!
//! `decide` is a function of `(settings, seed, pace, observation)`: no state is kept between consults (ARC-27),
//! and every draw is `Draw::salted` with this model's own salt, so it is independent of `rule`'s.

use mineworld_contracts::{ActionRequest, Observation, PersonId, SimDuration};
use mineworld_conversation::Talk;
use mineworld_decision::{Draw, mix};
use mineworld_group_activity::{AcceptInvitation, DeclineInvitation, Invitations};
use mineworld_rule_controller::{act, read};
use serde_json::Value;

use crate::considerations;
use crate::options::{self, Choice, Context, KINDS, WANDER_AXIS};
use crate::score::{self, combine};
use crate::settings::Settings;

/// The salt of this model's draws: "motivate" in ASCII, so its draws are independent of `rule`'s.
pub const MOTIVATED_SALT: u64 = 0x6d6f_7469_7661_7465;

/// The tag mixed into the commitment window's door draw: "door" in ASCII.
const DOOR_TAG: u64 = 0x646f_6f72;

/// The draw index of the reply yes/no (SD-DLb-5).
const REPLY_DRAW: u64 = 1;

/// The draw index of the invitation yes/no (SD-DLb-5).
const ACCEPT_DRAW: u64 = 2;

/// The draw index of the greeting's wording (SD-DLb-5).
const GREETING_DRAW: u64 = 3;

/// The draw indices of the wander offsets, one per axis (SD-DLb-5).
const WANDER_DRAWS: (u64, u64) = (4, 5);

/// The draw index of the invitation's kind (`rule`'s `KIND_DRAW`, which this model reuses).
const KIND_DRAW: u64 = 11;

/// Decides one consult. `None` when the model submits nothing: it chose to stay, or the world offers nothing.
pub fn decide(
    settings: &Settings,
    seed: u64,
    pace: SimDuration,
    observation: &Observation<Value>,
) -> Option<ActionRequest> {
    let context = Context::new(observation)?;
    let draw = Draw::salted(MOTIVATED_SALT, seed, observation);
    if let Some(answer) = invitation(&context, settings, &draw) {
        return answer;
    }
    if let Some(answer) = reply(&context, settings, &draw, pace) {
        return answer;
    }
    let scored = score::score_all(&context, settings);
    let window = context.now.div_euclid(settings.commit_minutes.max(1) * 60);
    let door_key = mix(
        mix(seed ^ MOTIVATED_SALT ^ DOOR_TAG, context.me.raw()),
        window.cast_unsigned(),
    );
    let choice = score::select(&scored, settings.top_band, &draw, door_key)?.clone();
    let greeting = greeting(&draw);
    let kind = KINDS[usize::try_from(draw.below(KINDS.len() as u64, KIND_DRAW)).ok()?];
    let wander = (
        draw.offset(WANDER_AXIS, WANDER_DRAWS.0),
        draw.offset(WANDER_AXIS, WANDER_DRAWS.1),
    );
    options::request(&choice, &context, greeting, kind, wander)
}

/// The next stride of the observer's own walk, between consults (`rule`'s `step`; takes no draw).
pub fn step(observation: &Observation<Value>) -> Option<ActionRequest> {
    act::step(observation)
}

/// The wording a greeting is chosen with: one of the rule's fixed greetings, by its draw.
fn greeting(draw: &Draw) -> &'static str {
    let index = draw.below(act::GREETINGS.len() as u64, GREETING_DRAW);
    act::GREETINGS[usize::try_from(index).unwrap_or(0)]
}

/// The yes/no weight of a person: the adjusted product of sociability, familiarity and regard (§5.6).
fn yes_weight(context: &Context<'_>, settings: &Settings, person: PersonId) -> u32 {
    let values: Vec<u32> = considerations::of(&Choice::Join(person), context, settings)
        .iter()
        .map(|(_, value)| *value)
        .collect();
    combine(1_000, &values)
}

/// An open invitation I may answer: accept by the draw, else decline if the world offers that. `None` when
/// there is no such invitation, so the consult goes on to the reply and the choice.
fn invitation(
    context: &Context<'_>,
    settings: &Settings,
    draw: &Draw,
) -> Option<Option<ActionRequest>> {
    let observation = context.observation;
    let held: Invitations = read::disclosed(observation.entity(context.me)?)?;
    let now = observation.at();
    let invitation = held.pending().iter().find(|invitation| {
        let inviter = invitation.from().entity_id();
        invitation.is_open_at(now)
            && read::available::<AcceptInvitation>(observation, Some(inviter))
    })?;
    let inviter = invitation.from();
    let target = inviter.entity_id();
    if draw.below(1_000, ACCEPT_DRAW) < u64::from(yes_weight(context, settings, inviter)) {
        return Some(Some(
            act::request(context.me, &AcceptInvitation {}).with_target(target),
        ));
    }
    if read::available::<DeclineInvitation>(observation, Some(target)) {
        return Some(Some(
            act::request(context.me, &DeclineInvitation {}).with_target(target),
        ));
    }
    Some(None)
}

/// The reply to the newest line in this consult's window, from a person I may answer, by the scored yes/no.
/// `None` when there is no such line or the yes/no says no, so the consult goes on to the choice.
fn reply(
    context: &Context<'_>,
    settings: &Settings,
    draw: &Draw,
    pace: SimDuration,
) -> Option<Option<ActionRequest>> {
    let observation = context.observation;
    let history = read::disclosed_history(observation.entity(context.me)?)?;
    let newest = read::newest_per_speaker(&history, context.me);
    let now = context.now;
    let opened = now.checked_sub(pace.seconds())?;
    let (speaker, heard) = newest.iter().find(|(speaker, heard)| {
        let at = heard.at().seconds();
        opened < at && at <= now && read::may_talk_to(observation, **speaker)
    })?;
    let person = heard.speaker();
    let threshold = u64::from(yes_weight(context, settings, person).max(settings.reply_floor));
    if draw.below(1_000, REPLY_DRAW) >= threshold {
        return None;
    }
    let said = act::reply_to(heard, &newest, *speaker, observation)?;
    Some(Some(
        ActionRequest::new(context.me, act::record(&Talk::new(said))).with_target(*speaker),
    ))
}
