//! The 2D reference client's interactions (`clients/2d`, step-13 S12 PR 13b), run for real against a
//! stub server that says exactly what a test needs and against real worlds: AC-I1 … AC-I8 and AC-I12
//! of step-13 §15.3.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_2d_interact -- --ignored --test-threads=1
//! ```
//!
//! The scripted player never names an action type in GDScript: this file writes the steps — which
//! subject to click, which rendered entry to choose by a selector, what to type — and the client
//! resolves each selector against the menu it actually drew. Oracles are independent of the client:
//! what the stub offered and received, and on a real server the save's fact log.

mod godot2d;
mod headless;
mod support;

use std::path::Path;

use godot2d::{Drive, STUB_OBSERVER, StubWorld, offering, passed, record, tagged};
use serde_json::{Value, json};

/// Writes `steps` into `dir` and returns the file's path.
fn steps_file(dir: &Path, steps: &Value) -> String {
    let path = dir.join("steps.json");
    std::fs::write(&path, serde_json::to_string_pretty(steps).expect("JSON"))
        .expect("steps written");
    path.to_str().expect("a UTF-8 path").to_owned()
}

/// The `MENU` lines about `subject`, in order.
fn menus(lines: &[String], subject: &str) -> Vec<Value> {
    tagged(lines, "MENU ")
        .into_iter()
        .filter(|menu| menu["subject"] == subject)
        .collect()
}

