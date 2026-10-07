//! CP-2: a Person's objective biography, derived from the log, matches the events that produced it
//! (`step-09-social.md` C6, `DECISIONS.md` `ARC-29`).
//!
//! ```text
//! mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save S
//! mineworld biography worlds/social-cafe --save S --person alice [--json]
//! ```
//!
//! The oracle is the facts' own **typed payloads**, decoded with each owner pack's published type —
//! not the envelope fields the command selects by — so the expected set does not come from the code
//! under test (rules §25). What would pass without proving anything, and how each is excluded
//! (`ARC-23`):
//!
//! ```text
//! an empty biography              matches any log; Alice's must hold every one of the seven types
//! an invented entry               sound: every entry is a fact of the log whose payload names her
//! a dropped entry                 complete: every such fact appears, exactly once
//! ```

mod headless;
mod social;

use std::collections::{BTreeMap, BTreeSet};

use headless::{PACK, Tables, fresh, mineworld, run, stderr, stdout};
use mineworld_contracts::{EventEnvelope, WorldTime};
use mineworld_persistence::format;
use serde_json::Value;

/// Alice's biography, from a fresh process, as JSON lines.
fn entries(save: &str, person: &str) -> Vec<Value> {
    let output = mineworld(&[
        "biography",
        PACK,
        "--save",
        save,
        "--person",
        person,
        "--json",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
        .lines()
        .map(|line| serde_json::from_str(line).expect("a JSON entry"))
        .collect()
}

#[test]
fn alices_biography_is_exactly_the_facts_that_name_her() {
    let save = fresh("biography-30");
    run(7, 30, Some(&save));
    let save_path = save.to_str().expect("path");
    let tables = Tables::read(&save);
    let facts: BTreeMap<u64, EventEnvelope> = tables
        .facts
        .iter()
        .map(|(id, bytes)| (*id, format::decode(bytes, "fact").expect("a fact")))
        .collect();
    let loaded = mineworld_worldpack::WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    let key_of = |id: mineworld_contracts::EntityId| {
        loaded
            .ids()
            .iter()
            .find(|(_, entity)| **entity == id)
            .map(|(key, _)| key.as_str().to_owned())
    };
    let alice = loaded
        .id(&mineworld_contracts::EntityKey::new("alice").expect("a key"))
        .expect("alice");

    let biography = entries(save_path, "alice");
    // Every check below records what it found rather than stopping at the first, so a failing run
    // says which of the three properties broke — located, sound, complete — and by how much.
    let mut problems: Vec<String> = Vec::new();

    // ── Located: every biographical type occurs in her life. ──────────────────────────────────
    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &biography {
        *by_type
            .entry(entry["event_type"].as_str().expect("a type").to_owned())
            .or_default() += 1;
    }
    eprintln!(
        "alice's biography, {} entries: {by_type:?}",
        biography.len()
    );
    for kind in social::BIOGRAPHICAL_TYPES {
        if by_type.get(kind).copied().unwrap_or(0) == 0 {
            problems.push(format!("LOCATED: alice's biography has no {kind}"));
        }
    }

    // ── Sound: nothing invented. ─────────────────────────────────────────────────────────────
    let mut listed = BTreeSet::new();
    let mut invented = Vec::new();
    for entry in &biography {
        let id = entry["event_id"].as_u64().expect("an id");
        let Some(fact) = facts.get(&id) else {
            invented.push(format!("#{id} is not a fact of the log"));
            continue;
        };
        if entry["event_type"] != fact.event_type().as_str()
            || entry["at"] != fact.at().seconds()
            || entry["place"].as_str().map(str::to_owned)
                != fact.place().and_then(|place| key_of(place.entity_id()))
        {
            invented.push(format!("#{id}: type, instant or place differ from the log"));
        }
        if social::payload_names(fact, alice) != Some(true) {
            invented.push(format!(
                "#{id} ({}) does not name alice in its own payload",
                fact.event_type().as_str()
            ));
        }
        if !listed.insert(id) {
            invented.push(format!("#{id} is listed twice"));
        }
    }
    if !invented.is_empty() {
        problems.push(format!(
            "SOUND: {} entr(ies) the log does not support, first: {:?}",
            invented.len(),
            invented.iter().take(3).collect::<Vec<_>>()
        ));
    }

    // ── Complete: nothing dropped. ───────────────────────────────────────────────────────────
    let expected: BTreeSet<u64> = facts
        .iter()
        .filter(|(_, fact)| social::payload_names(fact, alice) == Some(true))
        .map(|(id, _)| *id)
        .collect();
    let dropped: Vec<String> = expected
        .difference(&listed)
        .map(|id| format!("#{id} {}", facts[id].event_type().as_str()))
        .collect();
    if !dropped.is_empty() {
        problems.push(format!(
            "COMPLETE: {} fact(s) whose payload names alice are missing, first: {:?}",
            dropped.len(),
            dropped.iter().take(3).collect::<Vec<_>>()
        ));
    }
    assert!(
        problems.is_empty(),
        "the biography does not match the log:\n{}",
        problems.join("\n")
    );
    assert_eq!(listed, expected);

    // ── Derived, not stored: the same bytes every time, and the same ids as JSON. ────────────
    let text = |person: &str| {
        let output = mineworld(&["biography", PACK, "--save", save_path, "--person", person]);
        assert!(output.status.success(), "{}", stderr(&output));
        stdout(&output)
    };
    let first = text("alice");
    assert_eq!(first, text("alice"), "regenerated identically");
    for entry in &biography {
        assert!(
            first.contains(&format!("#{} ", entry["event_id"])),
            "every line carries its event id"
        );
    }

    // ── Names alongside keys (10c C4): checked against the authored literals. ────────────────
    assert!(
        first.starts_with("biography  alice \"Alice Moreau\" (entity "),
        "the header names her: {}",
        first.lines().next().unwrap_or_default()
    );
    let mut named_counterparts = 0;
    for entry in &biography {
        assert_eq!(
            entry["name"], "Alice Moreau",
            "her name on every JSON entry"
        );
        let keys = entry["counterparts"].as_array().expect("an array");
        let names = entry["counterpart_names"].as_array().expect("an array");
        assert_eq!(keys.len(), names.len(), "aligned with the keys: {entry}");
        for (key, name) in keys.iter().zip(names) {
            let key = key.as_str().expect("a key");
            assert_eq!(
                name.as_str(),
                social::authored_name(key),
                "{key}'s name, as authored: {entry}"
            );
            named_counterparts += 1;
        }
    }
    assert!(
        named_counterparts > 0,
        "located: some entry names somebody else"
    );
    assert!(
        first.contains("bob \"Bob Achterberg\""),
        "and a text line shows a counterpart as key and name"
    );
}

#[test]
fn biography_refuses_what_it_cannot_answer_by_name_and_never_panics() {
    let save = fresh("biography-refusals");
    run(7, 1, Some(&save));
    let save_path = save.to_str().expect("path");
    let elsewhere = fresh("biography-another-town");
    let created = mineworld(&[
        "create",
        elsewhere.join("other-town").to_str().expect("path"),
    ]);
    assert!(created.status.success(), "{}", stderr(&created));
    let other_pack = elsewhere.join("other-town");
    let other_pack = other_pack.to_str().expect("path");
    let missing = fresh("biography-no-save");
    for (arguments, names) in [
        (
            vec![
                "biography",
                PACK,
                "--save",
                missing.to_str().expect("path"),
                "--person",
                "alice",
            ],
            "holds no save",
        ),
        (
            vec![
                "biography",
                other_pack,
                "--save",
                save_path,
                "--person",
                "alice",
            ],
            "is a world of 'social-cafe', not of 'other-town'",
        ),
        (
            vec!["biography", PACK, "--save", save_path, "--person", "nobody"],
            "'nobody' is not a person of social-cafe",
        ),
        (
            vec!["biography", PACK, "--save", save_path, "--person", "cafe"],
            "'cafe' is not a person of social-cafe",
        ),
    ] {
        let output = mineworld(&arguments);
        let complaint = stderr(&output);
        assert!(!output.status.success(), "{arguments:?} must fail");
        assert!(complaint.contains(names), "{arguments:?}: {complaint}");
        assert!(
            !complaint.contains("panicked"),
            "{arguments:?}: {complaint}"
        );
    }
}
