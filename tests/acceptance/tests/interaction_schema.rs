//! IB-5, IB-7 (the schema's half), IB-10 — the World's Interaction List proven end to end with
//! test-only packs (step-18-interaction-list §12.5; `docs/DECISIONS.md` `ARC-63` … `ARC-65`).
//!
//! `test-tuning` (`configuration/mod.rs`) is configured through the seam exactly as the loader does it —
//! its section decoded by `decode_configuration` (which `interactions!()` defines), checked against the
//! world's classes, seeded — and then driven through `World::dispatch` and presence's `observe`:
//!
//! ```text
//! IB-5 a  a forbid for class A against class B: the dispatch is refused PermissionDenied and the
//!         offer shown unavailable for that reason, its requirement still shown; another pair is not
//!      b  default: forbid with one permit: only the permitted pair is accepted
//!      c  a scoped parameter changes `advanced.by` only for its class; a region only in its place
//!      d  audience: participants → the envelope's Visibility is Participants; public is refused
//!      e  biography: off for class A → the selection drops A's facts and keeps B's; both are in the log
//! IB-7    each decode-time refusal at its line, and each context refusal named with list and index
//! IB-10   a data: attachment's rows reach the owner's fact, with either line ending
//! IB-C6   with nothing configured, every lookup answers the compiled default
//! ```

mod configuration;

use std::collections::{BTreeMap, BTreeSet};

use configuration::{
    ADVANCE, ADVANCED, Advance, Advanced, Setup, Table, TableConfigured, Tuning, genesis,
    world_with,
};
use mineworld_authoring::{Attached, Attachment, ConfigurationContext, EntityClasses, Seeding};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, PlaceId, Rejection, Visibility, WorldTime,
};
use mineworld_kernel::World;
use mineworld_sdk::SystemPack;
use mineworld_sdk::interactions::{self, Configured, Known, Role, Roles};

const CLASSES: &str = "- { class: noble, of: person, tag: noble }\n\
                       - { class: servant, of: person, tag: servant }\n";

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a key")
}

/// A configured world: the square and the porch; ada (noble), bo (servant) and cy (untagged) in the
/// square, dee (noble) on the porch, each starting at 0.
struct Yard {
    world: World,
    ids: BTreeMap<EntityKey, EntityId>,
    genesis: Vec<EventEnvelope>,
    next: u64,
}

impl Yard {
    fn new(section: &str) -> Self {
        Self::try_new(section).expect("the section is accepted")
    }

    fn try_new(section: &str) -> Result<Self, String> {
        let (mut world, ids) = world_with(
            &[("square", &[]), ("porch", &[])],
            &[
                ("ada", &["noble"]),
                ("bo", &["servant"]),
                ("cy", &[]),
                ("dee", &["noble"]),
            ],
        );
        // Presence places each person, so perception has someone to offer `advance` against; its
        // arrivals are stated first, as the loader states locations before configuration.
        world
            .install(mineworld_presence::PresenceSystem)
            .expect("presence installs");
        let mut facts = Vec::new();
        for (person, place) in [
            ("ada", "square"),
            ("bo", "square"),
            ("cy", "square"),
            ("dee", "porch"),
        ] {
            let place = PlaceId::new(ids[&key(place)], EntityType::Place).expect("a place");
            facts.push(
                mineworld_presence::arrival(
                    &world.read(),
                    mineworld_contracts::PersonId::new(ids[&key(person)], EntityType::Person)
                        .expect("a person"),
                    mineworld_contracts::Location::in_place(place),
                )
                .map_err(|error| format!("{error:?}"))?,
            );
        }
        facts.extend(genesis(
            &world,
            &ids,
            &Setup {
                classes: Some(CLASSES),
                section: Some(section),
                starts: vec![
                    ("ada", 0, "square"),
                    ("bo", 0, "square"),
                    ("cy", 0, "square"),
                    ("dee", 0, "porch"),
                ],
                sections_first: false,
            },
        )?);
        let genesis = world
            .genesis(WorldTime::EPOCH, facts)
            .map_err(|error| format!("{error:?}"))?;
        Ok(Self {
            world,
            ids,
            genesis,
            next: 0,
        })
    }

    fn id(&self, key_: &str) -> EntityId {
        self.ids[&key(key_)]
    }

