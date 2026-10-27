//! Buying, handing over, eating and drinking in the 3D client (`clients/3d-spike`, the connected slice;
//! S14 PR 16d), run for real against real Market Towns: step-15 §22.4's O-, N-, K- and P- lines.
//!
//! Every test here starts Godot, so every test is `#[ignore]`d: `cargo test` reports them as ignored,
//! never as passed, on a machine without Godot (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_3d_market -- --ignored --test-threads=1
//! ```
//!
//! The 3D probe never names an action type, an item or a person: this file writes the steps in 13b's
//! selector format and the probe resolves each against the menu it actually built. Expected values are
//! literals of the plan, genesis facts (`ids`, the pack's files) or the raw frame the probe printed
//! beside its report — never the client's report alone — and the save's fact log is read after the
//! server stops. `AC-13` is compared by the server's own definition (`differing_fields`, I-S14-9)
//! against 13b's real 2D client, played with its own harness, unchanged.

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::collections::BTreeMap;
use std::path::Path;

use godot2d::worlds::{copy_dir, facts, hosted, id, ids, panel, step_done};
use godot2d::{MARKET_TOWN, World, tagged};
use godot3d::{evidence, menus, passed, play, whole};
use headless::Tables;
use mineworld_contracts::{ActionRequest, ActionResult, Causation, EventEnvelope};
use mineworld_persistence::{JournalEntry, Outcome, WorldInput, format};
use mineworld_server::{RequestField, WirePayload, differing_fields};
use serde_json::{Value, json};
use support::SaveDir;

type Town = BTreeMap<String, String>;

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// The shared catalog's English text of `key` (`clients/shared/settings/locale/en.po`), read here so
/// the expected words are the catalog's, not the client's.
fn english(key: &str) -> String {
    let po = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../clients/shared/settings/locale/en.po"
    ))
    .expect("the shared en.po reads");
    let mut lines = po.lines();
    while let Some(line) = lines.next() {
        if line == format!("msgid \"{key}\"") {
            let text = lines.next().expect("a msgstr");
            return text
                .trim_start_matches("msgstr \"")
                .trim_end_matches('"')
                .to_owned();
        }
    }
    panic!("{key} is not in the shared en.po")
}

/// The first entity a payload refers to by a typed reference, or "" (13b's `about`), computed here
/// from the raw offer.
fn about(offer: &Value) -> String {
    offer["payload"]
        .as_object()
        .and_then(|fields| {
            fields
                .values()
                .find(|v| v.get("entity").and_then(Value::as_str).is_some())
        })
        .map_or_else(String::new, |v| {
            v["entity"].as_str().expect("a string").to_owned()
        })
}

/// O-1 / O-2: a `MENU`'s entries are its frame's offers, one for one, in order, with the server's
/// verdict as the frame carries it.
fn assert_menu_is_offers(menu: &Value, what: &str) {
    let entries = menu["entries"].as_array().expect("entries");
    let offered = menu["offered"].as_array().expect("offered");
    assert_eq!(
        entries.len(),
        offered.len(),
        "{what}: one entry per offer: {menu}"
    );
    for (i, (entry, offer)) in entries.iter().zip(offered).enumerate() {
        assert_eq!(
            entry["action_type"], offer["action_type"],
            "{what} entry {i}"
        );
        assert_eq!(entry["target"], offer["target"], "{what} entry {i}");
        assert_eq!(
            entry["complete"],
            offer.get("payload").is_some(),
            "{what} entry {i}"
        );
        assert_eq!(entry["available"], offer["available"], "{what} entry {i}");
        assert_eq!(
            entry["reason"], offer["unavailable_reason"],
            "{what} entry {i}"
        );
        assert_eq!(entry["about"], about(offer), "{what} entry {i}");
    }
}

