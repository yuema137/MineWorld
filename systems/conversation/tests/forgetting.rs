//! FA-9 — `forget_after_days`: a listener's lines age out (`pr-fg-a-relationship-forgetting.md` §5.7;
//! `docs/DECISIONS.md` `ARC-86`).
//!
//! A scripted café: speaker `sam`, resident `rita` and commuter `carl`, all within reach. The world's
//! section says `{ target: resident, forget_after_days: 1 }`.
//!
//! ```text
//! t0              sam → rita, sam → carl
//! t0 + 1 d        rita is still shown sam's line (exactly one day old is not older than a day)
//! t0 + 1 d + 1 s  rita is shown nothing; her stored history still holds the line
//! t0 + 1 d + 2 s  sam → rita again: the stored history drops the old line
//! carl            a commuter keeps both lines; an unconfigured twin keeps everything
//! continuity      who is in a conversation, line by line, is the unconfigured twin's    M-FA9 → 1 d + 1 s
//! ```

use std::collections::BTreeMap;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, PersonId, PlaceId, Tag, Tags, Visibility,
    WorldTime,
};
use mineworld_conversation::{ConversationHistory, ConversationSystem, Talk, Utterance};
use mineworld_kernel::{Emission, World};
use mineworld_presence::{PerceptionProvider, PresenceSystem, arrival, observe};
use mineworld_sdk::interactions::{self, InteractionSection, Section, SectionConfigured};
use serde::Deserialize;

const DAY: i64 = 86_400;
const T0: i64 = 7_200;

const SECTION: &str = r#"{ "parameters": [ { "target": "resident", "forget_after_days": 1 } ] }"#;
const CLASSES: &str = r#"[ { "class": "resident", "of": "person", "tag": "resident" },
                           { "class": "commuter", "of": "person", "tag": "commuter" } ]"#;

struct Cafe {
    world: World,
    people: BTreeMap<&'static str, EntityId>,
    clock: i64,
    next: u64,
}

/// The configured fact, built as the loader builds it: decoded by the owner's type, resolved, encoded.
fn configured() -> Emission {
    let section: Section<ConversationSystem> =
        Section::deserialize(&mut serde_json::Deserializer::from_str(SECTION)).expect("decodes");
    let classes = serde_json::from_str(CLASSES).expect("classes decode");
    let resolved = interactions::resolve(&section, &classes).expect("resolves");
    Emission::new::<SectionConfigured<ConversationSystem>>(
        <ConversationSystem as InteractionSection>::encode(&resolved),
        Visibility::SystemInternal,
    )
}

impl Cafe {
    fn new(configure: bool) -> Self {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence");
        world.install(ConversationSystem).expect("conversation");
        let cafe = world
            .create_entity(EntityKey::new("cafe").expect("key"), EntityType::Place)
            .expect("created");
        let cafe = PlaceId::new(cafe, EntityType::Place).expect("a place");
        let mut people = BTreeMap::new();
        let mut facts = Vec::new();
        for (index, (name, tag)) in [
            ("sam", "resident"),
            ("rita", "resident"),
            ("carl", "commuter"),
        ]
        .into_iter()
        .enumerate()
        {
            let id = world
                .create_authored_entity(
                    EntityKey::new(name).expect("key"),
                    EntityType::Person,
                    Tags::new([Tag::new(tag).expect("a tag")]),
                    None,
                )
                .expect("created");
            people.insert(name, id);
            let spot = Location::in_place(cafe).with_local(LocalPosition::on_ground(
                Millimetres::new(1_000 + 500 * i32::try_from(index).expect("small")),
                Millimetres::new(1_000),
            ));
            let person = PersonId::new(id, EntityType::Person).expect("a person");
            facts.push(arrival(&world.read(), person, spot).expect("admitted"));
        }
        if configure {
            facts.push(configured());
        }
        world
            .genesis(WorldTime::from_seconds(T0 - 1), facts)
            .expect("begins");
        Self {
            world,
            people,
            clock: T0 - 1,
            next: 1,
        }
    }

    /// Moves the clock to `at`, firing whatever falls due.
    fn at(&mut self, at: i64) {
        self.clock = at;
        let _due = self
            .world
            .advance_to(WorldTime::from_seconds(at))
            .expect("advances");
    }