    /// `actor` advances at `target`: the answer, and the `advanced` fact if accepted.
    fn advance(&mut self, actor: &str, target: &str) -> (ActionResult, Option<EventEnvelope>) {
        self.next += 1;
        let intent = ActionIntent::new(
            ActionId::from_raw(self.next),
            self.id(actor),
            ActionRecord::new::<Advance>(serde_json::to_vec(&Advance {}).expect("encodes")),
            WorldTime::from_seconds(i64::try_from(self.next).expect("small")),
        )
        .with_target(self.id(target));
        let at = WorldTime::from_seconds(i64::try_from(self.next).expect("small"));
        let dispatched = self.world.dispatch(&intent, at).expect("answered");
        let fact = dispatched
            .events()
            .iter()
            .find(|fact| *fact.event_type() == ADVANCED)
            .cloned();
        (dispatched.result().clone(), fact)
    }

    fn by(&mut self, actor: &str, target: &str) -> u32 {
        let (_, fact) = self.advance(actor, target);
        let fact = fact.expect("accepted");
        let payload = fact.payload().payload_for::<Advanced>().expect("advanced");
        serde_json::from_slice::<Advanced>(payload)
            .expect("decodes")
            .by
    }

    /// What `observer` is offered at `target`.
    fn offered(
        &self,
        observer: &str,
        target: &str,
    ) -> mineworld_contracts::Affordance<serde_json::Value> {
        let observation = mineworld_presence::observe(
            &self.world,
            self.id(observer),
            WorldTime::EPOCH,
            &[
                &mineworld_presence::PresenceSystem as &dyn mineworld_presence::PerceptionProvider,
                &Tuning,
            ],
        );
        let target = self.id(target);
        observation
            .affordances()
            .iter()
            .find(|affordance| {
                *affordance.action_type() == ADVANCE && affordance.target() == Some(target)
            })
            .cloned()
            .expect("advance is offered")
    }
}

// ---- IB-5 ------------------------------------------------------------------------------------

/// (a) M-IB5a (permit wins ties) breaks the first assertion; M-IB5b (the offer skips `permits`) breaks
/// the affordance's.
#[test]
fn a_forbidden_pair_is_refused_at_dispatch_and_shown_unavailable_for_that_reason() {
    let mut yard = Yard::new(
        "rules:\n\
         \x20 - { action: advance, actor: noble, target: servant, effect: permit }\n\
         \x20 - { action: advance, actor: noble, target: servant, effect: forbid }\n",
    );
    assert_eq!(
        yard.advance("ada", "bo").0,
        ActionResult::Rejected(Rejection::PermissionDenied),
        "forbid overrides permit at equal specificity"
    );
    let offered = yard.offered("ada", "bo");
    assert!(!offered.is_available());
    assert_eq!(
        offered.unavailable_reason(),
        Some(&Rejection::PermissionDenied)
    );
    assert_eq!(
        *offered.requirement(),
        mineworld_contracts::SpatialRequirement::NONE,
        "the requirement is still shown"
    );
    assert!(matches!(
        yard.advance("ada", "cy").0,
        ActionResult::Accepted { .. }
    ));
    assert!(yard.offered("ada", "cy").is_available());
    assert!(matches!(
        yard.advance("bo", "ada").0,
        ActionResult::Accepted { .. }
    ));
}

/// (b) `default: forbid` with one permit: only the permitted pair is accepted.
#[test]
fn a_forbidding_default_admits_only_what_a_rule_permits() {
    let mut yard = Yard::new(
        "default: forbid\n\
         rules:\n\
         \x20 - { action: advance, actor: servant, target: noble, effect: permit }\n",
    );
    assert!(matches!(
        yard.advance("bo", "ada").0,
        ActionResult::Accepted { .. }
    ));
    for (actor, target) in [("ada", "bo"), ("cy", "ada"), ("bo", "cy")] {
        assert_eq!(
            yard.advance(actor, target).0,
            ActionResult::Rejected(Rejection::PermissionDenied),
            "{actor} → {target}"
        );
    }
}

/// (c) A scoped parameter applies only to its class; a region only at its place.
#[test]
fn a_scoped_parameter_applies_to_its_class_and_a_region_to_its_place() {
    let mut yard = Yard::new(
        "parameters:\n\
         \x20 - { step: 2 }\n\
         \x20 - { actor: noble, step: 7 }\n\
         regions:\n\
         \x20 porch:\n\
         \x20   parameters: [ { actor: noble, step: 11 } ]\n",
    );
    assert_eq!(yard.by("ada", "cy"), 7, "a noble in the square");
    assert_eq!(yard.by("bo", "cy"), 2, "a servant: the base");
    assert_eq!(yard.by("cy", "bo"), 2, "nobody's class: the base");
    assert_eq!(yard.by("dee", "cy"), 11, "a noble on the porch: the region");
}

