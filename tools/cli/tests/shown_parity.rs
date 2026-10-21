//! "One world, two views" (S14 PR 16e §4.3, Q-PAR-1; `docs/MVP.md` §7.2; S33-h's `ARC-174`): for one
//! moment of one world, the real 2D client and the real 3D client report showing the same places,
//! doorways, people, loose objects and offers.
//!
//! The moment is fixed by pausing the world clock (`POST /admin/clock {"paused": true}`) so nobody
//! moves (Q-PAR-2); then the 2D client joins the seat, prints `SHOWN` and leaves, and the 3D client
//! joins the **same** seat, prints `SHOWN` and leaves. Same observer, same frozen world, so the
//! disclosure is the same, and any difference between the reports is a client's.
//!
//! Compared, by `clients/protocol/ADOPTION.md` §7: the observer's place `{place, tags}` and its
//! doorways `{from, to, here, there}` (each client learns a place where it stands, and how far beyond
//! it a client draws is presentation, so a fresh seat's own place is the common ground); people by id,
//! and by the disclosed name each client's `IDENTITY_WORLD` gives them; loose objects `{id, kind}`;
//! affordances `{subject, action_type, target, about, complete, available, reason}`, each subject's in
//! the server's order. Not compared: that section's presentation-only list (how a place is drawn, a
//! person's label, look, role, activity and drawn position, a doorway's
//! wording, a menu entry's label and `enabled`).
//!
//! The standing negative case: a client run with the harness flag `--drop-doorway=<from>,<to>` (a
//! doorway of the start place, taken from the other client's report) is told apart, by that doorway
//! and that client.
//!
//! Every test starts Godot, so every test is `#[ignore]`d and runs at the milestone:
//!
//! ```text
//! cargo test -p mineworld-cli --test shown_parity -- --ignored --test-threads=1
//! ```

mod godot2d;
mod godot3d;
mod headless;
mod support;

use std::collections::{BTreeMap, BTreeSet};

use godot2d::worlds::{hosted, id};
use godot2d::{MARKET_TOWN, World, tagged};
use godot3d::{identity, whole, world_key};
use serde_json::{Value, json};
use support::{SaveDir, admin};

const SOCIAL_CAFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/social-cafe");
const SEAT: &str = "visitor";
const TOKEN: &str = "shown-parity-admin-token";

fn owned(arguments: &[String]) -> Vec<&str> {
    arguments.iter().map(String::as_str).collect()
}

/// A local point `(x, y)` in millimetres, from either client's shape (`[x, y]` or `{x, y, …}`).
fn mm(value: &Value) -> (i64, i64) {
    let axis = |v: &Value| v.as_f64().map_or(i64::MIN, |f| f.round() as i64);
    match value {
        Value::Array(xy) if xy.len() >= 2 => (axis(&xy[0]), axis(&xy[1])),
        _ => (axis(&value["x"]), axis(&value["y"])),
    }
}

/// A doorway as compared: from, to, and its two sides' local millimetres.
type Doorway = (String, String, (i64, i64), (i64, i64));

/// One client's report at the moment, reduced to the compared keys.
#[derive(Debug, Default)]
struct Shown {
    places: BTreeSet<(String, Vec<String>)>,
    doorways: BTreeSet<Doorway>,
    people: BTreeMap<String, Value>,
    objects: BTreeSet<(String, String)>,
    affordances: BTreeMap<String, Vec<Value>>,
}

impl Shown {
    /// The last `SHOWN` and `IDENTITY_WORLD` a run printed.
    fn of(lines: &[String]) -> Self {
        let report = tagged(lines, "SHOWN ")
            .pop()
            .unwrap_or_else(|| panic!("no SHOWN line in:\n{}", lines.join("\n")));
        let names = identity(lines)["people"].clone();
        let list = |key: &str| report[key].as_array().cloned().unwrap_or_default();
        let mut shown = Self::default();
        let here = id(&report["place"]);
        // The observer's own place, by id and tags. How far beyond it a client draws is presentation:
        // the 3D scene binds the street it sees through the café's door (its tags never disclosed),
        // the 2D map draws only where the seat stands (16e ledger D-e-5). Where the place's doorways
        // lead is compared below.
        for p in list("places")
            .into_iter()
            .filter(|p| id(&p["place"]) == here)
        {
            let mut tags: Vec<String> = p["tags"]
                .as_array()
                .map(|t| t.iter().map(id).collect())
                .unwrap_or_default();
            tags.sort();
            shown.places.insert((id(&p["place"]), tags));
        }
        // The doorways of the observer's own place, as each client drew them: a client learns a place's
        // passages where it stands, and the two clients draw the other places' ways back differently
        // (2D: once per drawn anchor, with the way back through a hub doorway; 3D: every passage whose
        // two sides are bound), so only the place the seat stands in is common ground (16e ledger).
        for d in list("doorways")
            .into_iter()
            .filter(|d| id(&d["from"]) == here)
        {
            shown.doorways.insert((
                id(&d["from"]),
                id(&d["to"]),
                mm(&d["here"]),
                mm(&d["there"]),
            ));
        }
        for p in list("people") {
            let person = id(&p["id"]);
            shown.people.insert(person.clone(), names[&person].clone());
        }
        for o in list("objects") {
            shown.objects.insert((id(&o["id"]), id(&o["kind"])));
        }
        for a in list("affordances") {
            let target = if a["target"].is_null() {
                Value::Null
            } else {
                json!(id(&a["target"]))
            };
            shown
                .affordances
                .entry(id(&a["subject"]))
                .or_default()
                .push(whole(&json!([
                    a["action_type"],
                    target,
                    a["about"],
                    a["complete"],
                    a["available"],
                    a["reason"]
                ])));
        }
        shown
    }
}

