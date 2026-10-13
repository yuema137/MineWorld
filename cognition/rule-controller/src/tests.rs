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
use mineworld_naming::{DisplayName, NAME_MAX_BYTES, Name};
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
    alices_view_naming(heard, available_against, &[])
}

/// [`alices_view`], with the `display-name` records the world discloses about some of the people she
/// perceives — as `naming` does for everybody it names.
fn alices_view_naming(
    heard: &[Heard],
    available_against: &[u64],
    names: &[(u64, &str)],
) -> Observation<Value> {
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
        let disclosed: Vec<ComponentRecord<Value>> = names
            .iter()
            .filter(|(named, _)| *named == speaker)
            .map(|(_, name)| {
                let name = DisplayName::new(Name::new(*name).expect("a legal name"));
                ComponentRecord::new::<DisplayName>(
                    id(speaker),
                    serde_json::to_value(&name).expect("serializes"),
                )
            })
            .collect();
        entities.push(
            PerceivedEntity::new(id(speaker), EntityType::Person)
                .at(here)
                .with_components(disclosed),
        );
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

/// `F-13`: a controller bound at an instant answers only what was said after it. A server restarted
/// on a save builds Alice's controller afresh; the line she answered before the crash is still the
/// newest in her history, and it is not hers to answer again.
#[test]
fn bound_at_an_instant_she_answers_only_what_is_said_after_it() {
    let bound = WorldTime::from_seconds(3_500);
    let mut alice = RuleController::since(bound);
    let before = alices_view(&[said(VISITOR, "said before the crash", 3_500)], &[VISITOR]);
    assert!(
        alice.decide(&before).is_none(),
        "a line heard at the binding instant was said to whoever drove her before"
    );
    assert!(alice.is_silent(), "and nothing was marked answered either");

    let after = alices_view(
        &[
            said(VISITOR, "said before the crash", 3_500),
            said(WANDERER, "said after the restart", 3_501),
        ],
        &[VISITOR, WANDERER],
    );
    let request = alice
        .decide(&after)
        .expect("a line one second after the binding");
    assert_eq!(requested(&request).1, Some(id(WANDERER)));
    assert!(
        alice.decide(&after).is_none(),
        "the old line is still not hers, and the new one is answered"
    );

    let mut unbound = RuleController::new();
    assert!(
        unbound.decide(&before).is_some(),
        "new() answers every line, as it always did"
    );
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
    // Replaced claim (step-09 §4.3.3 C4): this view discloses no name, so the other speaker is
    // "someone else" — and never an entity id, which the operator saw as "person 4" in the 3D slice.
    assert!(
        words.contains("Earlier, someone else said"),
        "with no name disclosed, the other speaker is someone else: {words}"
    );
    assert!(
        !words.contains(&VISITOR.to_string()),
        "and never an entity id: {words}"
    );
}

/// The same sentence in a world that names people: the other speaker by the name the observation
/// discloses about them.
#[test]
fn she_names_the_other_speaker_by_the_name_she_is_told() {
    let mut alice = RuleController::new();
    let both = alices_view_naming(
        &[
            said(VISITOR, "hello from the 2D window", 3_500),
            said(WANDERER, "hello from the 3D window", 3_560),
        ],
        &[VISITOR, WANDERER],
        &[(VISITOR, "Vera Lindgren"), (WANDERER, "Wes Calloway")],
    );
    let _to_visitor = alice.decide(&both).expect("the visitor first");
    let words = requested(&alice.decide(&both).expect("then the wanderer")).2;
    assert!(
        words.contains("Earlier, Vera Lindgren said \"hello from the 2D window\""),
        "the earlier speaker, by name: {words}"
    );
    assert!(
        !words.contains("Wes Calloway"),
        "and not the name of the person she is answering, who knows what they said: {words}"
    );
}

/// The longest legal name beside two quotations at their bound still fits an utterance, so naming
/// somebody can never make a controller fall silent.
#[test]
fn the_longest_name_still_fits_a_reply() {
    let mut alice = RuleController::new();
    let long = "y".repeat(mineworld_conversation::UTTERANCE_MAX_BYTES);
    let longest = "N".repeat(NAME_MAX_BYTES);
    let both = alices_view_naming(
        &[said(VISITOR, &long, 3_500), said(WANDERER, &long, 3_560)],
        &[VISITOR, WANDERER],
        &[(VISITOR, &longest)],
    );
    let _to_visitor = alice.decide(&both).expect("the visitor first");
    let words = requested(&alice.decide(&both).expect("a reply, not silence")).2;
    assert!(words.contains(&longest), "{words}");
    assert!(words.len() <= mineworld_conversation::UTTERANCE_MAX_BYTES);
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

/// The reactive controller answers and takes no initiative, so it never attempts a complete affordance
/// — offered or not, at any instant, before or after it has somebody to answer (`ARC-34` point 5,
/// `AC-15`). The offer band is the paced rule's alone.
#[test]
fn the_reactive_controller_never_attempts_an_offer() {
    let ring = mineworld_contracts::ActionTypeId::from_static("ring");
    let complete = |bell: &str| {
        Affordance::available(ring.clone(), None, SpatialRequirement::NONE)
            .with_payload(serde_json::json!({ "bell": bell }))
    };
    let heard = [said(VISITOR, "is the hall open?", 3_500)];
    for history in [&heard[..], &[]] {
        let mut alice = RuleController::new();
        let mut decided = 0;
        for k in 0..256 {
            let seen = alices_view(history, &[VISITOR]);
            let offered = Observation::new(id(ALICE), WorldTime::from_seconds(3_600 + k))
                .at_location(*seen.self_location().expect("Alice is somewhere"))
                .perceiving(seen.entities().to_vec())
                .offering(
                    seen.affordances()
                        .iter()
                        .cloned()
                        .chain([complete("low"), complete("high")])
                        .collect(),
                );
            if let Some(request) = alice.decide(&offered) {
                decided += 1;
                assert_eq!(
                    request.action_type(),
                    &Talk::ACTION_TYPE,
                    "the reactive controller only ever answers"
                );
            }
        }
        assert!(decided <= 1, "it answers a line once, and nothing else");
    }
}
