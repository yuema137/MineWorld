//! Trading in both reference clients (S31-e §8): money in the world's own currency, the sale panel, the
//! stall, price tags and the notice board, driven for real on generated copies of Market Town and checked
//! against the save's facts or the frame the client was given — never against the client's report alone.
//!
//! Every test starts Godot, so every test is `#[ignore]`d (a test not run is not a pass). Run them with
//!
//! ```text
//! cargo test -p mineworld-cli --test client_trade -- --ignored --test-threads=1
//! ```
//!
//! The variants (§5.12) are written into scratch directories by this file; nothing under `worlds/` changes:
//!
//! ```text
//! V-usd    Market Town as it is: US dollar, exponent 2, before
//! V-crown  { crown, cr, 0, after }; Grace holds two flowers and asks 450; Felix keeps the corner store;
//!          notices on, with Hana's news and Carol's prices topics, the visitor subscribed to both
//! V-dinar  { dinar, KD, 3, before }; Ivan holds an umbrella and asks 12 500 for it
//! V-none   Social Café: no economy, no notices
//! ```
//!
//! The steps name ids the test resolves from genesis (`ids`); no client script names an item, a person or
//! a topic, and no selector matches a label's words.

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::path::{Path, PathBuf};

use godot2d::worlds::{copy_dir, facts, hosted, id, ids, play, step_done};
use godot2d::{MARKET_TOWN, StubWorld, World, offering, record, tagged};
use serde_json::{Value, json};
use support::SaveDir;

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");

/// The shared catalog's English text of `key`, so expected words are the catalog's, not the client's.
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

/// `text` with `old` replaced by `new`, where `old` occurs exactly once.
fn replaced_once(text: &str, old: &str, new: &str) -> String {
    assert_eq!(text.matches(old).count(), 1, "exactly one {old:?}");
    text.replacen(old, new, 1)
}

fn edit(file: &Path, old: &str, new: &str) {
    let text = std::fs::read_to_string(file).expect("a pack file reads");
    std::fs::write(file, replaced_once(&text, old, new)).expect("written");
}

const USD: &str =
    "currency:\n  name: US dollar\n  symbol: \"$\"\n  exponent: 2\n  symbol_at: before\n";

/// A generated variant of Market Town (§5.12), in its own scratch directory.
struct Variant {
    _scratch: mineworld_test_support::Scratch,
    dir: PathBuf,
}

impl Variant {
    fn path(&self) -> String {
        self.dir.to_str().expect("UTF-8").to_owned()
    }

    fn usd(name: &str) -> Self {
        let scratch = mineworld_test_support::scratch!(empty name);
        let dir = scratch.join("market-town");
        copy_dir(Path::new(MARKET_TOWN), &dir);
        Self {
            _scratch: scratch,
            dir,
        }
    }

    /// V-crown: another currency, a person seller, a keeper, and notices on other topic names.
    fn crown(name: &str) -> Self {
        let v = Self::usd(name);
        edit(
            &v.dir.join("configure/economy.yaml"),
            USD,
            "currency:\n  name: crown\n  symbol: cr\n  exponent: 0\n  symbol_at: after\n",
        );
        let grace = v.dir.join("people/grace.yaml");
        edit(
            &grace,
            "holdings: { candle: 1, soap: 1, umbrella: 1 }",
            "holdings: { candle: 1, soap: 1, umbrella: 1, flowers: 2 }",
        );
        edit(
            &grace,
            "economy: { wallet: 200000 }",
            "economy: { wallet: 200000, asks: { flowers: 450 } }",
        );
        // Grace waits beside the visitor in the café, so the drive need not walk.
        edit(
            &grace,
            "  place: workplace\n  position:\n    x: 3000\n    y: -3000",
            "  place: cafe\n  position:\n    x: 2610\n    y: 600",
        );
        edit(
            &v.dir.join("organizations/corner-store.yaml"),
            "    at: store\n",
            "    at: store\n    keepers: [felix]\n",
        );
        v.notices();
        v
    }

    /// V-dinar: three decimals, another seller and kind.
    fn dinar(name: &str) -> Self {
        let v = Self::usd(name);
        edit(
            &v.dir.join("configure/economy.yaml"),
            USD,
            "currency:\n  name: dinar\n  symbol: KD\n  exponent: 3\n  symbol_at: before\n",
        );
        let ivan = v.dir.join("people/ivan.yaml");
        edit(
            &ivan,
            "holdings: { apple: 1, coffee: 1, mug: 1 }",
            "holdings: { apple: 1, coffee: 1, mug: 1, umbrella: 1 }",
        );
        edit(
            &ivan,
            "economy: { wallet: 200000 }",
            "economy: { wallet: 200000, asks: { umbrella: 12500 } }",
        );
        v
    }

