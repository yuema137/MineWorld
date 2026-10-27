//! The fetch band after S35 N-h (`pr-s35-nh-hands-and-errands.md` R1, R3, R4, R5; AC-Nh-5 rows 1 and 4),
//! and a disclosed shift as an appointment (R6), decided from observations built by hand. Every record is
//! written as the world discloses it — slug and JSON — since this controller names no market crate.

use mineworld_contracts::{
    Action, ActionRequest, ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType,
    Location, Observation, PerceivedEntity, PlaceId, ProcessId, Rejection, RejectionCode,
    SimDuration, SpatialRequirement, Tag, Tags, WorldTime,
};
use mineworld_movement::{Passage, Passages, WalkTo, move_offer_requirement};
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use serde_json::{Value, json};

use crate::commute::{Commute, errand_fits, next_part, pending};
use crate::fetch::FreeHand;
use crate::needs::NeedsPolicy;

const ME: u64 = 3;
const BOB: u64 = 4;
const STREET: u64 = 30;
const STORE: u64 = 31;
const CAFE: u64 = 33;
const HOME: u64 = 20;
const APPLE: u64 = 10;
const BREAD: u64 = 11;
const TEA: u64 = 12;
const BOOK: u64 = 13;
const SOAP: u64 = 14;
/// 06:00 on day 2.
const MORNING: i64 = 86_400 + 6 * 3_600;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

fn item(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "item" })
}

fn record(entity: u64, component_type: &str, payload: Value) -> ComponentRecord<Value> {
    serde_json::from_value(json!({
        "entity": entity.to_string(),
        "component_type": component_type,
        "schema_version": 1,
        "payload": payload,
    }))
    .expect("a record")
}

