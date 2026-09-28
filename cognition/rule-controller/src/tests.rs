//! What the rule actually does, decided from observations built by hand.
//!
//! Hand-built rather than produced by a running world, deliberately: this is the layer that owns
//! *"given exactly this view, what does this Person attempt"*, and an observation assembled here
//! cannot agree with the implementation by accident. Whether a real world produces such an
//! observation is a different claim, owned by the acceptance test that runs the real server
//! (`tools/cli/tests/ac15_one_alice.rs`).

use mineworld_contracts::{
    Action, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition, Location,
    Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, Rejection, SpatialRequirement,
    WorldTime,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, Utterance, talk_requirement};
use serde_json::Value;

use super::RuleController;

const ALICE: u64 = 2;
const VISITOR: u64 = 4;
const WANDERER: u64 = 5;
const CAFE: u64 = 1;
const NOW: WorldTime = WorldTime::from_seconds(3_600);

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn person(raw: u64) -> PersonId {
    PersonId::new(id(raw), EntityType::Person).expect("a person")
}

fn said(speaker: u64, words: &str, at: i64) -> Heard {
    Heard::new(
        person(speaker),
        WorldTime::from_seconds(at),
        Utterance::new(words).expect("a legal utterance"),
    )
}

/// What Alice's observation looks like: herself, her disclosed history, and the verdicts the server
/// reached about whom she may speak to.
fn alices_view(heard: &[Heard], available_against: &[u64]) -> Observation<Value> {
    let mut history = ConversationHistory::default();
    for entry in heard {
        history.remember(entry.clone());
    }
    let place = PlaceId::new(id(CAFE), EntityType::Place).expect("a place");
    let here = Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(1_200),
        Millimetres::new(2_400),
    ));

    let me = PerceivedEntity::new(id(ALICE), EntityType::Person)
        .at(here)
        .with_components(vec![ComponentRecord::new::<ConversationHistory>(
            id(ALICE),
            serde_json::to_value(&history).expect("a component serializes"),
        )]);
    let mut entities = vec![me];
    let mut affordances = Vec::new();
    for speaker in [VISITOR, WANDERER] {
        entities.push(PerceivedEntity::new(id(speaker), EntityType::Person).at(here));
        // The server's verdict, which this controller reads and never computes.
        affordances.push(if available_against.contains(&speaker) {
            Affordance::available(Talk::ACTION_TYPE, Some(id(speaker)), talk_requirement())
        } else {
            Affordance::unavailable(
                Talk::ACTION_TYPE,
                Some(id(speaker)),
                talk_requirement(),
                Rejection::TooFarAway,
            )
        });
    }

    Observation::new(id(ALICE), NOW)
        .at_location(here)
        .perceiving(entities)
        .offering(affordances)
}

/// What a request asks for, as a reader of the frame would see it.
fn requested(request: &mineworld_contracts::ActionRequest) -> (EntityId, Option<EntityId>, String) {
    let talk: Talk = serde_json::from_slice(
        request
            .payload()
            .payload_for::<Talk>()
            .expect("the record is labelled talk"),
    )
    .expect("a talk payload");
    (
        request.actor(),
        request.target(),
        talk.utterance().as_str().to_owned(),
    )
}

/// The ordinary case: somebody spoke, the server says she may answer, and she answers them.
#[test]
fn she_answers_the_person_who_spoke_to_her() {
    let mut alice = RuleController::new();
    let view = alices_view(&[said(VISITOR, "hello Alice", 3_500)], &[VISITOR]);

    let request = alice.decide(&view).expect("she has something to say");
    let (actor, target, words) = requested(&request);

    assert_eq!(actor, id(ALICE), "she acts as herself and as nobody else");
    assert_eq!(target, Some(id(VISITOR)));
    assert!(
        words.contains("hello Alice"),
        "she quotes what was said, which is how a player can tell she heard it: {words}"
    );
    assert_eq!(
        request.actor_location(),
        None,
        "a controller reports no position: it is not standing anywhere, and the server decides \
         where its Person is anyway"
    );
    assert!(!alice.is_silent());
}