    /// `notices` enabled after `appearance`, two topics, the visitor subscribed to both.
    fn notices(&self) {
        let manifest = self.dir.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("world.yaml");
        let enabled = replaced_once(&text, "  - appearance\n", "  - appearance\n  - notices\n");
        let configured = if enabled.contains("\nconfigure:\n") {
            replaced_once(&enabled, "\nconfigure:\n", "\nconfigure:\n  - notices\n")
        } else {
            format!("{enabled}\nconfigure:\n  - notices\n")
        };
        std::fs::write(&manifest, configured).expect("written");
        std::fs::write(
            self.dir.join("configure/notices.yaml"),
            "topics:\n  news.street:\n    body: text\n    publishers: [hana]\n    valid_for: { min: 1h, max: 7d }\n    per_day: 4\n  prices.street:\n    body: prices\n    publishers: [carol]\n    valid_for: { min: 1h, max: 7d }\n    per_day: 4\n",
        )
        .expect("notices.yaml");
        let visitor = self.dir.join("people/visitor.yaml");
        let text = std::fs::read_to_string(&visitor).expect("visitor.yaml");
        std::fs::write(
            &visitor,
            format!("{text}\nnotices: {{ subscribe: [news.street, prices.street] }}\n"),
        )
        .expect("written");
    }
}

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// The last `HUD` report.
fn hud(lines: &[String]) -> Value {
    tagged(lines, "HUD ")
        .pop()
        .unwrap_or_else(|| panic!("no HUD report:\n{}", lines.join("\n")))
}

/// The last `TRADE` report.
fn trade(lines: &[String]) -> Value {
    tagged(lines, "TRADE ")
        .pop()
        .unwrap_or_else(|| panic!("no TRADE report:\n{}", lines.join("\n")))
}

/// The row of `kind` about `item` in a `TRADE` report.
fn row<'a>(report: &'a Value, kind: &str, item: &str) -> &'a Value {
    report["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|r| r["kind"] == kind && r["item"] == item)
        .unwrap_or_else(|| panic!("no {kind} row about {item}: {report}"))
}

/// Every `sold` fact of the save, as (buyer, item, price).
fn sold(save: &SaveDir) -> Vec<(String, String, u64)> {
    facts(save)
        .into_iter()
        .filter(|(t, _)| t == "sold")
        .map(|(_, p)| {
            (
                id(&p["buyer"]),
                id(&p["item"]),
                p["price"].as_u64().expect("a price"),
            )
        })
        .collect()
}

// ── AC-e-2: the currency renames with zero client code ────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn the_money_line_is_in_the_worlds_own_currency() {
    for (variant, expected) in [
        (Variant::usd("trade-e2-usd"), "$2,000.00"),
        (Variant::crown("trade-e2-crown"), "200,000 cr"),
        (Variant::dinar("trade-e2-dinar"), "KD200.000"),
    ] {
        let save = SaveDir::new("trade-e2");
        let mut world = World::start(&owned(&hosted(&variant.path(), &save)), None).await;
        let steps = json!([{ "sleep": 1 }, { "hud": true }]);
        let (status, lines) = play(world.address, "visitor", "trade-e2-2d", &steps, &[]).await;
        assert!(status.success(), "the 2D drive ends cleanly");
        let money = hud(&lines)["money"].as_str().expect("money").to_owned();
        assert!(money.contains(expected), "2D: {money} shows {expected}");
        let (status, lines) = godot3d::play(world.address, "visitor", "trade-e2-3d", &steps, &[]);
        world.kill();
        assert!(
            status.success() && godot3d::passed(&lines),
            "the 3D probe passes"
        );
        let money = hud(&lines)["money"].as_str().expect("money").to_owned();
        assert!(money.contains(expected), "3D: {money} shows {expected}");
    }
}