/// (d) The list narrows the fact's audience (owner default Place) to Participants; widening is refused.
#[test]
fn a_narrowed_audience_reaches_the_envelope_and_a_widening_is_refused_at_load() {
    let mut plain = Yard::new("parameters: [ { step: 1 } ]\n");
    let (_, fact) = plain.advance("ada", "bo");
    assert!(matches!(
        fact.expect("accepted").visibility(),
        Visibility::Place(_)
    ));

    let mut narrowed =
        Yard::new("consequences:\n  - { fact: advanced, actor: noble, audience: participants }\n");
    let (_, fact) = narrowed.advance("ada", "bo");
    assert_eq!(
        *fact.expect("accepted").visibility(),
        Visibility::Participants
    );
    let (_, fact) = narrowed.advance("bo", "ada");
    assert!(
        matches!(fact.expect("accepted").visibility(), Visibility::Place(_)),
        "only the noble's"
    );

    let widened = Yard::try_new("consequences:\n  - { fact: advanced, audience: public }\n")
        .err()
        .expect("widening is refused");
    assert!(
        widened.contains("line 2") && widened.contains("may only narrow"),
        "{widened}"
    );
}

/// (e) `biography: off` for nobles: the selection drops a noble's `advanced` and keeps a servant's;
/// the log holds both. M-IB5c (the selection ignores `Configured`) breaks the first assertion.
#[test]
fn biography_off_for_a_class_drops_its_facts_from_the_selection_and_not_from_the_log() {
    let mut yard =
        Yard::new("consequences:\n  - { fact: advanced, actor: noble, biography: off }\n");
    let (_, noble) = yard.advance("ada", "bo");
    let (_, servant) = yard.advance("bo", "ada");
    let (noble, servant) = (noble.expect("in the log"), servant.expect("in the log"));

    let table = yard
        .genesis
        .iter()
        .find(|fact| *fact.event_type() == <Tuning as interactions::InteractionSection>::CONFIGURED)
        .map(|fact| interactions::consequences::<Tuning>(fact.payload().payload()))
        .expect("the configured fact")
        .expect("decodes");
    let read = yard.world.read();
    let entities: BTreeMap<EntityId, Known> = read
        .entities()
        .map(|entity| {
            (
                entity.id(),
                Known {
                    key: entity.key().clone(),
                    entity_type: entity.entity_type(),
                    tags: entity.tags().clone(),
                },
            )
        })
        .collect();
    let decl = Tuning::INTERACTIONS.expect("a section");
    let configured = Configured::none()
        .with_entities(entities)
        .with_section(decl.facts, &table);
    let compiled: BTreeSet<_> = Tuning::BIOGRAPHICAL.iter().cloned().collect();
    let (ada, bo) = (yard.id("ada"), yard.id("bo"));

    assert!(!interactions::biography::selected(
        &noble,
        ada,
        &compiled,
        &configured
    ));
    assert!(!interactions::biography::selected(
        &noble,
        bo,
        &compiled,
        &configured
    ));
    assert!(interactions::biography::selected(
        &servant,
        bo,
        &compiled,
        &configured
    ));
    assert!(interactions::biography::selected(
        &servant,
        ada,
        &compiled,
        &configured
    ));
    assert!(
        interactions::biography::selected(&noble, ada, &compiled, &Configured::none()),
        "with nothing configured, ARC-29 exactly"
    );
}

// ---- IB-7 (the schema's half) ----------------------------------------------------------------

