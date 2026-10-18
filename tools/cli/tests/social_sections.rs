//! IE-3, IE-5's scripted half, IE-8 and IE-9 — the social packs' sections, scripted through the real
//! loader and `World::dispatch`, with every affordance read from perception's real `observe`
//! (`pr-il-e-social.md` §6; `docs/DECISIONS.md` `ARC-63`, `ARC-65`).
//!
//! Scratch copies of `worlds/social-cafe`, made at test time. Classes `resident` (people tagged
//! `resident`) and `commuter` (tagged `commuter`). Four people are placed in the café for the script:
//!
//! ```text
//! carol  resident  (3000, 3000)     grace  commuter  (3000, 5000)   2.0 m from carol
//! otto   resident  (4000, 5000)     hana   commuter  (3000, 7000)   4.0 m from carol, 2.0 from grace
//! dev, erin (residents) stay in the park, 1.6 m apart; bob (no class) stays at the counter
//! ```
//!
//! ```text
//! IE-3 (a) talk resident → commuter forbidden: refused PermissionDenied even out of range; the
//!          affordance is unavailable for that reason with its requirement shown; the reverse is
//!          accepted                                          M-IE3a (offers skip permits),
//!                                                            M-IE3b (validate skips permits)
//!      (b) invite, accept-invitation, join-group-activity resident → commuter forbidden: each refused
//!          at dispatch and in its offer; the accept path cannot bypass the join rule   M-IE3d
//!      (c) a rule about decline-invitation, among the load refusals below
//!      (d) range 6000 for actor resident: a 4 m talk accepted for the resident, TooFarAway for the
//!          commuter; each affordance carries its range       M-IE3c (validate uses the constant)
//!      (e) a café region forbidding talk: refused in the café, accepted in the park
//! IE-5     acquaint resident → commuter forbidden: no entry, no edge, no fact for that direction;
//!          the reverse forms                                 M-IE5 (relationships ignores the rule)
//! IE-8     remember off for target commuter; remembered 2 for target resident   M-IE8
//! IE-9     activity_length 600 in the park region; started narrowed; joined's biography off for a
//!          resident                                          M-IE9 (begin uses ACTIVITY_LENGTH)
//! SD-IE-8  no installed pack provides an action type `acquaint`
//! ```

mod headless;

use std::collections::BTreeMap;
use std::path::Path;

use headless::{PACK, Scratch, fresh};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionResult, Affordance, EntityId, EntityKey,
    EventEnvelope, Millimetres, Rejection, Visibility, WorldTime,
};
use mineworld_conversation::{
    ConversationHistory, ConversationSystem, Talk, Utterance, talk_requirement,
    talk_requirement_within,
};
use mineworld_group_activity::{
    AcceptInvitation, ActivityKind, GroupActivitySystem, Invite, JoinGroupActivity,
};
use mineworld_presence::{PerceptionProvider, PresenceSystem};
use mineworld_relationships::{Acquaintances, knows};
use mineworld_worldpack::WorldPack;
use mineworld_worldpack::interactions::InteractionSection;
use mineworld_worldpack::interactions::biography::{self, Configured, Known};
use mineworld_worldpack::load::LoadedWorld;
use serde_json::json;

/// An action as a client frames it: its payload as the JSON a hand-built client sends.
trait Wire: Action {
    fn wire(&self) -> serde_json::Value;
}

impl Wire for Talk {
    fn wire(&self) -> serde_json::Value {
        json!({ "utterance": self.utterance().as_str() })
    }
}

impl Wire for Invite {
    fn wire(&self) -> serde_json::Value {
        json!({ "kind": self.kind().as_str() })
    }
}

impl Wire for AcceptInvitation {
    fn wire(&self) -> serde_json::Value {
        json!({})
    }
}

impl Wire for JoinGroupActivity {
    fn wire(&self) -> serde_json::Value {
        json!({})
    }
}

const CLASSES: &str = "- { class: resident, of: person, tag: resident }\n\
                       - { class: commuter, of: person, tag: commuter }\n";