// ── AC-e-3: buy at a shop ────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn buying_at_the_shop_costs_the_rows_price() {
    let town = ids(MARKET_TOWN);
    let visitor = town["visitor"].clone();
    for (variant, name) in [
        (Variant::usd("trade-e3-usd"), "usd"),
        (Variant::dinar("trade-e3-dinar"), "dinar"),
    ] {
        let save = SaveDir::new(&format!("trade-e3-{name}"));
        let mut world = World::start(&owned(&hosted(&variant.path(), &save)), None).await;
        let steps = json!([
            { "trade": "open", "of": "self" },
            { "trade": "press", "row": { "kind": "sale", "item": town["coffee"] }, "button": 0 },
            { "hud": true }
        ]);
        let (status, lines) = play(world.address, "visitor", "trade-e3-2d", &steps, &[]).await;
        world.kill();
        assert!(status.success(), "the drive ends cleanly");
        let shown = tagged(&lines, "TRADE ");
        let coffee = row(&shown[0], "sale", &town["coffee"]);
        assert_eq!(step_done(&lines, 1)["sent"], 1, "Buy sent the offer");
        let sales = sold(&save);
        assert_eq!(sales.len(), 1, "one sale: {sales:?}");
        assert_eq!(sales[0].0, visitor);
        assert_eq!(
            Some(sales[0].2),
            coffee["amount"].as_u64(),
            "the row's amount is the price paid"
        );
    }
}

// ── AC-e-4: buy from a person; a refused row stays pressable ─────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn buying_from_grace_and_a_refused_row_is_still_sent() {
    let town = ids(MARKET_TOWN);
    let grace = town["grace"].clone();
    let flowers = town["flowers"].clone();
    let variant = Variant::crown("trade-e4");
    let save = SaveDir::new("trade-e4");
    let mut world = World::start(&owned(&hosted(&variant.path(), &save)), None).await;
    let steps = json!([
        { "sleep": 1 }, { "hud": true },
        { "trade": "open", "of": grace },
        { "trade": "press", "row": { "kind": "sale", "item": flowers }, "button": 0 }
    ]);
    let (status, lines) = play(world.address, "visitor", "trade-e4-2d", &steps, &[]).await;
    world.kill();
    assert!(status.success(), "the drive ends cleanly");
    let tags = hud(&lines)["tags"].clone();
    assert!(
        tags.as_array()
            .expect("tags")
            .iter()
            .any(|t| t["subject"] == grace.as_str()
                && t["text"].as_str().is_some_and(|s| s.contains("450 cr"))),
        "Grace's tag shows her price: {tags}"
    );
    assert_eq!(
        sold(&save),
        vec![(town["visitor"].clone(), flowers.clone(), 450)],
        "one sale at her price"
    );

    // The poor visitor: the row is shown with the refusal's words, Buy still sends, nothing moves.
    let poor = Variant::crown("trade-e4-poor");
    edit(
        &poor.dir.join("people/visitor.yaml"),
        "economy: { wallet: 200000 }",
        "economy: { wallet: 100 }",
    );
    let save = SaveDir::new("trade-e4-poor");
    let mut world = World::start(&owned(&hosted(&poor.path(), &save)), None).await;
    let (status, lines) = play(world.address, "visitor", "trade-e4-poor", &steps, &[]).await;
    world.kill();
    assert!(status.success(), "the drive ends cleanly");
    let shown = trade(&lines);
    let refused = row(&shown, "sale", &flowers);
    assert_eq!(
        refused["available"], false,
        "the server refused it: {refused}"
    );
    assert!(
        refused["text"]
            .as_str()
            .is_some_and(|t| t.contains(&english("reason.insufficient-funds"))),
        "the refusal in words: {refused}"
    );
    assert_eq!(step_done(&lines, 3)["sent"], 1, "Buy was still sent");
    assert!(sold(&save).is_empty(), "nothing was sold");
}

