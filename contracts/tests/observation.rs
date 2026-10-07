//! Contract tests for the view a controller is given.
//!
//! Two properties are under test, and both are architectural rather than incidental: an observation
//! exposes exactly what it lists and offers no route to anything else (`INV-13`), and a client can
//! render an interaction prompt from the server's answer alone, without evaluating a rule
//! (`ENGINEERING_RULES.md` §§4, 8, 9).
//!
//! The scene is the one the 3D spike recorded in §7.3 of the design: a player standing near a
//! person, close enough to talk to them and too far from a second person, in a world that also
//! contains someone the player cannot perceive at all.

use mineworld_contracts::{
    Action, ActionTypeId, Affordance, Component, ComponentRecord, ComponentSchemaVersion,
    ComponentTypeId, ContractError, Entity, EntityId, EntityKey, EntityType, EntityTypeSet,
    LocalPosition, Location, Millidegrees, Millimetres, Observation, Orientation, PerceivedEntity,
    PlaceId, Rejection, Relation, RelationTypeDeclaration, RelationTypeId, SpatialRequirement,
    SystemId, Tag, Tags, WorldTime,
};
use serde::{Deserialize, Serialize};

/// The component a client reads a target's name from: world data owned by a system, never text the
/// kernel invented (`DD-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Named {
    name: String,
}