/// `value` with every number as a double: what the client read, since Godot parses every JSON number
/// that way, compared with what was sent without caring how the integer was spelled.
fn numeric(value: &Value) -> Value {
    match value {
        Value::Number(n) => json!(n.as_f64().expect("a finite number")),
        Value::Array(items) => Value::Array(items.iter().map(numeric).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .iter()
                .map(|(k, v)| (k.clone(), numeric(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// A requirement as the server writes one, for a stub offer.
fn requirement(range: Option<f64>) -> Value {
    json!({ "place": "same_place_as_actor", "within_range": range, "requires_line_of_access": false,
        "requires_target_available": true })
}

/// An offer as the server writes one.
fn offer(
    action_type: &str,
    target: Option<&str>,
    payload: Option<Value>,
    verdict: Result<(), Value>,
) -> Value {
    let mut offered = json!({ "action_type": action_type, "target": target,
        "available": verdict.is_ok(), "unavailable_reason": verdict.err(),
        "requirement": requirement(Some(3000.0)) });
    if let Some(payload) = payload {
        offered["payload"] = payload;
    }
    offered
}

const P: &str = "11";
const Q: &str = "12";

fn two_people() -> Vec<(&'static str, &'static str, (i64, i64))> {
    vec![
        (P, "Pia Stub", (5000, 2000)),
        (Q, "Quin Stub", (3000, 4500)),
    ]
}

/// AC-I4 — the menus are the offers: P's lists exactly its four offers in the server's order, the two
/// complete gives separately, the unavailable ones greyed with the server's reason, the unknown
/// incomplete `wave` disabled and never sent; Q's lists nothing; one's own lists the target-less
/// `ring` (a pack this client never heard of) and `buy`; `ring` arrives exactly as offered, its
/// `count` an integer.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn the_menus_are_the_offers() {
    let item = |id: &str| json!({ "entity": id, "entity_type": "item" });
    let world = StubWorld {
        people: two_people(),
        offers: vec![
            offer(
                "give",
                Some(P),
                Some(json!({ "item": item("31"), "count": 1 })),
                Ok(()),
            ),
            offer("talk", Some(P), None, Err(json!("busy"))),
            offer(
                "give",
                Some(P),
                Some(json!({ "item": item("30"), "count": 1 })),
                Err(json!({ "inventory-full": { "limit": 3 } })),
            ),
            offer("wave", Some(P), None, Ok(())),
            offer(
                "ring",
                None,
                Some(json!({ "bell": "low", "count": 2 })),
                Ok(()),
            ),
            offer("buy", None, Some(json!({ "item": item("30") })), Ok(())),
        ],
        ..StubWorld::default()
    };
    let (address, log) = offering(world).await;
    let dir = mineworld_test_support::scratch!(empty "2d-i4");
    let steps = steps_file(
        &dir,
        &json!([
            { "open": P }, { "open": Q }, { "open": "self" },
            { "choose": { "action_type": "ring" } },
            { "open": P }, { "choose": { "action_type": "wave" } },
            { "sleep": 1 }
        ]),
    );
    let (status, lines) = Drive::start(
        address,
        "carol",
        &["--drive=steps", &format!("--steps={steps}")],
    )
    .finish()
    .await;
    assert!(status.success() && passed(&lines), "the drive passes");

    let p = &menus(&lines, P)[0];
    let entries = p["entries"].as_array().expect("entries");
    let types: Vec<&str> = entries
        .iter()
        .map(|e| e["action_type"].as_str().expect("a type"))
        .collect();
    assert_eq!(
        types,
        ["give", "talk", "give", "wave"],
        "P's menu, in the server's order: {p}"
    );
    assert_eq!(entries[0]["about"], "31");
    assert_eq!(entries[2]["about"], "30");
    assert!(
        entries[0]["enabled"] == true
            && entries[0]["available"] == true
            && entries[0]["complete"] == true
    );
    assert_eq!(entries[1]["reason"], "busy");
    assert_eq!(entries[1]["available"], false);
    assert_eq!(entries[1]["enabled"], true, "greyed, but still choosable");
    assert!(
        entries[1]["label"]
            .as_str()
            .is_some_and(|l| l.contains("busy")),
        "the reason is worded in the label: {}",
        entries[1]["label"]
    );
    assert_eq!(
        numeric(&entries[2]["reason"]),
        numeric(&json!({ "inventory-full": { "limit": 3 } })),
        "the reason as sent"
    );
    assert_eq!(entries[2]["available"], false);
    assert_eq!(
        entries[3]["enabled"], false,
        "wave is not composable by this client"
    );
    assert!(
        entries[3]["label"]
            .as_str()
            .is_some_and(|l| l.contains("not supported by this client")),
        "{}",
        entries[3]["label"]
    );
    let q = &menus(&lines, Q)[0];
    assert_eq!(
        q["entries"],
        json!([]),
        "nothing is offered against Q, so nothing is listed: {q}"
    );
    let own = &menus(&lines, STUB_OBSERVER)[0];
    let own_types: Vec<&str> = own["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["action_type"].as_str().expect("a type"))
        .collect();
    assert_eq!(own_types, ["ring", "buy"], "one's own menu: {own}");

    let log = log.lock().expect("log");
    assert_eq!(
        log.submitted.len(),
        1,
        "ring was sent, wave was not: {:?}",
        log.submitted
    );
    let ring = &log.submitted[0].1;
    assert_eq!(ring["action_type"], "ring");
    assert_eq!(ring["target"], Value::Null);
    assert_eq!(
        ring["payload"]["payload"],
        json!({ "bell": "low", "count": 2 })
    );
    assert!(
        ring["payload"]["payload"]["count"].is_u64(),
        "an integer, as offered: {ring}"
    );
}

/// AC-I5 — submitted regardless, never replayed: both entries the world marked unavailable are sent
/// when chosen; a third request the stub never answers (it hangs up) is shown as unknown, and nothing
/// is sent again after the client reconnects.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn submitted_regardless_and_never_replayed() {
    let item = json!({ "entity": "31", "entity_type": "item" });
    let world = StubWorld {
        people: two_people(),
        offers: vec![
            offer("talk", Some(P), None, Err(json!("too_far_away"))),
            offer(
                "give",
                Some(P),
                Some(json!({ "item": item, "count": 1 })),
                Err(json!("too_far_away")),
            ),
        ],
        later_offers: Some((2, vec![offer("talk", Some(P), None, Ok(()))])),
        hang_up_on: Some(3),
        ..StubWorld::default()
    };
    let (address, log) = offering(world).await;
    let dir = mineworld_test_support::scratch!(empty "2d-i5");
    let steps = steps_file(
        &dir,
        &json!([
            { "open": P }, { "choose": { "action_type": "talk" }, "input": "first" },
            { "open": P }, { "choose": { "action_type": "give" } },
            { "await": { "offered": { "action_type": "talk", "target": P } } },
            { "open": P }, { "choose": { "action_type": "talk" }, "input": "third" },
            { "await": { "seated": true } },
            { "sleep": 3 }
        ]),
    );
    let (status, lines) = Drive::start(
        address,
        "carol",
        &["--drive=steps", &format!("--steps={steps}")],
    )
    .finish()
    .await;
    assert!(status.success() && passed(&lines), "the drive passes");
    let log = log.lock().expect("log");
    let sent: Vec<(&str, &Value)> = log
        .submitted
        .iter()
        .map(|(_, r)| {
            (
                r["action_type"].as_str().expect("a type"),
                &r["payload"]["payload"],
            )
        })
        .collect();
    assert_eq!(
        sent.len(),
        3,
        "three submits, none after the reconnect: {sent:?}"
    );
    assert_eq!(
        sent[0],
        ("talk", &json!({ "utterance": "first" })),
        "the unavailable talk was sent"
    );
    assert_eq!(sent[1].0, "give", "the unavailable give was sent");
    assert_eq!(sent[2], ("talk", &json!({ "utterance": "third" })));
    assert_eq!(log.connections, 2, "the client reconnected once");
    let unknown: Vec<Value> = tagged(&lines, "TOAST ")
        .into_iter()
        .filter(|t| t["kind"] == "unknown")
        .collect();
    assert!(
        unknown
            .iter()
            .any(|t| t["action_type"] == "talk" && t["target"] == P),
        "the unanswered talk is shown as unknown: {unknown:?}"
    );
}

/// AC-I12 — panels show what is disclosed: each known own component through its reader with every
/// value the frame carries, the place's shop listing, an unknown own component raw in "other", and a
/// component that disappears from a frame gone from the panels at that frame.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn the_panels_show_what_is_disclosed() {
    const GONE_AFTER: u64 = 25;
    let me = STUB_OBSERVER;
    let item = json!({ "entity": "30", "entity_type": "item" });
    let pia = json!({ "entity": P, "entity_type": "person" });
    let world = StubWorld {
        people: two_people(),
        own: vec![
            (record(me, "wallet", json!({ "balance": 12345 })), None),
            (
                record(
                    me,
                    "holdings",
                    json!({ "held": [{ "count": 2, "item": item }] }),
                ),
                None,
            ),
            (
                record(
                    me,
                    "acquaintances",
                    json!({ "known": [{ "counterpart": pia, "values": {} }] }),
                ),
                None,
            ),
            (
                record(
                    me,
                    "invitations",
                    json!({ "pending": [{ "from": pia, "kind": "coffee", "at": 0 }] }),
                ),
                None,
            ),
            (
                record(
                    me,
                    "agenda",
                    json!({ "label": "visit", "place": { "entity": "1", "entity_type": "place" },
                "routine": "4", "since": 28800, "until": 43200 }),
                ),
                None,
            ),
            (
                record(
                    me,
                    "employment",
                    json!({ "job": { "employer": { "entity": "40", "entity_type": "organization" },
                "workplace": { "entity": "1", "entity_type": "place" }, "from": 25200, "until": 54000,
                "wage": 9000, "produces": [] }, "shift": null }),
                ),
                None,
            ),
            (
                record(me, "weather-sense", json!({ "sky": "overcast", "wind": 3 })),
                Some(GONE_AFTER),
            ),
        ],
        place: vec![record(
            "1",
            "shop",
            json!({ "listed": [{ "in_stock": 4, "item": item, "price": 250 }],
            "operator": { "entity": "40", "entity_type": "organization" } }),
        )],
        ..StubWorld::default()
    };
    let (address, _log) = offering(world).await;
    let (status, lines) = Drive::start(address, "carol", &["--drive=panels", "--hold=5"])
        .finish()
        .await;
    assert!(status.success() && passed(&lines), "the drive passes");
    let all = tagged(&lines, "PANELS ");
    let first = all.first().expect("panels were drawn");
    let panel = |panels: &Value, component: &str| -> Option<Value> {
        panels["panels"]
            .as_array()
            .expect("panels")
            .iter()
            .find(|p| p["component"] == component)
            .cloned()
    };
    let rows = |component: &str| -> String {
        let shown =
            panel(first, component).unwrap_or_else(|| panic!("no {component} panel: {first}"));
        assert_eq!(shown["known"], true, "{component} is read by its reader");
        shown["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r.as_str().expect("text"))
            .collect::<Vec<_>>()
            .join(" | ")
    };
    assert!(rows("wallet").contains("123.45"), "{}", rows("wallet"));
    let holdings = rows("holdings");
    assert!(
        holdings.contains("30") && holdings.contains('2'),
        "{holdings}"
    );
    assert!(rows("acquaintances").contains("Pia Stub"));
    let invitations = rows("invitations");
    assert!(
        invitations.contains("Pia Stub") && invitations.contains("coffee"),
        "{invitations}"
    );
    let agenda = rows("agenda");
    assert!(
        agenda.contains("visit") && agenda.contains("08:00") && agenda.contains("12:00"),
        "{agenda}"
    );
    let employment = rows("employment");
    assert!(
        employment.contains("90.00")
            && employment.contains("25200")
            && employment.contains("54000"),
        "{employment}"
    );
    let shop = rows("shop");
    assert!(
        shop.contains("30") && shop.contains("2.50") && shop.contains('4'),
        "{shop}"
    );
    let other = panel(first, "weather-sense").expect("the unknown component is shown, not dropped");
    assert_eq!(other["known"], false);
    let raw: Value =
        serde_json::from_str(other["rows"][0].as_str().expect("raw JSON")).expect("raw JSON");
    assert_eq!(raw, json!({ "sky": "overcast", "wind": 3 }));
    // Gone at the frame that no longer carries it: the first panels drawn from frame GONE_AFTER + 1.
    let gone = all
        .iter()
        .find(|p| panel(p, "weather-sense").is_none())
        .expect("the panel disappears");
    assert_eq!(
        gone["seq"].as_u64(),
        Some(GONE_AFTER + 1),
        "within one observation: {gone}"
    );
}