/// The observation arrives ten times a second. She answers once.
#[test]
fn she_does_not_answer_the_same_words_twice_and_does_answer_new_ones() {
    let mut alice = RuleController::new();
    let first = alices_view(&[said(VISITOR, "hello Alice", 3_500)], &[VISITOR]);

    assert!(alice.decide(&first).is_some());
    assert!(
        alice.decide(&first).is_none(),
        "an unchanged observation is not a new thing to say"
    );
    assert!(
        alice
            .decide(&alices_view(
                &[said(VISITOR, "hello Alice", 3_500)],
                &[VISITOR]
            ))
            .is_none(),
        "and neither is an identical observation rebuilt from scratch"
    );

    let again = alices_view(
        &[
            said(VISITOR, "hello Alice", 3_500),
            said(VISITOR, "what is there to eat", 3_560),
        ],
        &[VISITOR],
    );
    let request = alice.decide(&again).expect("new words, a new answer");
    assert!(requested(&request).2.contains("what is there to eat"));
}

/// The server said no, so she does nothing — and nothing here measured a distance to find that out.
#[test]
fn she_does_not_speak_when_the_server_says_she_may_not() {
    let mut alice = RuleController::new();
    let out_of_reach = alices_view(&[said(VISITOR, "hello Alice", 3_500)], &[]);

    assert!(
        alice.decide(&out_of_reach).is_none(),
        "an unavailable affordance is the whole of the answer (ENGINEERING_RULES §8)"
    );
    assert!(
        alice.is_silent(),
        "and she has not marked it answered either, so she will answer when he comes closer"
    );

    let in_reach = alices_view(&[said(VISITOR, "hello Alice", 3_500)], &[VISITOR]);
    assert!(alice.decide(&in_reach).is_some());
}

/// `AC-15`'s sentence. Alice tells the second person what the first one said, which only a Person who
/// holds both conversations can do.
#[test]
fn she_tells_the_second_person_what_the_first_one_said() {
    let mut alice = RuleController::new();
    let both = alices_view(
        &[
            said(VISITOR, "hello from the 2D window", 3_500),
            said(WANDERER, "hello from the 3D window", 3_560),
        ],
        &[VISITOR, WANDERER],
    );

    // Lowest identity first, so the order two people are answered in is reproducible.
    let to_visitor = alice.decide(&both).expect("the visitor is answered first");
    assert_eq!(requested(&to_visitor).1, Some(id(VISITOR)));

    let to_wanderer = alice.decide(&both).expect("and then the wanderer");
    let (_, target, words) = requested(&to_wanderer);
    assert_eq!(target, Some(id(WANDERER)));
    assert!(
        words.contains("hello from the 2D window"),
        "the other window's words, carried forward by the person met in this one: {words}"
    );
    assert!(
        words.contains(&VISITOR.to_string()),
        "and the other speaker named by identity, because this world has no display name to use: \
         {words}"
    );
}

/// Nothing disclosed is nothing known. A controller cannot ask again, so it does nothing.
#[test]
fn a_person_told_nothing_says_nothing() {
    let mut alice = RuleController::new();
    let silent = Observation::new(id(ALICE), NOW)
        .perceiving(vec![PerceivedEntity::new(id(ALICE), EntityType::Person)])
        .offering(vec![Affordance::available(
            Talk::ACTION_TYPE,
            Some(id(VISITOR)),
            SpatialRequirement::same_place(),
        )]);

    assert!(
        alice.decide(&silent).is_none(),
        "an observation with no disclosed history gives a controller nothing to act on"
    );
    assert!(
        alice.decide(&Observation::new(id(ALICE), NOW)).is_none(),
        "and an empty observation is a legitimate view of a world, not an error"
    );
}

/// A long utterance is quoted, not echoed whole: the reply has to fit the contract's bound, and it
/// does so by construction rather than by luck.
#[test]
fn a_very_long_utterance_is_quoted_short_enough_to_reply_to() {
    let mut alice = RuleController::new();
    let long = "x".repeat(mineworld_conversation::UTTERANCE_MAX_BYTES);
    let view = alices_view(&[said(VISITOR, &long, 3_500)], &[VISITOR]);

    let request = alice
        .decide(&view)
        .expect("a long thing said is still something to answer");
    let words = requested(&request).2;
    assert!(
        words.len() <= mineworld_conversation::UTTERANCE_MAX_BYTES,
        "the reply fits what the contract accepts: {} bytes",
        words.len()
    );
    assert!(words.contains('…'), "and says that it was cut: {words}");
}
