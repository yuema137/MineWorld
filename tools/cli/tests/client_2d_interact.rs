//! The 2D reference client's interactions (`clients/2d`, step-13 S12 PR 13b), run for real against
//! real worlds: AC-I1 … AC-I3 and AC-I6 … AC-I8 of step-13 §15.3. The checks against a stub server
//! (AC-I4, AC-I5, AC-I12) are in `client_2d_interact_stub.rs`.
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
//! resolves each selector against the menu it actually drew. The oracle is the save's fact log, read
//! after the server stops, never the client's own report alone.

mod godot2d;
mod headless;
mod support;

use std::path::Path;

use godot2d::worlds::{
    copy_dir, facts, hosted, id, ids, menus, panel, play, start, step_done, step_line,
};
use godot2d::{MARKET_TOWN, World, passed, tagged};
use mineworld_contracts::ActionRequest;
use mineworld_server::{RequestField, WirePayload, differing_fields};
use serde_json::{Value, json};
use support::SaveDir;

// ── Real worlds (AC-I1 … AC-I3, AC-I6 … AC-I8) ─────────────────────────────────────────────────────

const COFFEE_PLEASE: &str = "Hello! A coffee, please.";

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// The answer to the request sent with `token`.
fn result_of(lines: &[String], token: &Value) -> Value {
    tagged(lines, "RESULT ")
        .into_iter()
        .find(|r| r["token"] == *token)
        .unwrap_or_else(|| panic!("request {token} was never answered"))
}

/// The label the client drew for a person, from its own `SHOWN` report.
fn label(lines: &[String], person: &str) -> String {
    tagged(lines, "SHOWN ")
        .iter()
        .flat_map(|s| s["people"].as_array().cloned().unwrap_or_default())
        .find(|p| p["id"] == person)
        .and_then(|p| p["label"].as_str().map(str::to_owned))
        .unwrap_or_else(|| panic!("{person} was never shown"))
}