    /// `from` speaks to `to` at `at`; the facts it caused.
    fn talk(&mut self, from: &str, to: &str, at: i64, said: &str) -> Vec<EventEnvelope> {
        self.at(at);
        let record = ActionRecord::new::<Talk>(
            serde_json::to_vec(&Talk::new(Utterance::new(said).expect("an utterance")))
                .expect("encodes"),
        );
        let when = WorldTime::from_seconds(at);
        let intent = ActionIntent::new(
            ActionId::from_raw(self.next),
            self.people[from],
            record,
            when,
        )
        .with_target(self.people[to]);
        self.next += 1;
        let dispatched = self.world.dispatch(&intent, when).expect("answers");
        assert!(matches!(dispatched.result(), ActionResult::Accepted { .. }));
        dispatched.events().to_vec()
    }

    /// What `who` is shown of their own history now: the lines' text, oldest first.
    fn shown(&self, who: &str) -> Vec<String> {
        let id = self.people[who];
        let providers: [&dyn PerceptionProvider; 2] = [&PresenceSystem, &ConversationSystem];
        observe(
            &self.world,
            id,
            WorldTime::from_seconds(self.clock),
            &providers,
        )
        .entity(id)
        .expect("themself")
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "conversation-history")
        .map(|record| {
            record.payload()["heard"]
                .as_array()
                .expect("a list")
                .iter()
                .map(|heard| heard["utterance"].as_str().expect("text").to_owned())
                .collect()
        })
        .unwrap_or_default()
    }

    /// What `who`'s stored history holds: the lines' text, oldest first.
    fn stored(&self, who: &str) -> Vec<String> {
        self.world
            .read()
            .component::<ConversationHistory>(self.people[who])
            .map(|history| {
                history
                    .heard()
                    .iter()
                    .map(|heard| heard.utterance().as_str().to_owned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn started(facts: &[EventEnvelope]) -> bool {
    facts
        .iter()
        .any(|fact| fact.event_type().as_str() == "conversation-started")
}

/// The script, on a configured or an unconfigured café: what each check saw, and whether each line
/// started a conversation.
#[test]
fn a_residents_lines_age_out_after_a_day_and_nothing_else_changes() {
    let mut cafe = Cafe::new(true);
    let mut twin = Cafe::new(false);
    let mut continuity = (Vec::new(), Vec::new());
    for (cafe, starts) in [
        (&mut cafe, &mut continuity.0),
        (&mut twin, &mut continuity.1),
    ] {
        starts.push(started(&cafe.talk("sam", "rita", T0, "first")));
        starts.push(started(&cafe.talk("sam", "carl", T0, "to carl")));
        starts.push(started(&cafe.talk("rita", "sam", T0 + 60, "a reply")));
    }

    cafe.at(T0 + DAY);
    assert_eq!(cafe.shown("rita"), ["first"], "exactly a day old is kept");
    cafe.at(T0 + DAY + 1);
    assert_eq!(
        cafe.shown("rita"),
        Vec::<String>::new(),
        "older than a day: never shown"
    );
    assert_eq!(
        cafe.stored("rita"),
        ["first"],
        "still stored until rita hears something"
    );
    assert_eq!(
        cafe.shown("carl"),
        ["to carl"],
        "a commuter forgets nothing"
    );

    twin.at(T0 + DAY + 1);
    assert_eq!(twin.shown("rita"), ["first"], "unconfigured: kept");

    for (cafe, starts) in [
        (&mut cafe, &mut continuity.0),
        (&mut twin, &mut continuity.1),
    ] {
        starts.push(started(&cafe.talk("sam", "rita", T0 + DAY + 2, "second")));
        starts.push(started(&cafe.talk("sam", "carl", T0 + DAY + 3, "again")));
    }
    assert_eq!(
        cafe.stored("rita"),
        ["second"],
        "the next line drops the old one"
    );
    assert_eq!(cafe.shown("rita"), ["second"]);
    assert_eq!(cafe.stored("carl"), ["to carl", "again"]);
    assert_eq!(twin.stored("rita"), ["first", "second"]);
    // Who is in a conversation is decided exactly as without the section: the reply continued the
    // first; a day later both greetings started new ones.
    assert_eq!(continuity.0, continuity.1);
    assert_eq!(continuity.0, [true, true, false, true, true]);
}