/// The item catalogue a report's frame disclosed: item id → name.
fn catalogue(record: &Value) -> BTreeMap<String, String> {
    record["disclosed"]["item_catalogue"]["kinds"]
        .as_array()
        .map(|kinds| {
            kinds
                .iter()
                .map(|k| {
                    (
                        id(&k["item"]),
                        k["name"].as_str().expect("a name").to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The disclosed name of a perceived entity in a report's frame.
fn disclosed_name(record: &Value, entity: &str) -> Option<String> {
    record["disclosed"]["display_names"][entity]["name"]
        .as_str()
        .map(str::to_owned)
}

/// N-1: every entry about an item names it as the frame's catalogue does; every entry with a target
/// names the target as the world does; every holdings and shop row names its item.
fn assert_named(lines: &[String]) {
    for menu in tagged(lines, "MENU ") {
        let names = catalogue(&menu);
        for entry in menu["entries"].as_array().expect("entries") {
            let label = entry["label"].as_str().expect("a label");
            let item = entry["about"].as_str().unwrap_or("");
            if !item.is_empty() {
                let name = names
                    .get(item)
                    .unwrap_or_else(|| panic!("{item} is not catalogued: {menu}"));
                assert!(label.contains(name.as_str()), "N-1: {label:?} names {name}");
            }
            if let Some(target) = entry["target"].as_str() {
                let name = disclosed_name(&menu, target).expect("the target is named");
                assert!(label.contains(name.as_str()), "N-1: {label:?} names {name}");
            }
        }
    }
    for record in tagged(lines, "PANELS ") {
        let names = catalogue(&record);
        for component in ["holdings", "shop"] {
            let Some(p) = panel(&record, component) else {
                continue;
            };
            let listing = if component == "holdings" {
                "held"
            } else {
                "listed"
            };
            let rows = p["rows"].as_array().expect("rows");
            let listed = p["payload"][listing].as_array().expect("a listing");
            assert_eq!(rows.len(), listed.len(), "one row per entry: {p}");
            for (row, entry) in rows.iter().zip(listed) {
                let name = &names[&id(&entry["item"])];
                assert!(
                    row.as_str().expect("text").contains(name.as_str()),
                    "N-1: {row} names {name}"
                );
            }
        }
    }
}

/// A whole number as the wire wrote it, whether Godot printed it as `1` or `1.0`.
fn int(value: &Value) -> u64 {
    let number = value
        .as_f64()
        .unwrap_or_else(|| panic!("a number: {value}"));
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let whole = number as u64;
    whole
}

/// Minor units as the shared trade module formats them in Market Town's declared currency (S31-e,
/// `clients/shared/TRADE.md` §3; `worlds/market-town/configure/economy.yaml`: `$`, exponent 2, before):
/// grouped in threes, two decimals, the symbol first.
fn money(minor: &Value) -> String {
    let units = minor.as_f64().expect("an amount");
    #[allow(clippy::cast_possible_truncation)]
    let units = units as i64;
    let digits = (units / 100).to_string();
    let mut grouped = String::new();
    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    format!("${grouped}.{:02}", units % 100)
}

/// N-2: no whole-word number on screen equals an id the session has seen, once the numbers the
/// screen's own data supplies are removed — money, counts, stock, declared ranges, and the clock line.
fn assert_no_ids(lines: &[String]) {
    let mut checked = 0;
    for screen in tagged(lines, "SCREEN ") {
        let seen: Vec<String> = screen["seen"]
            .as_array()
            .expect("seen")
            .iter()
            .map(id)
            .collect();
        let own = &screen["own"];
        let mut supplied: Vec<String> = Vec::new();
        let mut add = |text: String| {
            supplied.extend(
                text.split(|c: char| !c.is_ascii_digit())
                    .filter(|t| !t.is_empty())
                    .map(str::to_owned),
            );
        };
        if !own["wallet"]["balance"].is_null() {
            add(money(&own["wallet"]["balance"]));
        }
        for held in own["holdings"]["held"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            add(int(&held["count"]).to_string());
        }
        for listed in own["shop"]["listed"]
            .as_array()
            .cloned()
            .unwrap_or_default()
        {
            add(money(&listed["price"]));
            add(int(&listed["in_stock"]).to_string());
        }
        for offer in own["offered"].as_array().cloned().unwrap_or_default() {
            if let Some(range) = offer["requirement"]["within_range"].as_f64() {
                add(format!("{}", range / 1000.0));
            }
        }
        for text in screen["texts"].as_array().expect("texts") {
            let node = text["node"].as_str().expect("a node");
            let words = text["text"].as_str().expect("text");
            if node.ends_with("/Clock") {
                continue; // the world's day and time, supplied by the frame's `at`
            }
            for token in words
                .split(|c: char| !c.is_ascii_digit())
                .filter(|t| !t.is_empty())
            {
                if supplied.iter().any(|s| s == token) {
                    continue;
                }
                assert!(
                    !seen.iter().any(|s| s == token),
                    "N-2: {node} shows {words:?}, and {token} is an id this session has seen"
                );
            }
            checked += 1;
        }
    }
    assert!(checked > 0, "N-2 read no text");
}

/// The requests a run sent, read through the contract.
fn transcript(lines: &[String]) -> Vec<(Value, ActionRequest<WirePayload>)> {
    tagged(lines, "REQUEST ")
        .into_iter()
        .map(|r| {
            let request =
                serde_json::from_value(whole(&r["request"])).expect("a legal ActionRequest");
            (r, request)
        })
        .collect()
}

/// The non-`move` requests of a run, by the server's own reading.
fn interactions(lines: &[String]) -> Vec<(Value, ActionRequest<WirePayload>)> {
    transcript(lines)
        .into_iter()
        .filter(|(r, _)| r["request"]["action_type"] != "move")
        .collect()
}

/// The answer to the request sent with `token`.
fn result_of(lines: &[String], token: &Value) -> Value {
    tagged(lines, "RESULT ")
        .into_iter()
        .find(|r| r["token"] == *token)
        .unwrap_or_else(|| panic!("request {token} was never answered"))
}

/// The toast shown with the answer to `token`: the `TOAST` printed just before its `RESULT` (the slice
/// words the answer as it arrives; the probe reports the answer after).
fn toast_of(lines: &[String], token: &Value) -> Value {
    let at = lines
        .iter()
        .position(|line| {
            line.strip_prefix("RESULT ")
                .and_then(|rest| serde_json::from_str::<Value>(rest).ok())
                .is_some_and(|r| r["token"] == *token)
        })
        .unwrap_or_else(|| panic!("request {token} was never answered"));
    lines[..at]
        .iter()
        .rev()
        .find_map(|line| line.strip_prefix("TOAST "))
        .map(|rest| serde_json::from_str(rest).expect("JSON"))
        .unwrap_or_else(|| panic!("no toast before the answer to {token}"))
}

/// The visitor's interactions in a save, in the order the world received them: each request's action
/// type, the kind of its answer, and the fact types it caused (by `caused_by`), sorted.
fn caused(save: &SaveDir, actor: &str) -> Vec<(String, String, Vec<String>)> {
    let tables = Tables::read(Path::new(save.path()));
    let mut by_action: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for (_, bytes) in &tables.facts {
        let fact: EventEnvelope = format::decode(bytes, "fact").expect("a fact");
        if let Causation::Action(action) = fact.caused_by() {
            by_action
                .entry(action.raw())
                .or_default()
                .push(fact.event_type().as_str().to_owned());
        }
    }
    let mut out = Vec::new();
    for (_, bytes) in &tables.journal {
        let entry: JournalEntry = format::decode(bytes, "journal entry").expect("an entry");
        let WorldInput::Dispatch { intent, .. } = &entry.input else {
            continue;
        };
        if intent.actor().raw().to_string() != actor || intent.action_type().as_str() == "move" {
            continue;
        }
        let kind = match &entry.outcome {
            Outcome::Dispatched(ActionResult::Accepted { .. }) => "accepted",
            Outcome::Dispatched(ActionResult::Rejected(_)) => "rejected",
            _ => "other",
        };
        let mut types = by_action
            .remove(&intent.action_id().raw())
            .unwrap_or_default();
        types.sort();
        out.push((
            intent.action_type().as_str().to_owned(),
            kind.to_owned(),
            types,
        ));
    }
    out
}

/// AC-I2's steps (13b's `item_steps`, the same five choices): buy a coffee and drink it, walk to Bob
/// and give him the scarf, eat the apple.
fn item_steps(town: &Town) -> Value {
    json!([
        { "open": "self" }, { "choose": { "action_type": "buy", "about": town["coffee"] } },
        { "open": "self" }, { "choose": { "action_type": "drink", "about": town["coffee"] } },
        { "open": town["bob"] }, { "choose": { "walk": true } },
        { "open": town["bob"] }, { "choose": { "action_type": "give", "about": town["scarf"] } },
        { "open": "self" }, { "choose": { "action_type": "eat", "about": town["apple"] } }
    ])
}

/// The same steps, with a `report` at the end so the last `SCREEN` is taken after the eat.
fn item_steps_3d(town: &Town) -> Value {
    let mut steps = item_steps(town);
    steps
        .as_array_mut()
        .expect("an array")
        .push(json!({ "report": true }));
    steps
}

// ── O-1 … O-4, O-6, N-1, N-2, K-1: from the café's door ────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_3d_market -- --ignored"]
async fn the_menus_are_the_offers_from_the_door() {
    let town = ids(MARKET_TOWN);
    let (visitor, bob) = (town["visitor"].clone(), town["bob"].clone());
    let save = SaveDir::new("3d-market-door");
    let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save)), None).await;
    let steps = json!([
        { "open": "self" }, { "gate": true },
        { "open": bob }, { "choose": { "action_type": "give" } },
        { "open": bob }, { "choose": { "walk": true } },
        { "open": bob }, { "press_disabled": true }
    ]);
    let (status, lines) = play(
        world.address,
        "visitor",
        "3d-market-door-steps",
        &steps,
        &[],
    );
    world.kill();
    assert!(status.success() && passed(&lines), "the probe passes");

    // O-1: your menu is the target-less offers.
    let own = &menus(&lines, &visitor)[0];
    assert_menu_is_offers(own, "O-1");
    let entries = own["entries"].as_array().expect("entries");
    let of = |t: &str| entries.iter().filter(|e| e["action_type"] == t).count();
    assert_eq!(of("buy"), 6, "O-1: one buy per priced kind: {own}");
    assert!(
        entries
            .iter()
            .filter(|e| e["action_type"] == "buy")
            .all(|e| e["available"] == true && e["enabled"] == true)
    );
    assert_eq!(of("eat"), 1, "O-1: the apple");
    assert_eq!(
        entries
            .iter()
            .find(|e| e["action_type"] == "eat")
            .expect("eat")["about"],
        town["apple"]
    );
    let walk = entries
        .iter()
        .find(|e| e["action_type"] == "move")
        .expect("O-1: move is listed");
    assert_eq!(walk["kind"], "body");
    assert_eq!(
        walk["enabled"], false,
        "O-1: move is never chosen from a menu"
    );

    // K-1: the panels beside it are the disclosure.
    let panels = tagged(&lines, "PANELS ")
        .into_iter()
        .next()
        .expect("PANELS");
    let wallet = panel(&panels, "wallet").expect("the wallet panel");
    assert_eq!(
        wallet["rows"][0],
        money(&wallet["payload"]["balance"]),
        "K-1 money"
    );
    let names = catalogue(&panels);
    let holdings = panel(&panels, "holdings").expect("the holdings panel");
    for (row, held) in holdings["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .zip(holdings["payload"]["held"].as_array().expect("held"))
    {
        let row = row.as_str().expect("text");
        assert!(
            row.contains(names[&id(&held["item"])].as_str())
                && row.ends_with(&format!(" × {}", int(&held["count"]))),
            "K-1 carrying: {row} for {held}"
        );
    }
    let shop = panel(&panels, "shop").expect("the shop panel");
    for (row, listed) in shop["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .zip(shop["payload"]["listed"].as_array().expect("listed"))
    {
        let row = row.as_str().expect("text");
        assert!(
            row.contains(&money(&listed["price"]))
                && row.contains(&format!("{} left", int(&listed["in_stock"]))),
            "K-1 for sale: {row} for {listed}"
        );
    }

    // O-4: a menu holds the game.
    let gate = evidence(&lines, "gate");
    assert_eq!(gate["requests"], 0, "O-4: {gate}");
    assert!(
        gate["moved_m"].as_f64().expect("metres") < 0.01,
        "O-4: {gate}"
    );
    assert!(
        gate["yaw_deg"].as_f64().expect("degrees") < 0.1,
        "O-4: {gate}"
    );
    assert_eq!(
        gate["after_first_esc"],
        json!({"menu_open": false, "settings_open": false}),
        "O-4"
    );
    assert_eq!(
        gate["after_second_esc"],
        json!({"settings_open": true}),
        "O-4"
    );

    // O-2: Bob's menu from the door is his offers: two gives, chosen regardless of the verdict.
    let far = &menus(&lines, &bob)[0];
    assert_menu_is_offers(far, "O-2");
    let gives: Vec<&Value> = far["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .filter(|e| e["action_type"] == "give")
        .collect();
    assert_eq!(gives.len(), 2, "O-2: apple and scarf: {far}");
    let needs = english("ui.needs-range").replace("{metres}", "3");
    for give in &gives {
        assert_eq!(give["complete"], true);
        assert_eq!(give["available"], false, "O-2: from the door");
        assert_eq!(give["reason"], "too_far_away");
        assert!(
            give["label"].as_str().expect("a label").contains(&needs),
            "O-2: {give}"
        );
    }
    let offered_items: Vec<String> = far["offered"]
        .as_array()
        .expect("offered")
        .iter()
        .filter(|o| o["action_type"] == "give")
        .map(about)
        .collect();
    assert_eq!(
        gives
            .iter()
            .map(|g| g["about"].as_str().expect("about").to_owned())
            .collect::<Vec<_>>(),
        offered_items,
        "O-2: in the frame's order"
    );
    let talk = far["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .find(|e| e["action_type"] == "talk")
        .expect("O-2: talk is listed");
    assert_eq!(talk["kind"], "composed");
    for entry in far["entries"].as_array().expect("entries") {
        if entry["complete"] == false
            && entry["action_type"] != "talk"
            && entry["action_type"] != "move"
        {
            assert_eq!(
                entry["enabled"], false,
                "O-2: not composed in 3D, so disabled: {entry}"
            );
        }
    }

    // O-3: chosen regardless — sent exactly as offered, rejected by the world, said in words.
    let first_give = far["offered"]
        .as_array()
        .expect("offered")
        .iter()
        .find(|o| o["action_type"] == "give")
        .expect("a give");
    let sent = interactions(&lines);
    let (give_line, _) = sent.first().expect("O-3: the give was sent");
    assert_eq!(give_line["request"]["action_type"], "give");
    assert_eq!(give_line["request"]["target"], first_give["target"]);
    assert_eq!(
        whole(&give_line["request"]["payload"]["payload"]),
        whole(&first_give["payload"])
    );
    assert_eq!(
        result_of(&lines, &give_line["token"])["result"]["rejected"],
        "too_far_away"
    );
    let toast = toast_of(&lines, &give_line["token"]);
    assert_eq!(toast["key"], "ui.result.rejected", "O-3: {toast}");
    assert!(
        toast["text"]
            .as_str()
            .expect("text")
            .contains(&english("reason.too_far_away")),
        "O-3: the reason's words: {toast}"
    );

    // O-6: what the client does not compose is never sent.
    let pressed = evidence(&lines, "press_disabled");
    assert!(
        pressed["pressed"].as_u64() > Some(0),
        "O-6 pressed something: {pressed}"
    );
    assert_eq!(pressed["requests"], 0, "O-6: {pressed}");
    assert_eq!(
        sent.len(),
        1,
        "O-6: the rejected give is the only interaction sent"
    );

    // N-2 (no id), then N-1 (named by the world).
    assert_no_ids(&lines);
    assert_named(&lines);

    // O-3's oracle: the save holds no hand-over from the visitor.
    let history = facts(&save);
    assert!(
        !history
            .iter()
            .any(|(t, p)| t == "items-transferred" && id(&p["from"]) == visitor),
        "O-3: nothing was handed over"
    );
}

// ── O-5: a poor visitor ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_3d_market -- --ignored"]
async fn a_poor_visitor_still_sees_every_buy() {
    let town = ids(MARKET_TOWN);
    let visitor = town["visitor"].clone();
    let scratch = mineworld_test_support::scratch!(empty "3d-o5-world");
    let poor = scratch.join("market-town");
    copy_dir(Path::new(MARKET_TOWN), &poor);
    let person = poor.join("people/visitor.yaml");
    let authored = std::fs::read_to_string(&person).expect("visitor.yaml");
    let changed = authored.replacen("economy: { wallet: 200000 }", "economy: { wallet: 250 }", 1);
    assert_ne!(changed, authored, "the visitor's wallet was authored");
    std::fs::write(&person, changed).expect("written");
    let world_dir = poor.to_str().expect("UTF-8").to_owned();
    let save = SaveDir::new("3d-market-poor");
    let mut world = World::start(&owned(&hosted(&world_dir, &save)), None).await;
    let steps = json!([
        { "open": "self" }, { "choose": { "action_type": "buy", "about": town["coffee"] } }
    ]);
    let (status, lines) = play(
        world.address,
        "visitor",
        "3d-market-poor-steps",
        &steps,
        &[],
    );
    world.kill();
    assert!(status.success() && passed(&lines), "the probe passes");

    let own = &menus(&lines, &visitor)[0];
    assert_menu_is_offers(own, "O-5");
    let buys: Vec<&Value> = own["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .filter(|e| e["action_type"] == "buy")
        .collect();
    assert_eq!(buys.len(), 6, "O-5: every priced kind is listed: {own}");
    for buy in &buys {
        let item = buy["about"].as_str().expect("about");
        let affordable = [&town["tea"], &town["croissant"]].contains(&&item.to_owned());
        assert_eq!(buy["available"], affordable, "O-5: {buy}");
        assert_eq!(buy["enabled"], true, "O-5: listed and choosable: {buy}");
    }
    let sent = interactions(&lines);
    assert_eq!(sent.len(), 1, "O-5: the coffee was sent");
    assert_eq!(sent[0].0["request"]["action_type"], "buy");
    assert!(
        result_of(&lines, &sent[0].0["token"])["result"]
            .get("rejected")
            .is_some(),
        "O-5: answered rejected"
    );
    assert!(
        !facts(&save)
            .iter()
            .any(|(t, p)| t == "money-transferred" && id(&p["from"]) == visitor),
        "O-5: no money left the visitor"
    );
}

// ── P-1 … P-3, K-2: the same requests as the 2D client (`AC-13`) ───────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_3d_market -- --ignored"]
async fn the_3d_market_asks_what_the_2d_market_asks() {
    let town = ids(MARKET_TOWN);
    let (visitor, bob) = (town["visitor"].clone(), town["bob"].clone());
    let (coffee, scarf, apple) = (
        town["coffee"].clone(),
        town["scarf"].clone(),
        town["apple"].clone(),
    );

    // 13b's 2D client, its harness and its steps, unchanged, on a fresh Market Town.
    let save_2d = SaveDir::new("3d-p1-2d");
    let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save_2d)), None).await;
    let (status, lines_2d) = godot2d::worlds::play(
        world.address,
        "visitor",
        "3d-p1-2d-steps",
        &item_steps(&town),
        &[],
    )
    .await;
    world.kill();
    assert!(
        status.success() && godot2d::passed(&lines_2d),
        "the 2D drive passes"
    );

    // The 3D client, the same steps, on another fresh Market Town; then in Chinese and with no wording.
    let zh = SaveDir::new("3d-p3-settings");
    std::fs::create_dir_all(zh.path()).expect("a folder");
    let zh_file = Path::new(zh.path()).join("zh.cfg");
    std::fs::write(
        &zh_file,
        "[meta]\nversion=1\n[general]\nlanguage=\"zh_Hans\"\n",
    )
    .expect("the settings file");
    let mut runs = Vec::new();
    for extra in [
        String::new(),
        format!("--settings={}", zh_file.display()),
        "--no-wording".to_owned(),
    ] {
        let save = SaveDir::new(&format!("3d-p1-3d-{}", runs.len()));
        let mut world = World::start(&owned(&hosted(MARKET_TOWN, &save)), None).await;
        let arguments: Vec<&str> = [extra.as_str()]
            .into_iter()
            .filter(|a| !a.is_empty())
            .collect();
        let (status, lines) = play(
            world.address,
            "visitor",
            "3d-p1-3d-steps",
            &item_steps_3d(&town),
            &arguments,
        );
        world.kill();
        assert!(
            status.success() && passed(&lines),
            "{extra}: the 3D probe passes"
        );
        runs.push((extra, lines, save));
    }
    let (_, lines_3d, save_3d) = &runs[0];

    // P-1: the same four requests, by the server's definition; 2D reports no position, 3D its own.
    let two = interactions(&lines_2d);
    let three = interactions(lines_3d);
    let types = |run: &[(Value, ActionRequest<WirePayload>)]| -> Vec<String> {
        run.iter()
            .map(|(r, _)| {
                r["request"]["action_type"]
                    .as_str()
                    .expect("a type")
                    .to_owned()
            })
            .collect()
    };
    assert_eq!(
        types(&two),
        ["buy", "drink", "give", "eat"],
        "P-1: the 2D requests"
    );
    assert_eq!(
        types(&three),
        ["buy", "drink", "give", "eat"],
        "P-1: the 3D requests"
    );
    for ((line_2d, a), (line_3d, b)) in two.iter().zip(&three) {
        let differing = differing_fields(a, b);
        assert!(
            differing.iter().all(|f| *f == RequestField::ActorLocation),
            "P-1: {} differs in {differing:?}",
            line_2d["request"]["action_type"]
        );
        assert_eq!(
            line_2d["request"]["actor_location"],
            Value::Null,
            "P-1: 2D reports no position"
        );
        let place = &line_3d["request"]["actor_location"]["place"]["entity"];
        assert_eq!(
            place,
            &json!(town["cafe"]),
            "P-1: 3D reports where it stands: {line_3d}"
        );
        assert!(
            result_of(&lines_2d, &line_2d["token"])["result"]
                .get("accepted")
                .is_some()
                && result_of(lines_3d, &line_3d["token"])["result"]
                    .get("accepted")
                    .is_some(),
            "P-1: both accepted"
        );
    }
    let caused_2d = caused(&save_2d, &visitor);
    let caused_3d = caused(save_3d, &visitor);
    assert_eq!(
        caused_2d, caused_3d,
        "P-1: the same answers and the same facts, request by request"
    );

    // P-2: the same menus, field for field, apart from 2D's walk entry and both clients' move.
    let core = |menu: &Value| -> Vec<Value> {
        menu["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .filter(|e| e["kind"] != "walk" && e["action_type"] != "move")
            .map(|e| {
                json!([
                    e["action_type"],
                    e["target"],
                    e["about"],
                    e["complete"],
                    e["available"],
                    e["reason"]
                ])
            })
            .collect()
    };
    let at_step = |lines: &[String], step: usize, subject: &str| -> Value {
        let begun = godot2d::worlds::step_line(lines, step);
        let next = godot2d::worlds::step_line(lines, step + 1);
        godot2d::worlds::menus(&lines[begun..next], subject)
            .pop()
            .unwrap_or_else(|| panic!("no menu of {subject} at step {step}"))
    };
    for (step, subject, what) in [
        (0, &visitor, "self at genesis"),
        (2, &visitor, "self after the buy"),
        (6, &bob, "Bob near"),
    ] {
        assert_eq!(
            core(&at_step(lines_3d, step, subject)),
            core(&at_step(&lines_2d, step, subject)),
            "P-2: {what}"
        );
    }

    // K-2: the panels follow each accepted step within 2 s, as the save says.
    let first = tagged(lines_3d, "PANELS ")
        .into_iter()
        .next()
        .expect("PANELS");
    let price = panel(&first, "shop").expect("the café's listing")["payload"]["listed"]
        .as_array()
        .expect("listed")
        .iter()
        .find(|l| id(&l["item"]) == coffee)
        .expect("coffee is listed")["price"]
        .as_f64()
        .expect("a price");
    let history = facts(save_3d);
    let paid: Vec<f64> = history
        .iter()
        .filter(|(t, p)| t == "money-transferred" && id(&p["from"]) == visitor)
        .map(|(_, p)| p["amount"].as_f64().expect("an amount"))
        .collect();
    assert_eq!(paid, [price], "K-2: one payment of the listed price");
    let balance = |record: &Value| {
        panel(record, "wallet").expect("wallet")["payload"]["balance"]
            .as_f64()
            .expect("a balance")
    };
    let after_buy = step_done(lines_3d, 1);
    assert!(
        (balance(&first) - balance(&after_buy) - price).abs() < 0.5,
        "K-2: the wallet fell by the price"
    );
    assert_eq!(
        panel(&after_buy, "wallet").expect("wallet")["rows"][0],
        money(&json!(balance(&after_buy))),
        "K-2: the money row follows"
    );
    let names = catalogue(&first);
    for (step, item, count) in [
        (1, &coffee, 1),
        (3, &coffee, 0),
        (7, &scarf, 0),
        (9, &apple, 0),
    ] {
        let done = step_done(lines_3d, step);
        assert!(
            done["after_ms"].as_u64() <= Some(2000),
            "K-2: step {step} followed within 2 s: {done}"
        );
        let holdings = panel(&done, "holdings").expect("holdings");
        let held: u64 = holdings["payload"]["held"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter(|h| id(&h["item"]) == *item)
            .map(|h| int(&h["count"]))
            .sum();
        assert_eq!(held, count, "K-2: after step {step}, {item} × {count}");
        let rows = holdings["rows"].as_array().expect("rows");
        let shown = rows
            .iter()
            .filter(|r| {
                r.as_str()
                    .expect("text")
                    .contains(names[item.as_str()].as_str())
            })
            .count();
        assert_eq!(
            shown,
            usize::from(count > 0),
            "K-2: the Carrying rows follow at step {step}: {rows:?}"
        );
    }
    assert!(
        history.iter().any(|(t, p)| t == "items-transferred"
            && id(&p["item"]) == scarf
            && id(&p["from"]) == visitor
            && id(&p["to"]) == bob),
        "K-2: the scarf went to Bob"
    );

    // N-1 and N-2 over the whole run.
    assert_no_ids(lines_3d);
    assert_named(lines_3d);

    // P-3: words decide nothing — in Chinese and with no wording, the same requests, pair for pair.
    let reference = interactions(lines_3d);
    for (extra, lines, _) in &runs[1..] {
        let other = interactions(lines);
        assert_eq!(
            other.len(),
            reference.len(),
            "P-3 {extra}: as many requests"
        );
        for ((_, a), (_, b)) in reference.iter().zip(&other) {
            let differing = differing_fields(a, b);
            assert!(
                differing.iter().all(|f| *f == RequestField::ActorLocation),
                "P-3 {extra}: {differing:?}"
            );
        }
    }
}