/// AC-I1 — talk, from the door of the café: Alice's `talk` is listed unavailable `too_far_away`;
/// chosen anyway it is sent and rejected, and the toast offers "walk to"; walked up to, it is
/// accepted; her reply reaches the history panel; the acquaintances panel follows the save.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn talks_to_alice_from_the_door() {
    let town = ids(MARKET_TOWN);
    let (alice, visitor) = (town["alice"].clone(), town["visitor"].clone());
    let save = SaveDir::new("2d-i1");
    let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save)), None).await;
    let steps = json!([
        { "open": alice }, { "choose": { "action_type": "talk" }, "input": COFFEE_PLEASE },
        { "open": alice }, { "choose": { "walk": true } },
        { "open": alice }, { "choose": { "action_type": "talk" }, "input": COFFEE_PLEASE },
        { "sleep": 10 }, { "report": true }
    ]);
    let (status, lines) = play(world.address, "visitor", "2d-i1-steps", &steps, &[]).await;
    assert!(status.success() && passed(&lines), "the drive passes");
    world.kill();

    let first = &menus(&lines, &alice)[0];
    let offered_talk = first["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .find(|e| e["action_type"] == "talk")
        .cloned()
        .expect("talk is offered against Alice");
    assert_eq!(
        offered_talk["available"], false,
        "from the door: {offered_talk}"
    );
    assert_eq!(offered_talk["reason"], "too_far_away");

    // (a) two talks to Alice: rejected too_far_away, then accepted; every stride between accepted.
    let requests = tagged(&lines, "REQUEST ");
    let talks: Vec<&Value> = requests
        .iter()
        .filter(|r| r["request"]["action_type"] == "talk" && r["request"]["target"] == alice)
        .collect();
    assert_eq!(talks.len(), 2, "two talks: {talks:?}");
    assert_eq!(
        result_of(&lines, &talks[0]["token"])["result"]["rejected"],
        "too_far_away"
    );
    let accepted = result_of(&lines, &talks[1]["token"]);
    assert!(accepted["result"].get("accepted").is_some(), "{accepted}");
    for stride in requests
        .iter()
        .filter(|r| r["request"]["action_type"] == "move")
    {
        let answer = result_of(&lines, &stride["token"]);
        assert!(
            answer["result"].get("accepted").is_some(),
            "a stride refused: {answer}"
        );
    }
    let toast = tagged(&lines, "TOAST ")
        .into_iter()
        .find(|t| t["kind"] == "rejected" && t["action_type"] == "talk")
        .expect("the rejection is shown");
    assert!(
        toast["button"].as_str().is_some_and(|b| !b.is_empty()),
        "the toast offers walk-to: {toast}"
    );

    // (b) the save: one spoke by visitor to Alice, the typed text byte for byte; Alice answers.
    let facts = facts(&save);
    let spoke: Vec<&Value> = facts
        .iter()
        .filter(|(t, _)| t == "spoke")
        .map(|(_, p)| p)
        .collect();
    let mine: Vec<usize> = (0..spoke.len())
        .filter(|&i| id(&spoke[i]["speaker"]) == visitor && id(&spoke[i]["listener"]) == alice)
        .collect();
    assert_eq!(mine.len(), 1, "one spoke by visitor to Alice: {spoke:?}");
    assert_eq!(spoke[mine[0]]["utterance"], COFFEE_PLEASE);
    let reply = spoke[mine[0] + 1..]
        .iter()
        .find(|p| id(&p["speaker"]) == alice && id(&p["listener"]) == visitor)
        .expect("Alice answers")["utterance"]
        .as_str()
        .expect("an utterance")
        .to_owned();

    // (c) within 10 s of the accepted talk, the history's newest line is that reply, by name.
    let accepted_at = accepted["t_ms"].as_u64().expect("t_ms");
    let alice_name = label(&lines, &alice);
    let shown = tagged(&lines, "PANELS ")
        .into_iter()
        .find(|p| {
            panel(p, "conversation-history").is_some_and(|h| {
                h["rows"]
                    .as_array()
                    .and_then(|rows| rows.last())
                    .and_then(Value::as_str)
                    .is_some_and(|row| row.contains(&reply) && row.contains(&alice_name))
            })
        })
        .expect("the reply reaches the history panel");
    assert!(
        shown["t_ms"].as_u64().expect("t_ms") - accepted_at <= 10_000,
        "within 10 s: {shown}"
    );

    // (d) the acquaintances panel lists Alice iff the save says they became acquainted.
    let acquainted = facts.iter().any(|(t, p)| {
        t == "became-acquainted"
            && [id(&p["person"]), id(&p["counterpart"])].contains(&visitor)
            && [id(&p["person"]), id(&p["counterpart"])].contains(&alice)
    });
    let listed = tagged(&lines, "PANELS ")
        .last()
        .and_then(|p| panel(p, "acquaintances"))
        .is_some_and(|a| {
            a["rows"]
                .as_array()
                .is_some_and(|rows| rows.iter().any(|r| r == alice_name.as_str()))
        });
    assert_eq!(listed, acquainted, "the panel shows what is disclosed");
}

/// AC-I2's steps: buy a coffee and drink it, walk to Bob and give him the scarf, eat the apple.
fn item_steps(town: &std::collections::BTreeMap<String, String>) -> Value {
    json!([
        { "open": "self" }, { "choose": { "action_type": "buy", "about": town["coffee"] } },
        { "open": "self" }, { "choose": { "action_type": "drink", "about": town["coffee"] } },
        { "open": town["bob"] }, { "choose": { "walk": true } },
        { "open": town["bob"] }, { "choose": { "action_type": "give", "about": town["scarf"] } },
        { "open": "self" }, { "choose": { "action_type": "eat", "about": town["apple"] } }
    ])
}

/// How many of `item` a `holdings` panel's payload says are held.
fn held(record: &Value, item: &str) -> u64 {
    panel(record, "holdings")
        .and_then(|h| h["payload"]["held"].as_array().cloned())
        .unwrap_or_default()
        .iter()
        .filter(|h| id(&h["item"]) == item)
        .map(|h| h["count"].as_f64().expect("a count") as u64)
        .sum()
}

/// The requests a run sent, read through the contract.
fn transcript(lines: &[String]) -> Vec<ActionRequest<WirePayload>> {
    tagged(lines, "REQUEST ")
        .into_iter()
        .map(|r| serde_json::from_value(r["request"].clone()).expect("a legal ActionRequest"))
        .collect()
}