/// A person placed in the café for the script, keeping their tag.
fn person(name: &str, tag: &str, x: i32, y: i32) -> String {
    format!(
        "name: {name}\ntags:\n  - {tag}\nlocation:\n  place: cafe\n  position:\n    x: {x}\n    y: {y}\n"
    )
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

/// A scratch social-cafe with the script's people, `classes` and these sections configured.
fn copy(name: &str, sections: &[(&str, &str)]) -> Scratch {
    let scratch = fresh(name).within("social-cafe");
    copy_dir(Path::new(PACK), &scratch);
    let mut keys = vec!["classes"];
    keys.extend(sections.iter().map(|(key, _)| *key));
    let manifest = scratch.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("reads");
    let listed: String = keys.iter().map(|key| format!("  - {key}\n")).collect();
    std::fs::write(&manifest, format!("{text}\nconfigure:\n{listed}")).expect("writes");
    std::fs::create_dir_all(scratch.join("configure")).expect("writable");
    std::fs::write(scratch.join("configure/classes.yaml"), CLASSES).expect("writes");
    for (key, text) in sections {
        std::fs::write(scratch.join(format!("configure/{key}.yaml")), text).expect("writes");
    }
    for (file, text) in [
        ("carol", person("Carol Mensah", "resident", 3000, 3000)),
        ("grace", person("Grace Liu", "commuter", 3000, 5000)),
        ("otto", person("Otto Brandt", "resident", 4000, 5000)),
        ("hana", person("Hana Sato", "commuter", 3000, 7000)),
    ] {
        std::fs::write(scratch.join(format!("people/{file}.yaml")), text).expect("writes");
    }
    scratch
}

/// The script's world: loaded, with a clock that only moves forward.
struct Town {
    _scratch: Scratch,
    world: LoadedWorld,
    at: i64,
    next: u64,
}

impl Town {
    fn new(name: &str, sections: &[(&str, &str)]) -> Self {
        let scratch = copy(name, sections);
        let world = WorldPack::read(&scratch)
            .expect("the configured copy reads")
            .load(WorldTime::EPOCH)
            .expect("and loads");
        Self {
            _scratch: scratch,
            world,
            at: 10,
            next: 1,
        }
    }

    fn id(&self, key: &str) -> EntityId {
        self.world
            .id(&EntityKey::new(key).expect("a key"))
            .expect("declared")
    }

    /// Moves the clock on by `seconds`, returning everything that happened on the way.
    fn wait(&mut self, seconds: i64) -> Vec<EventEnvelope> {
        self.at += seconds;
        self.world
            .world_mut()
            .advance_to(WorldTime::from_seconds(self.at))
            .expect("advances")
            .into_events()
    }

    /// `actor` asks `action` of `target`, ten seconds after the last request.
    fn act<A: Wire>(
        &mut self,
        actor: &str,
        target: &str,
        action: &A,
    ) -> (ActionResult, Vec<EventEnvelope>) {
        let (actor, target) = (self.id(actor), self.id(target));
        self.wait(10);
        let at = WorldTime::from_seconds(self.at);
        let intent = ActionIntent::new(
            ActionId::from_raw(self.next),
            actor,
            ActionRecord::new::<A>(serde_json::to_vec(&action.wire()).expect("encodes")),
            at,
        )
        .with_target(target);
        self.next += 1;
        let dispatched = self
            .world
            .world_mut()
            .dispatch(&intent, at)
            .expect("answered");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    fn talk(&mut self, speaker: &str, listener: &str) -> (ActionResult, Vec<EventEnvelope>) {
        let said = Talk::new(Utterance::new(format!("line {}", self.next)).expect("an utterance"));
        self.act(speaker, listener, &said)
    }

    /// What `observer` is offered of `action` against `target`, through perception's `observe`.
    fn offered(&self, observer: &str, target: &str, action: &str) -> Affordance<serde_json::Value> {
        let providers: [&dyn PerceptionProvider; 3] =
            [&PresenceSystem, &ConversationSystem, &GroupActivitySystem];
        let target = self.id(target);
        mineworld_presence::observe(
            self.world.world(),
            self.id(observer),
            WorldTime::from_seconds(self.at),
            &providers,
        )
        .affordances()
        .iter()
        .find(|affordance| {
            affordance.action_type().as_str() == action && affordance.target() == Some(target)
        })
        .cloned()
        .unwrap_or_else(|| panic!("{action} is offered against {target:?}"))
    }

    fn history(&self, of: &str) -> Vec<String> {
        self.world
            .world()
            .read()
            .component::<ConversationHistory>(self.id(of))
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

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

fn denied() -> ActionResult {
    ActionResult::Rejected(Rejection::PermissionDenied)
}

fn of_type<'a>(facts: &'a [EventEnvelope], kind: &str) -> Vec<&'a EventEnvelope> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == kind)
        .collect()
}

fn coffee() -> Invite {
    Invite::new(ActivityKind::new("coffee").expect("a kind"))
}

fn assert_refused_offer(offer: &Affordance<serde_json::Value>, what: &str) {
    assert!(!offer.is_available(), "{what}: unavailable");
    assert_eq!(
        offer.unavailable_reason(),
        Some(&Rejection::PermissionDenied),
        "{what}: for the list's reason"
    );
}

// ---- IE-3 ---------------------------------------------------------------------------------------

#[test]
fn a_forbidden_talk_is_refused_before_distance_and_its_offer_says_why() {
    let mut town = Town::new(
        "ie3a-talk",
        &[(
            "conversation",
            "rules:\n  - { action: talk, actor: resident, target: commuter, effect: forbid }\n",
        )],
    );
    assert_eq!(
        town.talk("carol", "hana").0,
        denied(),
        "4 m away and forbidden: the list answers first"
    );
    assert_eq!(town.talk("carol", "grace").0, denied(), "2 m away");
    let offer = town.offered("carol", "grace", "talk");
    assert_refused_offer(&offer, "carol's talk at grace");
    assert_eq!(
        *offer.requirement(),
        talk_requirement(),
        "the requirement is still shown"
    );
    assert!(
        accepted(&town.talk("grace", "carol").0),
        "no rule the other way"
    );
    assert!(town.offered("grace", "carol", "talk").is_available());
}

#[test]
fn a_forbidden_invite_accept_or_join_is_refused_and_accepting_cannot_bypass_joining() {
    let rules = "rules:\n\
                 \x20 - { action: invite, actor: resident, target: commuter, effect: forbid }\n\
                 \x20 - { action: accept-invitation, actor: resident, target: commuter, effect: forbid }\n\
                 \x20 - { action: join-group-activity, actor: resident, target: commuter, effect: forbid }\n";
    let mut town = Town::new("ie3b-group", &[("group-activity", rules)]);
    assert_eq!(town.act("carol", "grace", &coffee()).0, denied());
    assert_refused_offer(&town.offered("carol", "grace", "invite"), "carol's invite");

    // Two commuters start something; then a commuter invites a resident, which no rule forbids.
    assert!(accepted(&town.act("hana", "grace", &coffee()).0));
    let (result, started) = town.act("grace", "hana", &AcceptInvitation::default());
    assert!(accepted(&result), "{result:?}");
    assert_eq!(of_type(&started, "group-activity-started").len(), 1);
    assert!(accepted(&town.act("grace", "carol", &coffee()).0));

    // Accepting would join grace's activity: refused, and so is joining it directly.
    assert_eq!(
        town.act("carol", "grace", &AcceptInvitation::default()).0,
        denied()
    );
    assert_refused_offer(
        &town.offered("carol", "grace", "accept-invitation"),
        "carol's accept",
    );
    assert_eq!(
        town.act("carol", "grace", &JoinGroupActivity::default()).0,
        denied()
    );
    assert_refused_offer(
        &town.offered("carol", "grace", "join-group-activity"),
        "carol's join",
    );
    // A person of no class joins freely: the rules name only residents.
    assert!(accepted(
        &town.act("bob", "grace", &JoinGroupActivity::default()).0
    ));
}

#[test]
fn a_scoped_range_is_the_one_validated_and_the_one_offered() {
    let mut town = Town::new(
        "ie3d-range",
        &[(
            "conversation",
            "parameters:\n  - { actor: resident, range: 6000 }\n",
        )],
    );
    assert!(
        accepted(&town.talk("carol", "hana").0),
        "a resident's voice reaches 6 m"
    );
    assert_eq!(
        town.talk("hana", "carol").0,
        ActionResult::Rejected(Rejection::TooFarAway),
        "a commuter's reaches the compiled 3 m"
    );
    assert_eq!(
        *town.offered("carol", "hana", "talk").requirement(),
        talk_requirement_within(Millimetres::new(6_000))
    );
    assert!(town.offered("carol", "hana", "talk").is_available());
    assert_eq!(
        *town.offered("hana", "carol", "talk").requirement(),
        talk_requirement()
    );
    assert!(!town.offered("hana", "carol", "talk").is_available());
}

#[test]
fn a_region_forbids_talk_in_its_place_only() {
    let mut town = Town::new(
        "ie3e-region",
        &[(
            "conversation",
            "regions:\n  cafe:\n    rules: [ { action: talk, effect: forbid } ]\n",
        )],
    );
    assert_eq!(town.talk("carol", "grace").0, denied(), "in the café");
    assert!(accepted(&town.talk("dev", "erin").0), "in the park");
}

// ---- IE-5, scripted -----------------------------------------------------------------------------

#[test]
fn a_forbidden_acquaintance_forms_nothing_in_that_direction_and_the_other_direction_forms() {
    let mut town = Town::new(
        "ie5-acquaint",
        &[(
            "relationships",
            "rules:\n  - { action: acquaint, actor: resident, target: commuter, effect: forbid }\n",
        )],
    );
    let (carol, grace) = (town.id("carol"), town.id("grace"));
    let (result, facts) = town.talk("carol", "grace");
    assert!(accepted(&result), "{result:?}");
    let acquainted = of_type(&facts, "became-acquainted");
    assert_eq!(acquainted.len(), 1, "one direction only: {acquainted:?}");
    assert_eq!(
        acquainted[0].subjects(),
        [grace],
        "grace comes to know carol"
    );
    let read = town.world.world().read();
    let edges: Vec<(EntityId, EntityId)> = read
        .relations_of_type(&knows())
        .map(|edge| (edge.from(), edge.to()))
        .collect();
    assert_eq!(edges, [(grace, carol)], "no knows edge carol → grace");
    let holds = |holder: EntityId| {
        read.component::<Acquaintances>(holder)
            .is_some_and(|known| !known.known().is_empty())
    };
    assert!(!holds(carol), "carol holds no entry about grace");
    assert!(holds(grace));
}

// ---- IE-8 ---------------------------------------------------------------------------------------

#[test]
fn a_listener_who_keeps_nothing_keeps_nothing_and_a_bound_keeps_the_last_lines() {
    let mut town = Town::new(
        "ie8-remember",
        &[(
            "conversation",
            "parameters:\n  - { target: resident, remembered: 2 }\n\
             consequences:\n  - { fact: spoke, target: commuter, remember: off }\n",
        )],
    );
    let (result, facts) = town.talk("carol", "grace");
    assert!(accepted(&result), "{result:?}");
    assert_eq!(of_type(&facts, "spoke").len(), 1, "the fact is recorded");
    assert!(
        town.history("grace").is_empty(),
        "grace keeps nothing of it"
    );

    // One direction to a listener who keeps nothing: each line starts a conversation (QIE-5).
    let (_, first) = town.talk("otto", "grace");
    town.wait(50);
    let (_, second) = town.talk("otto", "grace");
    assert_eq!(
        of_type(&first, "conversation-started").len()
            + of_type(&second, "conversation-started").len(),
        2,
        "two lines 60 s apart, two beginnings"
    );

    // A resident keeps at most two lines.
    let mut said = Vec::new();
    for _ in 0..3 {
        let (result, facts) = town.talk("grace", "carol");
        assert!(accepted(&result), "{result:?}");
        let spoke = of_type(&facts, "spoke")[0];
        let line: serde_json::Value =
            serde_json::from_slice(spoke.payload().payload()).expect("a JSON payload");
        said.push(line["utterance"].as_str().expect("an utterance").to_owned());
    }
    assert_eq!(town.history("carol"), said[1..], "exactly the last two");
}

// ---- IE-9 ---------------------------------------------------------------------------------------

#[test]
fn an_activity_lasts_what_its_place_says_and_its_facts_are_routed() {
    let section = "regions:\n  park:\n    parameters: [ { activity_length: 600 } ]\n\
                   consequences:\n\
                   \x20 - { fact: group-activity-started, audience: participants }\n\
                   \x20 - { fact: joined-group-activity, actor: resident, biography: off }\n";
    let mut town = Town::new("ie9-activity", &[("group-activity", section)]);

    assert!(accepted(&town.act("dev", "erin", &coffee()).0));
    let (result, park) = town.act("erin", "dev", &AcceptInvitation::default());
    assert!(accepted(&result), "{result:?}");
    let started = of_type(&park, "group-activity-started")[0].clone();
    assert_eq!(*started.visibility(), Visibility::Participants);
    let park_start = started.at().seconds();

    assert!(accepted(&town.act("hana", "grace", &coffee()).0));
    let (result, cafe) = town.act("grace", "hana", &AcceptInvitation::default());
    assert!(accepted(&result), "{result:?}");
    let cafe_start = of_type(&cafe, "group-activity-started")[0].at().seconds();

    // Carol, a resident, and Bob, of no class, join the café's activity.
    let (_, carol_joined) = town.act("carol", "grace", &JoinGroupActivity::default());
    let (_, bob_joined) = town.act("bob", "grace", &JoinGroupActivity::default());
    let carol_joined = of_type(&carol_joined, "joined-group-activity")[0].clone();
    let bob_joined = of_type(&bob_joined, "joined-group-activity")[0].clone();

    let ends = |events: &[EventEnvelope]| -> Vec<i64> {
        of_type(events, "group-activity-ended")
            .iter()
            .map(|fact| fact.at().seconds())
            .collect()
    };
    let to = |town: &mut Town, instant: i64| {
        let seconds = instant - town.at;
        town.wait(seconds)
    };
    assert!(ends(&to(&mut town, park_start + 599)).is_empty());
    assert_eq!(
        ends(&to(&mut town, park_start + 600)),
        [park_start + 600],
        "the park's 600 s"
    );
    assert!(ends(&to(&mut town, cafe_start + 3_599)).is_empty());
    assert_eq!(
        ends(&to(&mut town, cafe_start + 3_600)),
        [cafe_start + 3_600],
        "the café's compiled 3 600 s"
    );

    // The joined fact leaves a resident's biography and stays in anyone else's.
    let configured_fact = town
        .world
        .genesis()
        .iter()
        .find(|fact| fact.event_type().as_str() == "group-activity-interactions-configured")
        .expect("the section's genesis fact")
        .clone();
    let table = biography::consequences::<GroupActivitySystem>(configured_fact.payload().payload())
        .expect("the payload reads");
    let read = town.world.world().read();
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
    let configured = Configured::none()
        .with_entities(entities)
        .with_section(<GroupActivitySystem as InteractionSection>::FACTS, &table);
    let compiled = mineworld_group_activity::BIOGRAPHICAL
        .iter()
        .cloned()
        .collect();
    assert!(!biography::selected(
        &carol_joined,
        town.id("carol"),
        &compiled,
        &configured
    ));
    assert!(biography::selected(
        &bob_joined,
        town.id("bob"),
        &compiled,
        &configured
    ));
}

// ---- refusals at load ---------------------------------------------------------------------------

#[test]
fn what_a_social_section_cannot_say_is_refused_at_its_line_and_column() {
    for (name, key, text, expected) in [
        (
            "ie3c-decline",
            "group-activity",
            "rules:\n  - { action: decline-invitation, effect: forbid }\n",
            "'decline-invitation' is not an action 'group-activity' declares",
        ),
        (
            "ie5-region",
            "relationships",
            "regions:\n  cafe:\n    rules: [ { action: acquaint, effect: forbid } ]\n",
            "line 3 column 14: a region may not carry a rule about 'acquaint'",
        ),
        (
            "ie4-other-fact",
            "relationships",
            "consequences:\n  - { fact: spoke, biography: off }\n",
            "'spoke' is not a fact 'relationships' lets a list govern",
        ),
        (
            "ie7-public",
            "conversation",
            "consequences:\n  - { fact: spoke, audience: public }\n",
            "the audience of 'spoke' may only narrow",
        ),
        (
            "ie7-place",
            "conversation",
            "consequences:\n  - { fact: conversation-started, audience: place }\n",
            "the audience of 'conversation-started' may only narrow",
        ),
        (
            "ie7-nobody",
            "conversation",
            "consequences:\n  - { fact: spoke, audience: nobody }\n",
            "unknown variant",
        ),
    ] {
        let scratch = copy(name, &[(key, text)]);
        let refused = match WorldPack::read(&scratch) {
            Err(error) => error.to_string(),
            Ok(pack) => pack
                .load(WorldTime::EPOCH)
                .err()
                .unwrap_or_else(|| panic!("{name}: refused"))
                .to_string(),
        };
        assert!(refused.contains(expected), "{name}: {refused}");
        assert!(
            refused.contains("error: line "),
            "{name}: at its line and column: {refused}"
        );
    }
}

// ---- SD-IE-8 ------------------------------------------------------------------------------------

/// `acquaint` is a rule name relationships asks in a reaction; no installed pack provides an action
/// of that type, so nothing a client sends can be confused with it.
#[test]
fn no_installed_pack_provides_an_action_named_acquaint() {
    // As a host does before it composes a world (ARC-39, ARC-62).
    mineworld_worldpack::catalog::Capability::register_extensions();
    let mut world = mineworld_kernel::World::new();
    for capability in mineworld_worldpack::catalog::AVAILABLE {
        capability
            .install(&mut world)
            .expect("the installed set installs");
    }
    let acquaint = mineworld_relationships::interactions::ACQUAINT;
    assert!(world.systems().routes().count() > 10, "the whole set");
    assert_eq!(world.systems().provider(&acquaint), None);
}