// ── AC-e-5: the stall ────────────────────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn the_stall_lists_reprices_and_withdraws() {
    let town = ids(MARKET_TOWN);
    let scarf = town["scarf"].clone();
    let variant = Variant::crown("trade-e5");
    let save = SaveDir::new("trade-e5");
    let mut world = World::start(&owned(&hosted(&variant.path(), &save)), None).await;
    let held = json!({ "kind": "held", "item": scarf });
    let asked = json!({ "kind": "ask", "item": scarf });
    let steps = json!([
        { "trade": "open", "of": "self" },
        { "trade": "press", "row": held, "button": 0, "amount": "12" },
        { "trade": "press", "row": asked, "button": 0, "amount": "15" },
        { "trade": "press", "row": asked, "button": 0, "amount": "1.5" },
        { "trade": "press", "row": asked, "button": 0, "amount": "0" },
        { "trade": "press", "row": asked, "button": 1 }
    ]);
    let (status, lines) = play(world.address, "visitor", "trade-e5-2d", &steps, &[]).await;
    world.kill();
    assert!(status.success(), "the drive ends cleanly");
    for (step, sent) in [(1, 1), (2, 1), (3, 0), (4, 1), (5, 1)] {
        assert_eq!(step_done(&lines, step)["sent"], sent, "step {step}");
    }
    assert!(
        !tagged(&lines, "FIELD ").is_empty(),
        "1.5 is not an amount in crowns"
    );
    let history = facts(&save);
    let types: Vec<&str> = history
        .iter()
        .filter(|(t, _)| ["listed", "price-set", "withdrawn"].contains(&t.as_str()))
        .map(|(t, _)| t.as_str())
        .collect();
    assert_eq!(types, ["listed", "price-set", "withdrawn"], "{types:?}");
    let listed = history.iter().find(|(t, _)| t == "listed").expect("listed");
    assert_eq!(listed.1["price"], 12, "12 crowns are 12 minor units");
}

// ── AC-e-7, AC-e-8 (stub arm): refusals in words, and the record decides what is current ─────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn refusals_have_words_and_a_stale_notice_is_still_shown() {
    let crown = json!({ "name": "crown", "symbol": "cr", "exponent": 0, "symbol_at": "after", "references": [] });
    let codes = [
        "not-for-sale",
        "price-changed",
        "insufficient-funds",
        "out-of-stock",
        "cannot-carry",
    ];
    let offers: Vec<Value> = codes
        .iter()
        .enumerate()
        .map(|(n, code)| {
            json!({ "action_type": "buy", "target": null, "available": false,
                "unavailable_reason": { "system": { "code": code } },
                "payload": { "item": { "entity": format!("{}", 40 + n), "entity_type": "item" }, "price": 3 } })
        })
        .collect();
    let board = json!({ "notices": [{ "id": 1, "topic": "news.street", "publisher": "9",
        "body": { "text": "A notice whose time has passed." }, "issued_at": 0, "valid_until": 0 }] });
    let (address, _log) = offering(StubWorld {
        offers,
        own: vec![
            (record("9", "currency", crown), None),
            (record("9", "wallet", json!({ "balance": 5 })), None),
            (record("9", "notice-board", board), None),
        ],
        ..StubWorld::default()
    })
    .await;
    let steps = json!([{ "trade": "open", "of": "self" }, { "board": "open" }, { "hud": true }]);
    let (status, lines) = play(
        address,
        "visitor",
        "trade-e7",
        &steps,
        &["--world-key=market-town"],
    )
    .await;
    assert!(status.success(), "the drive ends cleanly");
    let shown = trade(&lines);
    for (n, code) in codes.iter().enumerate() {
        let r = row(&shown, "sale", &format!("{}", 40 + n));
        assert!(
            r["text"]
                .as_str()
                .is_some_and(|t| t.contains(&english(&format!("reason.{code}")))),
            "{code} in words: {r}"
        );
    }
    let boards = tagged(&lines, "BOARD ");
    let last = boards.last().expect("a BOARD report");
    assert_eq!(
        last["notices"].as_array().map(Vec::len),
        Some(1),
        "the record is the authority: {last}"
    );
    assert_eq!(
        hud(&lines)["money"],
        "Money: 5 cr",
        "the money line in crowns"
    );
}

// ── AC-e-11: no economy, nothing new ─────────────────────────────────────────────────────────────────

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_trade -- --ignored"]
async fn a_world_without_economy_shows_no_trade() {
    let save = SaveDir::new("trade-e11");
    let mut world = World::start(&owned(&hosted(SOCIAL_CAFE, &save)), None).await;
    let steps = json!([{ "trade": "open", "of": "self", "absent": true }, { "hud": true }]);
    let (status, lines) = play(
        world.address,
        "visitor",
        "trade-e11",
        &steps,
        &["--world-key=social-cafe"],
    )
    .await;
    world.kill();
    assert!(status.success(), "the drive ends cleanly");
    let shown = hud(&lines);
    assert_eq!(shown["money"], "", "no money line: {shown}");
    assert_eq!(shown["tags"], json!([]), "no tag");
    assert!(
        tagged(&lines, "TRADE ")
            .iter()
            .all(|t| t["rows"] == json!([])),
        "no trade rows"
    );
}