/// AC-I2 (items change hands, checked against the save) and AC-I8 (the same steps with no pack and
/// with no wording send the same requests, by the server's semantic core).
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn items_change_hands_whatever_draws_them() {
    let town = ids(MARKET_TOWN);
    let (visitor, bob) = (town["visitor"].clone(), town["bob"].clone());
    let (coffee, scarf, apple) = (
        town["coffee"].clone(),
        town["scarf"].clone(),
        town["apple"].clone(),
    );
    let mut runs = Vec::new();
    for presentation in ["", "--presentation=none", "--no-wording"] {
        let save = SaveDir::new(&format!("2d-i2{}", presentation.replace(['-', '='], "")));
        let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save)), None).await;
        let extra: Vec<&str> = [presentation]
            .into_iter()
            .filter(|p| !p.is_empty())
            .collect();
        let (status, lines) = play(
            world.address,
            "visitor",
            "2d-i2-steps",
            &item_steps(&town),
            &extra,
        )
        .await;
        assert!(
            status.success() && passed(&lines),
            "{presentation}: the drive passes"
        );
        world.kill();
        runs.push((presentation, lines, facts(&save)));
    }
    let (_, lines, facts) = &runs[0];

    // The save: one payment of the listed price, the coffee in and used up, the apple used up, the
    // scarf handed to Bob.
    let first = tagged(lines, "PANELS ").into_iter().next().expect("panels");
    let price = panel(&first, "shop").expect("the café's listing")["payload"]["listed"]
        .as_array()
        .expect("listed")
        .iter()
        .find(|l| id(&l["item"]) == coffee)
        .expect("coffee is listed")["price"]
        .as_f64()
        .expect("a price") as u64;
    let paid: Vec<u64> = facts
        .iter()
        .filter(|(t, p)| t == "money-transferred" && id(&p["from"]) == visitor)
        .map(|(_, p)| p["amount"].as_u64().expect("an amount"))
        .collect();
    assert_eq!(paid, [price], "one payment of the listed price");
    let moved = |item: &str, from: Option<&str>, to: &str| {
        facts.iter().any(|(t, p)| {
            t == "items-transferred"
                && id(&p["item"]) == item
                && id(&p["to"]) == to
                && from.is_none_or(|f| id(&p["from"]) == f)
                && p["count"] == 1
        })
    };
    let used = |item: &str| {
        facts.iter().any(|(t, p)| {
            t == "items-consumed" && id(&p["holder"]) == visitor && id(&p["item"]) == item
        })
    };
    assert!(
        moved(&coffee, None, &visitor),
        "the coffee came to the visitor"
    );
    assert!(
        used(&coffee) && used(&apple),
        "the coffee and the apple were used up"
    );
    assert!(moved(&scarf, Some(&visitor), &bob), "the scarf went to Bob");

    // The client: the wallet fell by the price; the holdings followed each step within 2 s.
    let balance = |record: &Value| {
        panel(record, "wallet").expect("wallet")["payload"]["balance"]
            .as_f64()
            .expect("a balance") as u64
    };
    assert_eq!(
        balance(&first) - balance(&step_done(lines, 1)),
        price,
        "the wallet fell by the price"
    );
    for (step, item, count) in [
        (1, &coffee, 1),
        (3, &coffee, 0),
        (7, &scarf, 0),
        (9, &apple, 0),
    ] {
        let done = step_done(lines, step);
        assert!(
            done["after_ms"].as_u64() <= Some(2000),
            "step {step} followed within 2 s: {done}"
        );
        assert_eq!(
            held(&done, item),
            count,
            "after step {step}, {item} × {count}"
        );
    }
    // Bob's menu, before the give: one give per kind then held, in the frame's order, complete.
    let before_give = &lines[..step_line(lines, 7)];
    let bob_menu = menus(before_give, &bob).pop().expect("Bob's menu was open");
    let gives: Vec<&Value> = bob_menu["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .filter(|e| e["action_type"] == "give")
        .collect();
    let offered: Vec<String> = bob_menu["offered"]
        .as_array()
        .expect("offered")
        .iter()
        .filter(|o| o["action_type"] == "give")
        .map(|o| id(&o["payload"]["item"]))
        .collect();
    assert_eq!(gives.len(), 2, "apple and scarf: {bob_menu}");
    assert!(gives.iter().all(|g| g["complete"] == true));
    assert_eq!(
        gives
            .iter()
            .map(|g| g["about"].as_str().expect("about").to_owned())
            .collect::<Vec<_>>(),
        offered,
        "in the frame's order"
    );

    // AC-I8: the same requests, by semantic core, with no pack and with no wording.
    let reference = transcript(&runs[0].1);
    for (presentation, lines, _) in &runs[1..] {
        let other = transcript(lines);
        assert_eq!(
            other.len(),
            reference.len(),
            "{presentation}: as many requests"
        );
        for (i, (a, b)) in reference.iter().zip(&other).enumerate() {
            assert!(
                differing_fields(a, b).is_empty(),
                "{presentation} request {i} differs: {:?}",
                differing_fields(a, b)
            );
        }
    }
}