/// Each refusal decidable from the pack's declarations is raised at its line; each needing the classes
/// names its list and index.
#[test]
fn every_section_refusal_is_by_name_with_its_position() {
    for (section, line, why) in [
        (
            "rules:\n  - { action: jump, effect: forbid }\n",
            "line 2",
            "'jump' is not an action",
        ),
        (
            "rules:\n  - { action: advance, object: noble, effect: forbid }\n",
            "line 2",
            "role 'object' is not declared",
        ),
        (
            "consequences:\n  - { fact: tuned, biography: off }\n",
            "line 2",
            "'tuned' is not a fact",
        ),
        (
            "parameters:\n  - { step: 101 }\n",
            "line 2",
            "'step' is 1 … 100, not 101",
        ),
        (
            "parameters:\n  - { pace: 3 }\n",
            "line 2",
            "'pace' is neither a role nor a parameter",
        ),
        (
            "consequences:\n  - { fact: advanced, audience: public }\n",
            "line 2",
            "may only narrow",
        ),
        ("extends: nowhere\n", "line 1", "names no reference list"),
        ("extends: loop-a\n", "line 1", "cycles"),
        ("extends: deep-1\n", "line 1", "longer than 4"),
        ("ruels: []\n", "line 1", "'ruels' is not a key of a section"),
    ] {
        let refusal = Yard::try_new(section)
            .err()
            .unwrap_or_else(|| panic!("{section:?} accepted"));
        assert!(
            refusal.contains(line) && refusal.contains(why),
            "{section:?}: {refusal}"
        );
    }

    let undefined = Yard::try_new("parameters:\n  - { actor: knight, step: 2 }\n")
        .err()
        .expect("undefined");
    assert!(
        undefined.contains("ClassUndefined")
            && undefined.contains("knight")
            && undefined.contains("\"parameters\"")
            && undefined.contains("index: 0"),
        "{undefined}"
    );
    let ambiguous = Yard::try_new(
        "parameters:\n  - { actor: noble, step: 2 }\n  - { actor: person, step: 3 }\n",
    )
    .err()
    .expect("ambiguous");
    assert!(
        ambiguous.contains("Ambiguous")
            && ambiguous.contains("index: 0")
            && ambiguous.contains("index: 1")
            && ambiguous.contains("step"),
        "{ambiguous}"
    );

    let yard = Yard::new("extends: gentle\n");
    let square = PlaceId::new(yard.id("square"), EntityType::Place).expect("a place");
    assert_eq!(
        interactions::parameters::<Tuning>(&yard.world.read(), square, &Roles::new()).step,
        2,
        "a reference list a section extends"
    );
}

// ---- IB-C6: nothing configured ---------------------------------------------------------------

/// With no section, every lookup answers the compiled default.
#[test]
fn with_nothing_configured_every_lookup_answers_the_compiled_default() {
    let (world, ids) = world_with(&[("square", &[])], &[("ada", &["noble"]), ("bo", &[])]);
    let read = world.read();
    let square = PlaceId::new(ids[&key("square")], EntityType::Place).expect("a place");
    let roles = Roles::new()
        .with(Role::Actor, ids[&key("ada")])
        .with(Role::Target, ids[&key("bo")]);
    assert_eq!(
        interactions::permits::<Tuning>(&read, square, &ADVANCE, &roles),
        Ok(())
    );
    assert_eq!(
        interactions::parameters::<Tuning>(&read, square, &roles).step,
        1
    );
    let routed = interactions::consequence::<Tuning>(
        &read,
        Some(square),
        &ADVANCED,
        &roles,
        Visibility::Place(square),
    );
    assert_eq!(routed.visibility, Visibility::Place(square));
    assert!(routed.biographical);
}

// ---- IB-10 -----------------------------------------------------------------------------------

/// A `data:` attachment's bytes reach the owner, which states the rows it decoded — whatever the line
/// ending. M-IB10 (seed handed empty bytes) breaks the rows assertion.
#[test]
fn an_attachments_rows_reach_the_owners_fact_with_either_line_ending() {
    let world = World::new();
    let read = world.read();
    let keys = BTreeMap::new();
    let classes = EntityClasses::default();
    for text in ["1,2,3\n4,5,6\n", "1,2,3\r\n4,5,6\r\n"] {
        let configuration =
            serde_saphyr::with_deserializer_from_str("table: data/table.csv\n", |file| {
                Table::decode_configuration(file)
            })
            .expect("decodes");
        let named: Vec<String> = configuration
            .attachments()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(named, ["data/table.csv"]);
        let mut attached = Attached::none();
        attached.insert(
            Attachment::try_from("data/table.csv".to_owned()).expect("a path"),
            text.as_bytes().to_vec(),
        );
        let facts = configuration
            .seed(
                &Seeding::new(&read, &keys),
                &ConfigurationContext::new(&classes, &attached),
            )
            .expect("seeds");
        let payload = facts[0]
            .record()
            .payload_for::<TableConfigured>()
            .expect("table-configured");
        assert_eq!(
            serde_json::from_slice::<TableConfigured>(payload).expect("decodes"),
            TableConfigured {
                rows: vec![[1, 2, 3], [4, 5, 6]]
            }
        );
    }
}
