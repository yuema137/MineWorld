//! Request builders shared by the decision models: what to submit, built from an option the observation
//! offers (`pr-dl-b-motivated.md` SD-DLb-1, F-DLb-1).
//!
//! Every function here builds a request and takes no draw. A model that asks for a walk, a reply or a
//! greeting builds it through this module, so the wording and the payload encoding are one implementation.
//! They moved here with their bodies unchanged; the draw that chooses between them stays with each model.

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionRecord, ActionRequest, EntityId, Location, Observation, PlaceId,
};
use mineworld_conversation::{Heard, UTTERANCE_MAX_BYTES, Utterance};
use mineworld_movement::{Destination, WalkStep, WalkTo};
use serde::Serialize;
use serde_json::Value;

use crate::read::{disclosed_name, heading, walks};

/// How much of somebody's words this controller quotes back.
///
/// A bound, because an utterance is bounded ([`UTTERANCE_MAX_BYTES`]) and a reply that quotes two
/// of them plus its own words would otherwise be refused by the contract for length. Quoting is
/// deliberately literal within the bound: this controller does not summarize, because summarizing
/// is interpretation and interpretation is what MVP-1's cognition layer is for.
const QUOTED_AT_MOST: usize = 80;

/// What this controller marks a truncated quotation with.
const ELLIPSIS: &str = "…";

/// What a person says when they speak first. A fixed set, because a rule does not compose speech.
pub const GREETINGS: [&str; 6] = [
    "Good morning.",
    "Lovely day, isn't it?",
    "Have you been here long?",
    "Hello again.",
    "Busy in here today.",
    "Nice to see you.",
];

/// What a person who is not named in the observation is called in a reply.
pub const SOMEONE: &str = "someone else";

/// What a walking band chose: a request, or to carry on with the walk already under way.
pub enum Walked {
    /// Ask for this walk.
    Ask(ActionRequest),
    /// My own disclosed walk already goes there: ask for nothing (step-11 N-D14).
    Continue,
}

impl Walked {
    /// The request to make, if any.
    pub fn request(self) -> Option<ActionRequest> {
        match self {
            Self::Ask(request) => Some(request),
            Self::Continue => None,
        }
    }
}

/// A `walk-to` request for `to`, if the world offers `walk-to` to me; [`Walked::Continue`] when my own
/// walk already goes there. The route is the world's: this names a destination and nothing else.
pub fn walk_to(observation: &Observation<Value>, to: Destination) -> Option<Walked> {
    if heading(observation) == Some(to) {
        return Some(Walked::Continue);
    }
    let offered = observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == WalkTo::ACTION_TYPE && affordance.is_available()
    });
    offered.then(|| {
        let payload = serde_json::to_vec(&WalkTo::new(to))
            .expect("a request payload is JSON-representable by construction");
        Walked::Ask(ActionRequest::new(
            observation.observer(),
            ActionRecord::new::<WalkTo>(payload),
        ))
    })
}

/// A walk into `place`, wherever it is: movement plans the way, through as many doorways as it takes
/// (`ARC-75`; step-11 N-D15). The destination is the place itself, with no point in it, so the walk
/// ends on entering it. Nothing here looks for a door.
pub fn head_for(observation: &Observation<Value>, place: PlaceId) -> Option<Walked> {
    observation.self_location()?;
    walk_to(observation, Destination::Place(Location::in_place(place)))
}

/// The next stride of the observer's own walk: a `walk-step` when their walk is disclosed and the
/// world offers `walk-step`, otherwise nothing. A sleeper takes no stride: a walk begun before bed waits
/// for the sleeper to wake (`ARC-286`; a stride would wake them as acting).
pub fn step(observation: &Observation<Value>) -> Option<ActionRequest> {
    let offered = observation.affordances().iter().any(|affordance| {
        *affordance.action_type() == WalkStep::ACTION_TYPE && affordance.is_available()
    });
    (offered && walks(observation) && !crate::needs::i_sleep(observation)).then(|| {
        let payload = serde_json::to_vec(&WalkStep::default())
            .expect("an empty payload is JSON-representable");
        ActionRequest::new(
            observation.observer(),
            ActionRecord::new::<WalkStep>(payload),
        )
    })
}

/// What to say back: what you just said, and evidence that I remember somebody else saying
/// something.
///
/// The second clause is the whole point of the controller in `AC-15`. A player speaks to Alice in one
/// window; a *different* player walks up to her in another and is told what the first one said. That
/// sentence can only be produced by a Person who holds both conversations, which is what "there is
/// only one Alice" means.
///
/// The other speaker is named by the display name the observation discloses about them — `naming`'s
/// `display-name`, read here and never invented, since a name is component state and a controller
/// that made one up would be inventing world data (`DECISIONS.md` `ARC-31`). When the observation
/// discloses none — they have left the room, or the world has no `naming` — they are "someone else":
/// never an entity id, which is the world's bookkeeping and not something a person says.
pub fn reply_to(
    heard: &Heard,
    newest: &BTreeMap<EntityId, &Heard>,
    speaker: EntityId,
    observation: &Observation<Value>,
) -> Option<Utterance> {
    let mine = quoted(heard.utterance().as_str());
    let said = match newest
        .iter()
        .find(|(other, _)| **other != speaker)
        .map(|(other, theirs)| (*other, quoted(theirs.utterance().as_str())))
    {
        Some((other, theirs)) => {
            let who = disclosed_name(observation, other).unwrap_or_else(|| SOMEONE.to_owned());
            format!("I remember you. You said \"{mine}\". Earlier, {who} said \"{theirs}\" to me.")
        }
        None => format!(
            "I remember you. You said \"{mine}\". You are the first person to speak to me here."
        ),
    };
    // The bound is the contract's, and it is checked rather than assumed: `quoted` keeps each
    // quotation short enough that the whole sentence fits, and if a future wording stopped fitting
    // the controller would fall silent instead of submitting a request the contract refuses.
    debug_assert!(
        said.len() <= UTTERANCE_MAX_BYTES,
        "a reply fits an utterance"
    );
    Utterance::new(said).ok()
}

/// Somebody's words, short enough to quote inside a reply.
///
/// Truncated on a character boundary, because an utterance is UTF-8 text and a byte slice through the
/// middle of a character is not.
pub fn quoted(said: &str) -> String {
    match said.char_indices().nth(QUOTED_AT_MOST) {
        None => said.to_owned(),
        Some((boundary, _)) => format!("{}{ELLIPSIS}", &said[..boundary]),
    }
}

/// This controller's own payload encoding, as [`RuleController`](crate::RuleController)'s.
pub fn record<A: Action + Serialize>(action: &A) -> ActionRecord {
    ActionRecord::new::<A>(
        serde_json::to_vec(action)
            .expect("a request payload is JSON-representable by construction"),
    )
}

/// The request `actor` makes of `action`, with no target: the caller adds one where the action has one.
pub fn request<A: Action + Serialize>(actor: EntityId, action: &A) -> ActionRequest {
    ActionRequest::new(
        actor,
        ActionRecord::new::<A>(
            serde_json::to_vec(action)
                .expect("a request payload is JSON-representable by construction"),
        ),
    )
}