/// AC-I6 — a pack removed: with `item-transfer` left out of the world, Bob's menu offers no `give` and
/// every other entry is as it was; one's own menu is unchanged. The client is the same in both runs.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn a_removed_pack_offers_no_give() {
    let town = ids(MARKET_TOWN);
    let bob = town["bob"].clone();
    let scratch = mineworld_test_support::scratch!(empty "2d-i6-world");
    let without = scratch.join("market-town");
    copy_dir(Path::new(MARKET_TOWN), &without);
    let declared = std::fs::read_to_string(without.join("world.yaml")).expect("world.yaml");
    let removed = declared.replacen("\n  - item-transfer\n", "\n", 1);
    assert_ne!(removed, declared, "item-transfer was declared");
    std::fs::write(without.join("world.yaml"), removed).expect("written");
    let steps = json!([
        { "open": "self" }, { "open": bob }, { "choose": { "walk": true } }, { "open": bob }, { "sleep": 1 }
    ]);
    let mut seen = Vec::new();
    for world_dir in [
        MARKET_TOWN.to_owned(),
        without.to_str().expect("UTF-8").to_owned(),
    ] {
        let save = SaveDir::new("2d-i6");
        let mut world = World::start(&owned(&hosted(&world_dir, &save)), None).await;
        let (status, lines) = play(world.address, "visitor", "2d-i6-steps", &steps, &[]).await;
        assert!(
            status.success() && passed(&lines),
            "{world_dir}: the drive passes"
        );
        world.kill();
        let shape = |menu: Value| -> Vec<Value> {
            menu["entries"]
                .as_array()
                .expect("entries")
                .iter()
                .map(|e| {
                    json!([
                        e["action_type"],
                        e["about"],
                        e["label"],
                        e["available"],
                        e["enabled"]
                    ])
                })
                .collect()
        };
        let own = shape(menus(&lines, &town["visitor"]).remove(0));
        let near = shape(menus(&lines, &bob).pop().expect("Bob's menu"));
        seen.push((own, near));
    }
    let (with_own, with_near) = &seen[0];
    let (without_own, without_near) = &seen[1];
    assert_eq!(without_own, with_own, "one's own menu is unchanged");
    assert!(
        with_near.iter().any(|e| e[0] == "give"),
        "with the pack, Bob is offered gives"
    );
    let expected: Vec<&Value> = with_near.iter().filter(|e| e[0] != "give").collect();
    assert_eq!(
        without_near.iter().collect::<Vec<_>>(),
        expected,
        "no give, nothing else changed"
    );
}

/// AC-I7 — `AC-13`'s 2D half: in the world `request-3d.json` was recorded against, the 2D client's
/// `talk` differs from the 3D client's only in `actor_location`, by the server's own comparison.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn the_2d_talk_is_the_3d_talk() {
    const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
    const EVIDENCE_3D: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../clients/protocol/evidence/request-3d.json"
    );
    const SAID: &str = "hello Alice, this is the demonstration scene";
    let alice = ids(SOCIAL_CAFE)["alice"].clone();
    let save = SaveDir::new("2d-i7");
    let mut world = World::start(&owned(&hosted(SOCIAL_CAFE, &save)), None).await;
    let out = mineworld_test_support::scratch!(empty "2d-i7-requests");
    let recorded = out.join("request-2d.json");
    let requests = format!("--requests={}", recorded.to_str().expect("UTF-8"));
    let steps = json!([
        { "open": alice }, { "choose": { "walk": true } },
        { "open": alice }, { "choose": { "action_type": "talk" }, "input": SAID }
    ]);
    let (status, lines) = play(
        world.address,
        "visitor",
        "2d-i7-steps",
        &steps,
        &[&requests],
    )
    .await;
    assert!(status.success() && passed(&lines), "the drive passes");
    world.kill();

    let talk_in = |path: &Path| -> (Value, ActionRequest<WirePayload>) {
        let listed: Vec<Value> =
            serde_json::from_str(&std::fs::read_to_string(path).expect("recorded")).expect("JSON");
        let entry = listed
            .into_iter()
            .find(|e| e["request"]["action_type"] == "talk")
            .expect("a talk was recorded");
        let request =
            serde_json::from_value(entry["request"].clone()).expect("a legal ActionRequest");
        (entry, request)
    };
    let (entry_2d, talk_2d) = talk_in(&recorded);
    let (_, talk_3d) = talk_in(Path::new(EVIDENCE_3D));
    assert_eq!(entry_2d["flavour"], "2d", "demo.gd's format");
    assert_eq!(
        entry_2d["request"]["actor_location"],
        Value::Null,
        "2D reports no position"
    );
    assert_eq!(
        differing_fields(&talk_2d, &talk_3d),
        [RequestField::ActorLocation],
        "the same actor, target and payload as the 3D client's"
    );
    let answer = result_of(&lines, &entry_2d["token"]);
    assert!(
        answer["result"].get("accepted").is_some(),
        "the server accepts it: {answer}"
    );
}