/// Every compared key on which the two reports differ, naming what each client alone shows.
fn differences(two: &Shown, three: &Shown) -> Vec<String> {
    fn sets<T: Ord + std::fmt::Debug>(
        key: &str,
        a: &BTreeSet<T>,
        b: &BTreeSet<T>,
        out: &mut Vec<String>,
    ) {
        let only_2d: Vec<&T> = a.difference(b).collect();
        let only_3d: Vec<&T> = b.difference(a).collect();
        if !only_2d.is_empty() {
            out.push(format!("{key}: the 3D client does not show {only_2d:?}"));
        }
        if !only_3d.is_empty() {
            out.push(format!("{key}: the 2D client does not show {only_3d:?}"));
        }
    }
    let mut out = Vec::new();
    sets("places", &two.places, &three.places, &mut out);
    sets("doorways", &two.doorways, &three.doorways, &mut out);
    let people = |s: &Shown| s.people.keys().cloned().collect::<BTreeSet<_>>();
    sets("people", &people(two), &people(three), &mut out);
    for (person, name) in &two.people {
        if let Some(other) = three.people.get(person)
            && other != name
        {
            out.push(format!("people: {person} is {name} in 2D, {other} in 3D"));
        }
    }
    sets("objects", &two.objects, &three.objects, &mut out);
    let subjects = |s: &Shown| s.affordances.keys().cloned().collect::<BTreeSet<_>>();
    sets(
        "affordance subjects",
        &subjects(two),
        &subjects(three),
        &mut out,
    );
    for (subject, offers) in &two.affordances {
        if let Some(other) = three.affordances.get(subject)
            && other != offers
        {
            out.push(format!(
                "affordances of {subject}: 2D {offers:?}, 3D {other:?}"
            ));
        }
    }
    out
}

/// One paused world, `visitor` taken by each client in turn.
struct Moment {
    world: World,
    key: String,
    _save: SaveDir,
}

impl Moment {
    /// Starts `pack` with an admin token; with `walk_into`, the 2D client first walks the seat into
    /// the place of that tag while the world runs; then pauses the clock.
    async fn of(pack: &str, tag: &str, walk_into: Option<&str>) -> Self {
        let save = SaveDir::new(&format!("shown-parity-{tag}"));
        let mut arguments = hosted(pack, &save);
        // `--hold 0`: each client takes the seat the one before it just left (S11-B).
        arguments.extend(["--admin-token", TOKEN, "--hold", "0"].map(str::to_owned));
        let world = World::start(&owned(&arguments), None).await;
        let key = world_key(pack);
        if let Some(place_tag) = walk_into {
            let steps = json!([{ "walk_into": place_tag }]);
            let (status, lines) = godot2d::worlds::play(
                world.address,
                SEAT,
                &format!("shown-parity-{tag}-walk"),
                &steps,
                &[key.as_str()],
            )
            .await;
            assert!(
                status.success() && godot2d::passed(&lines),
                "{tag}: the walk into {place_tag} passes:\n{}",
                lines.join("\n")
            );
        }
        let paused = admin(
            world.address,
            "POST",
            "/admin/clock",
            Some(TOKEN),
            r#"{"paused": true}"#,
        )
        .await;
        assert_eq!(
            paused.status, 200,
            "{tag}: the clock pauses: {}",
            paused.body
        );
        Self {
            world,
            key,
            _save: save,
        }
    }

    async fn revision(&self) -> Value {
        self.world.status().await["revision"].clone()
    }