/// How an offer stands: available, or refused with a system code.
#[derive(Clone, Copy)]
enum Stand {
    Yes,
    No(&'static str),
}

/// Where I stand, hungry, and everything the world discloses and offers there.
#[derive(Clone)]
struct Scene {
    at: u64,
    place_tags: Vec<&'static str>,
    doors: Vec<(u64, Vec<&'static str>)>,
    /// What every food kind would nourish me (`needs`' `kinds`).
    kinds: Vec<(u64, u16)>,
    /// What I hold: kind, count.
    held: Vec<(u64, u32)>,
    /// Offers by action slug: kind, target, how it stands.
    offers: Vec<(&'static str, u64, Option<u64>, Stand)>,
    /// `shop-stock`: shop place, stocked kinds; `None` discloses no record.
    stock: Option<Vec<(u64, Vec<u64>)>>,
}

impl Scene {
    fn at(at: u64) -> Self {
        Self {
            at,
            place_tags: Vec::new(),
            doors: Vec::new(),
            kinds: vec![(APPLE, 300), (BREAD, 400), (TEA, 0)],
            held: Vec::new(),
            offers: Vec::new(),
            stock: None,
        }
    }

    fn offer(mut self, slug: &'static str, kind: u64, target: Option<u64>, stand: Stand) -> Self {
        self.offers.push((slug, kind, target, stand));
        self
    }

    fn build(&self) -> Observation<Value> {
        let here = Location::in_place(place(self.at));
        let leads_to: Vec<Value> = self
            .doors
            .iter()
            .map(|(to, tags)| {
                let mut door =
                    serde_json::to_value(Passage::new(place(*to), None, None)).expect("serializes");
                door["to_tags"] = json!(tags);
                door
            })
            .collect();
        let tags = Tags::new(
            self.place_tags
                .iter()
                .map(|name| Tag::new(*name).expect("a tag")),
        );
        let mut site = PerceivedEntity::new(id(self.at), EntityType::Place).with_tags(tags);
        if !self.doors.is_empty() {
            site = site.with_components(vec![ComponentRecord::new::<Passages>(
                id(self.at),
                json!({ "leads_to": leads_to }),
            )]);
        }
        let nourishing = |raw: u64| {
            self.kinds
                .iter()
                .find(|(kind, _)| *kind == raw)
                .map_or(0, |(_, hunger)| *hunger)
        };
        let carried: Vec<Value> = self
            .held
            .iter()
            .filter(|(raw, _)| nourishing(*raw) > 0)
            .map(|(raw, _)| json!({ "item": item(*raw), "hunger": nourishing(*raw) }))
            .collect();
        let kinds: Vec<Value> = self
            .kinds
            .iter()
            .map(|(raw, hunger)| json!({ "item": item(*raw), "hunger": hunger }))
            .collect();
        let held: Vec<Value> = self
            .held
            .iter()
            .map(|(raw, count)| json!({ "item": item(*raw), "count": count }))
            .collect();
        let mut mine = vec![
            record(
                ME,
                "needs",
                json!({ "hunger": 700, "tiredness": 0, "nourishment": carried, "kinds": kinds }),
            ),
            record(ME, "holdings", json!({ "held": held, "capacity": 6 })),
        ];
        if let Some(stock) = &self.stock {
            let shops: Vec<Value> = stock
                .iter()
                .map(|(shop, kinds)| {
                    json!({
                        "place": serde_json::to_value(place(*shop)).expect("serializes"),
                        "stocked": kinds.iter().map(|raw| item(*raw)).collect::<Vec<_>>(),
                    })
                })
                .collect();
            mine.push(record(ME, "shop-stock", json!({ "shops": shops })));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here)
            .with_components(mine);
        let bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here);
        let mut affordances = vec![Affordance::available(
            WalkTo::ACTION_TYPE,
            None,
            move_offer_requirement(),
        )];
        for (slug, kind, target, stand) in &self.offers {
            let action = ActionTypeId::new(*slug).expect("a slug");
            let target = target.map(id);
            let offer = match stand {
                Stand::Yes => Affordance::available(action, target, SpatialRequirement::NONE),
                Stand::No(code) => Affordance::unavailable(
                    action,
                    target,
                    SpatialRequirement::NONE,
                    Rejection::System {
                        code: RejectionCode::new(*code).expect("a code"),
                        detail: None,
                    },
                ),
            };
            affordances.push(offer.with_payload(json!({ "item": item(*kind) })));
        }
        Observation::new(id(ME), WorldTime::from_seconds(MORNING))
            .at_location(here)
            .perceiving(vec![site, me, bob])
            .offering(affordances)
    }
}

fn decided(policy: &NeedsPolicy, scene: &Scene) -> Option<ActionRequest> {
    policy.fetch(&scene.build(), false).flatten()
}

fn what(request: &ActionRequest) -> (String, Value) {
    let payload: Value = serde_json::from_slice(request.payload().payload()).expect("JSON");
    let named = payload["item"]["entity"].clone();
    let named = if named.is_null() {
        payload["to"].clone()
    } else {
        named
    };
    (request.action_type().as_str().to_owned(), named)
}

fn walked(raw: u64) -> Value {
    serde_json::to_value(mineworld_movement::Destination::Place(Location::in_place(
        place(raw),
    )))
    .expect("serializes")
}

/// AC-Nh-5 row 1: full hands at a shop with an apple, holding a tea — drink the tea, then buy the apple.
#[test]
fn full_at_a_shop_a_person_drinks_the_tea_that_does_not_feed_them_then_buys_the_apple() {
    let policy = NeedsPolicy::default();
    let full = Scene {
        place_tags: vec!["shop"],
        held: vec![(TEA, 1), (BOOK, 2), (SOAP, 3)],
        ..Scene::at(STORE)
    }
    .offer("buy", APPLE, None, Stand::No("cannot-carry"))
    .offer("drink", TEA, None, Stand::Yes)
    .offer("put-down", BOOK, None, Stand::Yes)
    .offer("put-down", SOAP, None, Stand::Yes);
    let first = decided(&policy, &full).expect("frees a hand");
    assert_eq!(what(&first), ("drink".to_owned(), json!(TEA.to_string())));
    // The tea is gone; there is room, and the apple is bought.
    let room = Scene {
        held: vec![(BOOK, 2), (SOAP, 3)],
        offers: Vec::new(),
        ..full.clone()
    }
    .offer("buy", APPLE, None, Stand::Yes);
    let second = decided(&policy, &room).expect("buys");
    assert_eq!(what(&second), ("buy".to_owned(), json!(APPLE.to_string())));
}

/// AC-Nh-5 row 2 (the controller's half): six goods and nothing to drink, alone — put the most-held kind
/// down; with `free_hand` emptied, nothing is freed; with only `give`, nobody to give to but Bob.
#[test]
fn holding_only_goods_a_person_puts_the_most_held_kind_down_by_the_policy_order() {
    let policy = NeedsPolicy::default();
    let goods = Scene {
        place_tags: vec!["shop"],
        held: vec![(BOOK, 2), (SOAP, 4)],
        ..Scene::at(STORE)
    }
    .offer("buy", BREAD, None, Stand::No("cannot-carry"))
    .offer("put-down", BOOK, None, Stand::Yes)
    .offer("put-down", SOAP, None, Stand::Yes)
    .offer("give", BOOK, Some(BOB), Stand::Yes);
    let freed = decided(&policy, &goods).expect("puts down");
    assert_eq!(
        what(&freed),
        ("put-down".to_owned(), json!(SOAP.to_string()))
    );
    let givers = NeedsPolicy {
        free_hand: vec![FreeHand::Give],
        ..policy.clone()
    };
    let given = decided(&givers, &goods).expect("gives");
    assert_eq!(what(&given).0, "give");
    assert_eq!(given.target(), Some(id(BOB)));
    let stuck = NeedsPolicy {
        free_hand: Vec::new(),
        ..policy
    };
    assert_eq!(
        decided(&stuck, &goods),
        None,
        "an empty list turns the step off"
    );
}

/// R3, R4: what lies here is picked up first, and only what feeds me is bought or picked up — a coffee
/// is no breakfast.
#[test]
fn food_lying_here_comes_first_and_a_drink_is_never_bought_as_food() {
    let policy = NeedsPolicy::default();
    let pantry = Scene::at(HOME)
        .offer("pick-up", BOOK, None, Stand::Yes)
        .offer("pick-up", BREAD, None, Stand::Yes)
        .offer("buy", APPLE, None, Stand::Yes);
    let picked = decided(&policy, &pantry).expect("picks up");
    assert_eq!(
        what(&picked),
        ("pick-up".to_owned(), json!(BREAD.to_string()))
    );
    let teashop = Scene {
        place_tags: vec!["shop"],
        ..Scene::at(STORE)
    }
    .offer("buy", TEA, None, Stand::Yes);
    assert_eq!(
        decided(&policy, &teashop),
        None,
        "tea nourishes 0: not bought"
    );
    let bakery =
        teashop
            .clone()
            .offer("buy", APPLE, None, Stand::Yes)
            .offer("buy", BREAD, None, Stand::Yes);
    let bought = decided(&policy, &bakery).expect("buys");
    assert_eq!(
        what(&bought),
        ("buy".to_owned(), json!(BREAD.to_string())),
        "the most nourishing"
    );
}

/// AC-Nh-5 row 4, R5: a shop whose disclosed stock has nothing for me is not walked to; one that has is;
/// with no shop anywhere stocking food, a dead end is not left either.
#[test]
fn a_shop_with_nothing_for_me_is_not_walked_to() {
    let policy = NeedsPolicy::default();
    let street = Scene {
        doors: vec![
            (HOME, vec!["residential"]),
            (STORE, vec!["shop"]),
            (CAFE, vec!["shop"]),
        ],
        stock: Some(vec![(STORE, vec![TEA]), (CAFE, vec![APPLE])]),
        ..Scene::at(STREET)
    };
    let walk = decided(&policy, &street).expect("walks");
    assert_eq!(what(&walk), ("walk-to".to_owned(), walked(CAFE)));
    let empty = Scene {
        stock: Some(vec![(STORE, Vec::new()), (CAFE, vec![TEA])]),
        ..street.clone()
    };
    assert_eq!(decided(&policy, &empty), None, "no shop stocks food for me");
    let home = Scene {
        place_tags: vec!["residential"],
        doors: vec![(STREET, vec!["street"])],
        ..empty.clone()
    };
    let home = Scene { at: HOME, ..home };
    assert_eq!(
        decided(&policy, &home),
        None,
        "nowhere has food: I stay home"
    );
    // Food elsewhere, not through a door here: out of the dead end, as before.
    let hopeful = Scene {
        stock: Some(vec![(STORE, vec![APPLE])]),
        ..home
    };
    let out = decided(&policy, &hopeful).expect("goes out");
    assert_eq!(what(&out), ("walk-to".to_owned(), walked(STREET)));
    // An empty shop I stand in, food at the café: leave by the walk rule (not waiting).
    let store = Scene {
        at: STORE,
        place_tags: vec!["shop"],
        doors: vec![(STREET, vec!["street"])],
        stock: Some(vec![(STORE, Vec::new()), (CAFE, vec![BREAD])]),
        ..Scene::at(STORE)
    }
    .offer("buy", APPLE, None, Stand::No("out-of-stock"));
    let leave = decided(&policy, &store).expect("leaves");
    assert_eq!(what(&leave), ("walk-to".to_owned(), walked(STREET)));
    assert_eq!(policy.fetch(&store.build(), true), None, "waiting holds me");
    // Too poor for the food here: nothing more (F-Nh-3).
    let broke = Scene {
        stock: Some(vec![(STORE, vec![APPLE]), (CAFE, vec![BREAD])]),
        offers: Vec::new(),
        ..store
    }
    .offer("buy", APPLE, None, Stand::No("insufficient-funds"));
    assert_eq!(decided(&policy, &broke), None);
}

/// The `free_hand` block decodes, refuses an unknown step and a list too long, and defaults to the
/// design's order.
#[test]
fn the_free_hand_order_is_policy_data() {
    let default: NeedsPolicy = serde_json::from_value(json!({})).expect("defaults");
    assert_eq!(
        default.free_hand,
        vec![
            FreeHand::Drink,
            FreeHand::Eat,
            FreeHand::PutDown,
            FreeHand::Give
        ]
    );
    let written: NeedsPolicy =
        serde_json::from_value(json!({ "free_hand": ["put-down"] })).expect("one step");
    assert_eq!(written.free_hand, vec![FreeHand::PutDown]);
    assert!(serde_json::from_value::<NeedsPolicy>(json!({ "free_hand": ["throw"] })).is_err());
    let five = json!({ "free_hand": ["drink", "eat", "put-down", "give", "drink"] });
    let refused = serde_json::from_value::<NeedsPolicy>(five).expect_err("too long");
    assert!(refused.to_string().contains("at most 4"), "{refused}");
}

// ---- R6: a disclosed shift is an appointment -----------------------------------------------------

/// Me at home at `at`, my routine's next part (place, from, punctual), and my disclosed job (workplace,
/// from) when there is one.
fn day(at: i64, next: Option<(u64, i64, bool)>, job: Option<(u64, &str)>) -> Observation<Value> {
    let here = Location::in_place(place(HOME));
    let mut mine = Vec::new();
    if let Some((to, from, punctual)) = next {
        let agenda = Agenda::new(
            place(HOME),
            AgendaLabel::new("home").expect("a label"),
            WorldTime::from_seconds(0),
            WorldTime::from_seconds(from),
            ProcessId::from_raw(1),
        );
        let disclosed = DisclosedAgenda {
            agenda,
            next: Some(NextPart {
                place: place(to),
                label: AgendaLabel::new("meet").expect("a label"),
                from: WorldTime::from_seconds(from),
                punctual,
            }),
        };
        mine.push(ComponentRecord::new::<Agenda>(
            id(ME),
            serde_json::to_value(&disclosed).expect("serializes"),
        ));
    }
    if let Some((workplace, from)) = job {
        mine.push(record(
            ME,
            "employment",
            json!({ "job": {
                "employer": { "entity": "40", "entity_type": "organization" },
                "workplace": serde_json::to_value(place(workplace)).expect("serializes"),
                "from": from, "until": "12:00", "wage": 100, "produces": [],
            }}),
        ));
    }
    let me = PerceivedEntity::new(id(ME), EntityType::Person)
        .at(here)
        .with_components(mine);
    Observation::new(id(ME), WorldTime::from_seconds(at))
        .at_location(here)
        .perceiving(vec![me])
}

#[test]
fn a_disclosed_shift_is_an_appointment_beside_the_punctual_flag() {
    let seven = 86_400 + 7 * 3_600;
    // The routine's part at the workplace at the shift's start, not flagged: it becomes punctual.
    let unflagged = day(MORNING, Some((STORE, seven, false)), Some((STORE, "07:00")));
    let next = next_part(&unflagged).expect("a part");
    assert_eq!(
        (next.place, next.from.seconds(), next.punctual),
        (place(STORE), seven, true)
    );
    assert!(pending(&unflagged).is_some(), "a commute is planned for it");
    let due = Commute::default().due(
        &unflagged,
        mineworld_contracts::SimDuration::from_seconds(900),
        mineworld_contracts::SimDuration::from_seconds(30),
    );
    assert_eq!(due, Some(place(STORE)), "within the cap of 07:00, due");
    // No routine part there: the shift itself, at its next start.
    let shift_only = day(
        MORNING,
        Some((CAFE, seven + 3_600, false)),
        Some((STORE, "07:00")),
    );
    let next = next_part(&shift_only).expect("the shift");
    assert_eq!(
        (next.place, next.from.seconds(), next.punctual),
        (place(STORE), seven, true)
    );
    // An earlier punctual part elsewhere comes first.
    let earlier = day(
        MORNING,
        Some((CAFE, seven - 600, true)),
        Some((STORE, "07:00")),
    );
    assert_eq!(next_part(&earlier).expect("a part").place, place(CAFE));
    // After the shift began, the next one is tomorrow's.
    let after = day(seven + 60, None, Some((STORE, "07:00")));
    assert_eq!(
        next_part(&after).expect("tomorrow").from.seconds(),
        seven + 86_400
    );
    // No job disclosed: exactly the routine's part, unflagged.
    let jobless = day(MORNING, Some((STORE, seven, false)), None);
    assert!(!next_part(&jobless).expect("a part").punctual);
    assert!(pending(&jobless).is_none());
}

/// F-Nh-7: an errand is judged as a round trip before a punctual next part. With no way disclosed the
/// lead and the errand window are both the cap (120 min), so at a pace of 15 min an errand fits only
/// while `now + 15 + 120 ≤ from − 120` minutes; a jobless day, with no punctual part, always fits.
#[test]
fn an_errand_fits_only_while_the_commute_stays_ahead_of_it() {
    let seven = 86_400 + 7 * 3_600;
    let (pace, stride) = (
        SimDuration::from_seconds(900),
        SimDuration::from_seconds(30),
    );
    let fits = |at: i64, job: Option<(u64, &str)>| {
        let observation = day(at, Some((STORE, seven, false)), job);
        errand_fits(&observation, &Commute::default(), pace, stride, place(CAFE))
    };
    let edge = seven - 7_200 - 900 - 7_200;
    assert!(
        fits(edge, Some((STORE, "07:00"))),
        "at the edge it still fits"
    );
    assert!(
        !fits(edge + 1, Some((STORE, "07:00"))),
        "a second later the walk back would make me late"
    );
    assert!(fits(seven - 600, None), "no appointment: any errand fits");
}