impl Component for Named {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("named");
    const OWNER: SystemId = SystemId::from_static("identity-stub");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// The action type the observer is offered. There is no `Action` implementation here on purpose:
/// an affordance carries the *name* of an action and its requirement, never its payload type, so a
/// client can be offered an action whose payload it could not construct. That the name comes from a
/// system's declaration is `contracts/tests/action.rs`'s business.
const TALK: ActionTypeId = ActionTypeId::from_static("talk");

fn cafe() -> PlaceId {
    PlaceId::new(EntityId::from_raw(7), EntityType::Place).unwrap()
}

fn named(name: &str) -> ComponentRecord<String> {
    ComponentRecord::new::<Named>(
        EntityId::from_raw(0),
        serde_json::to_string(&Named {
            name: name.to_owned(),
        })
        .unwrap(),
    )
}

fn at(x: i32, y: i32) -> Location {
    Location::in_place(cafe()).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

fn reach() -> SpatialRequirement {
    SpatialRequirement::same_place()
        .within(Millimetres::new(3_000))
        .expect("three metres is a legal reach")
}

/// The scene: the player perceives two people, may talk to the near one and not the far one, and is
/// told why. A third person exists in the world and is not in the observation.
fn scene() -> Observation<String> {
    Observation::new(EntityId::from_raw(41), WorldTime::from_seconds(64_800))
        .at_location(at(0, 0).with_facing(Orientation::facing(Millidegrees::new(90_000))))
        .perceiving(vec![
            PerceivedEntity::new(EntityId::from_raw(42), EntityType::Person)
                .at(at(1_200, 1_600))
                .with_tags(Tags::new([Tag::new("staff").unwrap()]))
                .with_components(vec![named("Alice")]),
            PerceivedEntity::new(EntityId::from_raw(43), EntityType::Person)
                .at(at(9_000, 0))
                .with_components(vec![named("Bob")]),
        ])
        .offering(vec![
            Affordance::available(TALK, Some(EntityId::from_raw(42)), reach()),
            Affordance::unavailable(
                TALK,
                Some(EntityId::from_raw(43)),
                reach(),
                Rejection::TooFarAway,
            ),
        ])
}

/// `INV-13` as an assertion: the observation answers about what it lists and about nothing else.
/// The entity the world contains but did not expose is indistinguishable, from inside the
/// observation, from an entity that does not exist — and there is no second call that would return
/// more, because the value is the whole answer.
#[test]
fn an_observation_answers_only_about_the_entities_it_lists() {
    let observation = scene();

    assert_eq!(observation.entities().len(), 2);
    assert_eq!(
        observation.entity(EntityId::from_raw(42)).map(|e| e.id()),
        Some(EntityId::from_raw(42))
    );
    assert_eq!(
        observation.entity(EntityId::from_raw(43)).map(|e| e.id()),
        Some(EntityId::from_raw(43))
    );

    // Someone the world contains and this observer was not shown.
    let unperceived = EntityId::from_raw(44);
    assert!(observation.entity(unperceived).is_none());
    assert!(
        !observation
            .entities()
            .iter()
            .any(|entity| entity.id() == unperceived)
    );

    // An entity the observer perceives without being told everything about it: the exposure is
    // per-observation, so two observers of one entity can legitimately be shown different lists,
    // and a controller cannot tell a withheld component from an absent one.
    let bob = observation.entity(EntityId::from_raw(43)).unwrap();
    assert!(bob.tags().is_empty());
    assert_eq!(bob.components().len(), 1);
}

/// The interaction prompt a 3D client draws — `[E] Talk to Alice` — must be derivable from the
/// observation alone: the action type from the affordance, the target from the affordance, the name
/// from a component of that target in the same observation. If any of those had to be computed, the
/// client would be implementing a world rule, which `ENGINEERING_RULES.md` §8 forbids, and the 2D
/// client would have to implement the same rule again (§9).
#[test]
fn a_client_can_render_an_interaction_prompt_from_the_observation_alone() {
    let observation = scene();

    let offered = observation
        .affordances()
        .iter()
        .find(|affordance| affordance.is_available())
        .expect("the near person must be talkable to");
    assert_eq!(
        offered.action_type(),
        &ActionTypeId::new("talk").unwrap(),
        "the client maps this to its own wording; the kernel supplies none"
    );

    let target = offered.target().expect("this affordance names a target");
    let perceived = observation
        .entity(target)
        .expect("an affordance's target must be resolvable inside the same observation");
    let name: Named = serde_json::from_str(
        perceived.components()[0]
            .payload_for::<Named>()
            .expect("the name component is exposed to this observer"),
    )
    .unwrap();
    assert_eq!(name.name, "Alice");

    // The refused one carries the reason a player can be shown, and the requirement a client can
    // display without checking it.
    let refused = observation
        .affordances()
        .iter()
        .find(|affordance| !affordance.is_available())
        .expect("the far person must be offered and refused");
    assert_eq!(refused.unavailable_reason(), Some(&Rejection::TooFarAway));
    assert_eq!(
        refused.requirement().within_range(),
        Some(Millimetres::new(3_000))
    );

    // And the client did not decide any of that. The server's own evaluator, given the same
    // locations, reaches the same two answers — which is the point of there being one
    // implementation rather than one per client.
    let observer_location = observation.self_location().unwrap();
    assert_eq!(
        reach().evaluate(
            observer_location,
            observation.entity(target).unwrap().location(),
            true
        ),
        Ok(())
    );
    assert_eq!(
        reach().evaluate(
            observer_location,
            observation
                .entity(refused.target().unwrap())
                .unwrap()
                .location(),
            true
        ),
        Err(Rejection::TooFarAway)
    );
}

/// "Available, because too far away" must not be representable. A client shown both halves would
/// have to decide which to believe, and one shown neither has nothing to tell the player — so
/// neither construction nor deserialization may produce a disagreeing pair.
#[test]
fn an_affordance_cannot_disagree_with_itself_about_availability() {
    let available = serde_json::to_string(&Affordance::<String>::available(
        TALK,
        Some(EntityId::from_raw(42)),
        SpatialRequirement::NONE,
    ))
    .unwrap();
    assert_eq!(
        available,
        r#"{"action_type":"talk","target":"42","available":true,"unavailable_reason":null,"requirement":{"place":"any","within_range":null,"requires_line_of_access":false,"requires_target_available":false}}"#
    );
    assert!(serde_json::from_str::<Affordance>(&available).is_ok());

    let both = available.replace(
        r#""available":true,"unavailable_reason":null"#,
        r#""available":true,"unavailable_reason":"too_far_away""#,
    );
    let error = serde_json::from_str::<Affordance>(&both)
        .expect_err("available with a reason must be refused");
    assert_eq!(
        error.to_string(),
        ContractError::AffordanceAvailabilityDisagreement {
            available: true,
            has_reason: true,
        }
        .to_string()
    );

    let neither = available.replace(r#""available":true"#, r#""available":false"#);
    let error = serde_json::from_str::<Affordance>(&neither)
        .expect_err("unavailable with no reason must be refused");
    assert_eq!(
        error.to_string(),
        ContractError::AffordanceAvailabilityDisagreement {
            available: false,
            has_reason: false,
        }
        .to_string()
    );
}

// -------------------------------------------------------------------------------------------
// Complete affordances (`ARC-34`)
// -------------------------------------------------------------------------------------------

/// A request a test system would accept: one of a bounded set of choices, so the offering system can
/// state every complete request it would take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Ring {
    bell: String,
}

impl Action for Ring {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("ring");
    const OWNER: SystemId = SystemId::from_static("chimes-stub");
}

/// Another action, to show a request is labelled for exactly one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Knock {
    bell: String,
}

impl Action for Knock {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("knock");
    const OWNER: SystemId = SystemId::from_static("chimes-stub");
}

/// The base's literals, captured on `main @ da31613` before the field existed (step-10 §9.3 E-C0).
const AVAILABLE_ON_THE_BASE: &str = r#"{"action_type":"talk","target":"42","available":true,"unavailable_reason":null,"requirement":{"place":"any","within_range":null,"requires_line_of_access":false,"requires_target_available":false}}"#;
const UNAVAILABLE_ON_THE_BASE: &str = r#"{"action_type":"talk","target":"43","available":false,"unavailable_reason":"too_far_away","requirement":{"place":"same_place_as_actor","within_range":3000,"requires_line_of_access":false,"requires_target_available":false}}"#;
const UNTARGETED_ON_THE_BASE: &str = r#"{"action_type":"ring","target":null,"available":true,"unavailable_reason":null,"requirement":{"place":"any","within_range":null,"requires_line_of_access":false,"requires_target_available":false}}"#;

/// An affordance that is not complete crosses the wire exactly as it did before the field existed:
/// every observation, transcript and frame of a world whose packs offer nothing complete is unchanged.
#[test]
fn an_affordance_without_a_payload_is_the_shape_it_always_was() {
    let shapes = [
        (
            Affordance::<serde_json::Value>::available(
                TALK,
                Some(EntityId::from_raw(42)),
                SpatialRequirement::NONE,
            ),
            AVAILABLE_ON_THE_BASE,
        ),
        (
            Affordance::unavailable(
                TALK,
                Some(EntityId::from_raw(43)),
                reach(),
                Rejection::TooFarAway,
            ),
            UNAVAILABLE_ON_THE_BASE,
        ),
        (
            Affordance::available(Ring::ACTION_TYPE, None, SpatialRequirement::NONE),
            UNTARGETED_ON_THE_BASE,
        ),
    ];
    for (affordance, literal) in shapes {
        assert_eq!(serde_json::to_string(&affordance).unwrap(), literal);
        assert_eq!(affordance.payload(), None);
        assert_eq!(
            affordance.request(EntityId::from_raw(41), Clone::clone),
            None,
            "an affordance that is not complete names no request"
        );
    }
}

/// A frame written before the field existed decodes, with no payload; a complete affordance carries
/// its payload last and survives the transport; and the payload does not loosen the availability
/// check.
#[test]
fn an_old_frame_decodes_and_a_payload_round_trips() {
    let old: Affordance<serde_json::Value> =
        serde_json::from_str(UNAVAILABLE_ON_THE_BASE).expect("a frame without the field decodes");
    assert_eq!(old.payload(), None);
    assert_eq!(old.unavailable_reason(), Some(&Rejection::TooFarAway));

    let complete = Affordance::available(Ring::ACTION_TYPE, None, SpatialRequirement::NONE)
        .with_payload(serde_json::json!({ "bell": "low" }));
    let text = serde_json::to_string(&complete).unwrap();
    assert_eq!(
        text,
        format!(
            "{},\"payload\":{{\"bell\":\"low\"}}}}",
            UNTARGETED_ON_THE_BASE
                .strip_suffix('}')
                .expect("a JSON object")
        )
    );
    let back: Affordance<serde_json::Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(back, complete);
    assert_eq!(back.payload(), Some(&serde_json::json!({ "bell": "low" })));

    // Inside an observation too.
    let observation =
        Observation::new(EntityId::from_raw(41), WorldTime::EPOCH).offering(vec![complete.clone()]);
    let text = serde_json::to_string(&observation).unwrap();
    assert_eq!(
        serde_json::from_str::<Observation<serde_json::Value>>(&text).unwrap(),
        observation
    );

    // A payload does not make "available, because too far away" representable.
    let both = text.replace(
        r#""available":true,"unavailable_reason":null"#,
        r#""available":true,"unavailable_reason":"too_far_away""#,
    );
    assert!(serde_json::from_str::<Observation<serde_json::Value>>(&both).is_err());
}

/// A complete affordance names exactly one request: the affordance's own action type, its own target,
/// and its payload as the caller encodes it — which the owning system's type then decodes.
#[test]
fn a_complete_affordance_requests_exactly_what_it_offers() {
    let me = EntityId::from_raw(41);
    let bell = EntityId::from_raw(42);
    let offered = Affordance::available(Ring::ACTION_TYPE, Some(bell), SpatialRequirement::NONE)
        .with_payload(serde_json::json!({ "bell": "high" }));

    let request = offered
        .request(me, |payload| serde_json::to_vec(payload).unwrap())
        .expect("a complete affordance names a request");
    assert_eq!(request.actor(), me);
    assert_eq!(request.action_type(), &Ring::ACTION_TYPE);
    assert_eq!(request.payload().action_type(), &Ring::ACTION_TYPE);
    assert_eq!(request.target(), Some(bell));
    assert_eq!(request.actor_location(), None);
    let decoded: Ring =
        serde_json::from_slice(request.payload().payload_for::<Ring>().unwrap()).unwrap();
    assert_eq!(
        decoded,
        Ring {
            bell: "high".to_owned()
        }
    );
    assert!(
        request.payload().payload_for::<Knock>().is_err(),
        "the request is labelled for the action it was offered as, and no other"
    );

    // A well-formed request: it crosses the wire through the contract's own agreement check.
    let text = serde_json::to_string(&request).unwrap();
    assert_eq!(
        serde_json::from_str::<mineworld_contracts::ActionRequest>(&text).unwrap(),
        request
    );

    // Untargeted, and unavailable: the request is still the one named; whether to submit is the
    // requester's judgement and whether it is accepted is the server's.
    let elsewhere = Affordance::unavailable(
        Ring::ACTION_TYPE,
        None,
        SpatialRequirement::same_place(),
        Rejection::TooFarAway,
    )
    .with_payload(serde_json::json!({ "bell": "low" }));
    let request = elsewhere.request(me, Clone::clone).unwrap();
    assert_eq!(request.target(), None);
    assert_eq!(
        request.payload().payload_for::<Ring>().unwrap(),
        &serde_json::json!({ "bell": "low" })
    );
}

/// The stored shape of an observation, asserted exactly: this is what crosses the wire to a
/// controller or a client on every tick it is given one, so changing it is a protocol change.
#[test]
fn an_observation_is_stored_as_its_documented_shape() {
    let minimal = Observation::<String>::new(EntityId::from_raw(41), WorldTime::EPOCH);
    let text = r#"{"observer":"41","at":0,"self_location":null,"entities":[],"relations":[],"events":[],"affordances":[]}"#;
    assert_eq!(serde_json::to_string(&minimal).unwrap(), text);
    assert_eq!(
        serde_json::from_str::<Observation<String>>(text).unwrap(),
        minimal
    );

    // A world in which a person perceives nothing is a legitimate world, and it is also what an
    // observation is before anything is deliberately exposed.
    assert!(minimal.entities().is_empty());
    assert!(minimal.relations().is_empty());
    assert!(minimal.affordances().is_empty());
    assert_eq!(minimal.entity(EntityId::from_raw(42)), None);

    // The full scene survives the transport unchanged, affordances and reasons included.
    let scene = scene();
    let text = serde_json::to_string(&scene).unwrap();
    assert_eq!(
        serde_json::from_str::<Observation<String>>(&text).unwrap(),
        scene
    );
}

// -------------------------------------------------------------------------------------------
// Relations in an observation (`spike/FINDINGS.md` F3)
// -------------------------------------------------------------------------------------------

/// The place hierarchy and adjacency a client needs in order to render a *world* rather than a
/// room. Declared by an invented geography system, because relation types are a system's
/// declaration and never the kernel's.
fn contains() -> RelationTypeDeclaration {
    RelationTypeDeclaration::directed(
        RelationTypeId::new("contains").unwrap(),
        SystemId::new("geography-stub").unwrap(),
        EntityTypeSet::new([EntityType::Place]).unwrap(),
        EntityTypeSet::new([EntityType::Place]).unwrap(),
    )
}

fn adjoins() -> RelationTypeDeclaration {
    RelationTypeDeclaration::undirected(
        RelationTypeId::new("adjoins").unwrap(),
        SystemId::new("geography-stub").unwrap(),
        EntityTypeSet::new([EntityType::Place]).unwrap(),
    )
}

fn place(id: u64, key: &str) -> Entity {
    Entity::new(
        EntityId::from_raw(id),
        EntityKey::new(key).unwrap(),
        EntityType::Place,
    )
}

/// "Walk out of the café and along the promenade" — the project's north star
/// (`ENGINEERING_RULES.md` §1), and unrepresentable in an `Observation` until this field existed.
///
/// A place entity has no `Location` of its own, and place hierarchy is *defined* as a relation
/// rather than a field (`spatial.rs`), so relations were the one mechanism that could tell a client
/// the café contains a kitchen and adjoins the promenade — and the one mechanism an observation
/// could not deliver (`spike/FINDINGS.md` F3).
#[test]
fn an_observation_carries_the_place_structure_a_client_needs_to_render_a_world() {
    let cafe = place(7, "lakeside-cafe");
    let kitchen = place(8, "cafe-kitchen");
    let promenade = place(9, "lakeside-promenade");

    let inside =
        Relation::between(&contains(), &cafe, &kitchen).expect("a café contains a kitchen");
    let along = Relation::between(&adjoins(), &promenade, &cafe).expect("the two places adjoin");

    let observation = Observation::<String>::new(EntityId::from_raw(41), WorldTime::EPOCH)
        .perceiving(vec![PerceivedEntity::new(
            EntityId::from_raw(7),
            EntityType::Place,
        )])
        .relating(vec![inside.clone(), along.clone()]);

    assert_eq!(observation.relations().len(), 2);
    assert_eq!(observation.relations()[0], inside);
    assert_eq!(
        observation.relations()[0].relation_type(),
        &RelationTypeId::new("contains").unwrap()
    );
    assert_eq!(observation.relations()[0].from(), EntityId::from_raw(7));
    assert_eq!(observation.relations()[0].to(), EntityId::from_raw(8));

    // The undirected edge arrived canonically ordered, as `Relation::between` guarantees: the
    // observation transports edges and does not re-form them, so a client reading two observations
    // cannot see one edge two ways.
    assert_eq!(observation.relations()[1], along);
    assert_eq!(observation.relations()[1].from(), EntityId::from_raw(7));
    assert_eq!(observation.relations()[1].to(), EntityId::from_raw(9));

    // An edge may name an entity this observer was not shown. Being told that the café adjoins the
    // promenade is not the same as perceiving the promenade, and a client must not treat it as
    // something it can draw.
    assert!(observation.entity(EntityId::from_raw(9)).is_none());

    // And it crosses the wire, ids and all. Written by hand, including the id encoding.
    let text = serde_json::to_string(&observation).unwrap();
    assert!(
        text.contains(r#""relations":[{"relation_type":"contains","from":"7","to":"8"},{"relation_type":"adjoins","from":"7","to":"9"}]"#),
        "the relations must be on the wire in the documented shape, but the frame was: {text}"
    );
    assert_eq!(
        serde_json::from_str::<Observation<String>>(&text).unwrap(),
        observation
    );
}

/// Exposure is deliberate, never incidental: a fresh observation lists no relations, exactly as it
/// lists no entities. `INV-13` is preserved by the same construction — the field is a list of what
/// was chosen, and a perception system that put every edge in a world here would be making the same
/// mistake as one that listed every entity.
#[test]
fn an_observation_exposes_no_relations_until_one_is_deliberately_added() {
    let observation = Observation::<String>::new(EntityId::from_raw(41), WorldTime::EPOCH);
    assert!(observation.relations().is_empty());

    // The scene the rest of this file uses names no relations either, so nothing about perceiving
    // two people implies an edge between them.
    assert!(scene().relations().is_empty());
}