    /// The 2D client's report, with `extra` flags.
    async fn two_d(&self, name: &str, extra: &[&str]) -> Vec<String> {
        let mut arguments = vec![self.key.as_str()];
        arguments.extend(extra);
        let (status, lines) = godot2d::worlds::play(
            self.world.address,
            SEAT,
            name,
            &json!([{ "sleep": 2 }, { "report": true }]),
            &arguments,
        )
        .await;
        assert!(
            status.success() && godot2d::passed(&lines),
            "{name}: the 2D drive passes:\n{}",
            lines.join("\n")
        );
        lines
    }

    /// The 3D client's report, with `extra` flags.
    fn three_d(&self, name: &str, extra: &[&str]) -> Vec<String> {
        let mut arguments = vec![self.key.as_str()];
        arguments.extend(extra);
        let (status, lines) = godot3d::play(
            self.world.address,
            SEAT,
            name,
            &json!([{ "sleep": 2 }, { "report": true }]),
            &arguments,
        );
        assert!(
            status.success() && godot3d::passed(&lines),
            "{name}: the 3D probe passes"
        );
        lines
    }
}

/// AC-e-6 and AC-e-7 at one moment: the two reports equal on every compared key; then each client run
/// with a doorway of the start place dropped is told apart, by that doorway and that client.
async fn one_world_two_views(pack: &str, tag: &str, walk_into: Option<&str>) {
    let moment = Moment::of(pack, tag, walk_into).await;
    let before = moment.revision().await;
    let lines_2d = moment.two_d(&format!("shown-parity-{tag}-2d"), &[]).await;
    let lines_3d = moment.three_d(&format!("shown-parity-{tag}-3d"), &[]);
    let after = moment.revision().await;
    // R-e-2: a paused world held still between the two reports.
    assert_eq!(
        before, after,
        "{tag}: the paused world's revision did not move between the reports"
    );
    let (two, three) = (Shown::of(&lines_2d), Shown::of(&lines_3d));
    assert!(
        !two.places.is_empty() && !two.doorways.is_empty() && !two.people.is_empty(),
        "{tag}: the 2D client reports a world: {two:?}"
    );
    let found = differences(&two, &three);
    assert!(
        found.is_empty(),
        "{tag}: one world, two views — the reports differ:\n  {}",
        found.join("\n  ")
    );
    println!(
        "{tag}: equal — {} places, {} doorways, {} people, {} objects, {} offers",
        two.places.len(),
        two.doorways.len(),
        two.people.len(),
        two.objects.len(),
        two.affordances.values().map(Vec::len).sum::<usize>()
    );

    // AC-e-7: drop a doorway of the start place from one client's scene, then from the other's.
    let here = id(&tagged(&lines_2d, "SHOWN ").pop().expect("SHOWN")["place"]);
    let (from, to, _, _) = two
        .doorways
        .iter()
        .find(|d| d.0 == here)
        .cloned()
        .unwrap_or_else(|| panic!("{tag}: the start place {here} shows no doorway"));
    let flag = format!("--drop-doorway={from},{to}");
    let dropped_3d = Shown::of(&moment.three_d(&format!("shown-parity-{tag}-3d-drop"), &[&flag]));
    let found = differences(&two, &dropped_3d);
    assert!(
        found
            .iter()
            .any(|f| f.starts_with("doorways: the 3D client does not show") && f.contains(&to)),
        "{tag}: AC-e-7 3D: the dropped doorway {from} -> {to} is named: {found:?}"
    );
    println!("{tag}: AC-e-7, 3D dropped {from} -> {to}: {found:?}");
    let dropped_2d = Shown::of(
        &moment
            .two_d(&format!("shown-parity-{tag}-2d-drop"), &[&flag])
            .await,
    );
    let found = differences(&dropped_2d, &three);
    assert!(
        found
            .iter()
            .any(|f| f.starts_with("doorways: the 2D client does not show") && f.contains(&to)),
        "{tag}: AC-e-7 2D: the dropped doorway {from} -> {to} is named: {found:?}"
    );
    println!("{tag}: AC-e-7, 2D dropped {from} -> {to}: {found:?}");
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test shown_parity -- --ignored"]
async fn social_cafe_one_world_two_views() {
    one_world_two_views(SOCIAL_CAFE, "social-cafe", None).await;
}

#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test shown_parity -- --ignored"]
async fn market_town_one_world_two_views() {
    one_world_two_views(MARKET_TOWN, "market-town", None).await;
}

/// Market Town's street: the seat walked out of the café first (the 2D client's `walk_into`, by the
/// disclosed tag), then the moment is taken there.
#[tokio::test]
#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test shown_parity -- --ignored"]
async fn market_town_street_one_world_two_views() {
    one_world_two_views(MARKET_TOWN, "market-town-street", Some("street")).await;
}