/// AC-I3 — a group activity among three 2D clients: invite (a kind the server refuses, then
/// "coffee"), accept, join, leave; an invite while Bob is busy is listed unavailable, sent, and
/// rejected; Bob leaves; a second invite is declined. Checked against the save, in order.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_2d_interact -- --ignored"]
async fn three_clients_do_something_together() {
    let town = ids(MARKET_TOWN);
    let (visitor, bob, wanderer) = (
        town["visitor"].clone(),
        town["bob"].clone(),
        town["wanderer"].clone(),
    );
    let save = SaveDir::new("2d-i3");
    let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save)), None).await;
    let sync = mineworld_test_support::scratch!(empty "2d-i3-sync");
    let sync_arg = format!("--sync={}", sync.to_str().expect("UTF-8"));
    let part = |of: &str, present: bool| json!({ "await": { "component": "participation", "of": of, "present": present } });
    let invite = |kind: &str| json!({ "choose": { "action_type": "invite" }, "input": kind });
    let visitor_steps = json!([
        { "open": bob }, { "choose": { "walk": true } },
        { "open": bob }, invite("Coffee!"),
        { "open": bob }, invite("coffee"), { "mark": "invited" },
        { "after": "accepted" }, part("self", true), { "report": true },
        { "after": "joined" }, { "open": "self" }, { "choose": { "action_type": "leave-group-activity" } },
        part("self", false), { "open": bob }, invite("chat"), { "mark": "rejected" },
        { "after": "bob-left" }, part(&bob, false), { "open": bob }, invite("chat"), { "mark": "invited2" },
        { "after": "declined" }, { "sleep": 1 }
    ]);
    let offered = |action: &str| json!({ "await": { "offered": { "action_type": action, "target": visitor } } });
    let bob_steps = json!([
        { "after": "invited" }, offered("accept-invitation"), { "report": true },
        { "open": visitor }, { "choose": { "action_type": "accept-invitation" } },
        part("self", true), { "report": true }, { "mark": "accepted" },
        { "after": "rejected" }, { "open": "self" }, { "choose": { "action_type": "leave-group-activity" } },
        part("self", false), { "mark": "bob-left" },
        { "after": "invited2" }, offered("decline-invitation"),
        { "open": visitor }, { "choose": { "action_type": "decline-invitation" } }, { "mark": "declined" },
        { "sleep": 1 }
    ]);
    let wanderer_steps = json!([
        { "after": "accepted" }, part(&bob, true),
        { "open": bob }, { "choose": { "walk": true } },
        { "open": bob }, { "choose": { "action_type": "join-group-activity" } },
        part("self", true), { "report": true }, { "mark": "joined" },
        { "after": "declined" }, { "sleep": 1 }
    ]);
    let (v, _v) = start(
        world.address,
        "visitor",
        "2d-i3-visitor",
        &visitor_steps,
        &[&sync_arg],
    );
    let (b, _b) = start(world.address, "bob", "2d-i3-bob", &bob_steps, &[&sync_arg]);
    let (w, _w) = start(
        world.address,
        "wanderer",
        "2d-i3-wanderer",
        &wanderer_steps,
        &[&sync_arg],
    );
    let (v_status, v_lines) = v.finish().await;
    let (b_status, b_lines) = b.finish().await;
    let (w_status, w_lines) = w.finish().await;
    world.kill();
    for (who, status, lines) in [
        ("visitor", v_status, &v_lines),
        ("bob", b_status, &b_lines),
        ("wanderer", w_status, &w_lines),
    ] {
        assert!(status.success() && passed(lines), "{who}: the drive passes");
    }

    // The server refused the slug the client sent unchanged; the busy invite was listed unavailable,
    // sent anyway, and rejected.
    let invites: Vec<Value> = tagged(&v_lines, "REQUEST ")
        .into_iter()
        .filter(|r| r["request"]["action_type"] == "invite")
        .collect();
    let kinds: Vec<&str> = invites
        .iter()
        .map(|r| {
            r["request"]["payload"]["payload"]["kind"]
                .as_str()
                .expect("kind")
        })
        .collect();
    assert_eq!(
        kinds,
        ["Coffee!", "coffee", "chat", "chat"],
        "sent as typed"
    );
    for i in [0, 2] {
        let answer = result_of(&v_lines, &invites[i]["token"]);
        assert!(
            answer["result"].get("accepted").is_none(),
            "invite {i} is not accepted: {answer}"
        );
    }
    let busy = menus(&v_lines[..step_line(&v_lines, 15)], &bob)
        .pop()
        .expect("Bob's menu while busy");
    let entry = busy["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .find(|e| e["action_type"] == "invite")
        .cloned()
        .expect("invite listed");
    assert_eq!(entry["available"], false, "Bob is in an activity: {entry}");

    // What each client drew: Bob's invitations panel named the visitor and the kind; every client
    // drew the marker on the participants, from the disclosure.
    let invited_panel = tagged(&b_lines, "PANELS ").into_iter().any(|p| {
        panel(&p, "invitations").is_some_and(|i| {
            i["rows"].as_array().is_some_and(|rows| {
                rows.iter()
                    .any(|r| r.as_str().is_some_and(|r| r.contains("coffee")))
            })
        })
    });
    assert!(
        invited_panel,
        "Bob's invitations panel listed the coffee invitation"
    );
    let marked = |lines: &[String], person: &str| {
        tagged(lines, "SHOWN ").iter().any(|s| {
            s["people"].as_array().is_some_and(|people| {
                people
                    .iter()
                    .any(|p| p["id"] == person && p["activity"] == "coffee")
            })
        })
    };
    for (who, lines) in [
        ("visitor", &v_lines),
        ("bob", &b_lines),
        ("wanderer", &w_lines),
    ] {
        assert!(
            marked(lines, &bob) && marked(lines, &visitor),
            "{who} drew the marker on both"
        );
    }

    // The save, in order.
    let party = [visitor.as_str(), bob.as_str(), wanderer.as_str()];
    let story: Vec<String> = facts(&save)
        .iter()
        .filter_map(|(t, p)| {
            let line = match t.as_str() {
                "invited" | "invitation-accepted" | "invitation-declined" => {
                    format!(
                        "{t} {}>{} {}",
                        id(&p["inviter"]),
                        id(&p["invitee"]),
                        p["kind"].as_str()?
                    )
                }
                "group-activity-started" => format!("{t} {}", p["kind"].as_str()?),
                "joined-group-activity" | "left-group-activity" => {
                    format!("{t} {}", id(&p["person"]))
                }
                _ => return None,
            };
            let ours = t == "group-activity-started" || party.iter().any(|who| line.contains(*who));
            ours.then_some(line)
        })
        .collect();
    let expected = [
        format!("invited {visitor}>{bob} coffee"),
        format!("invitation-accepted {visitor}>{bob} coffee"),
        "group-activity-started coffee".to_owned(),
        format!("joined-group-activity {wanderer}"),
        format!("left-group-activity {visitor}"),
        format!("left-group-activity {bob}"),
        format!("invited {visitor}>{bob} chat"),
        format!("invitation-declined {visitor}>{bob} chat"),
    ];
    let mut wanted = expected.iter();
    let mut next = wanted.next();
    for line in &story {
        if Some(line) == next {
            next = wanted.next();
        }
    }
    assert!(next.is_none(), "the save tells it in order: {story:#?}");
    let invited: Vec<&String> = story.iter().filter(|l| l.starts_with("invited ")).collect();
    assert_eq!(
        invited.len(),
        2,
        "the rejected invites left no fact: {story:#?}"
    );
}
