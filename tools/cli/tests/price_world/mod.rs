//! Shared by the price clerk's real-binary tests (`price_reporter.rs`, `price_reporter_lm.rs`;
//! `pr-s31d-reporter-policies.md` §8 W2, W3, W4, W6): scratch copies of `worlds/market-town` with the
//! literal edits §8 lists, asserted by a file comparison to differ from Market Town in nothing else; and
//! the save's notices, subscriptions and price notes read back **by slug** (`ARC-35` item 6).
//!
//! The edits: `notices` and `price-survey` enabled (and `notices` configured with the clerk's topic); the
//! clerk a surveyor of food and drink; Alice and Felix subscribed; each shop naming its keeper; the ten
//! food and drink kinds given their shop price as reference value; the café also asking juice, above the
//! store's, and holding ten (F-S31d-9: a second seller of one kind). Since the design's audit, Market
//! Town's shop prices moved; the reference values are read from the organization files, not written here.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityId, EventEnvelope, Visibility};
use mineworld_persistence::format;
use serde_json::Value;

use crate::headless::Tables;
use crate::market::MARKET_TOWN;

/// How much more than the store's juice the café asks, per mille.
pub const CAFE_JUICE_PERMILLE: u64 = 1_400;
/// The food and drink kinds, whose shop prices become their reference values.
pub const FOOD_AND_DRINK: [&str; 10] = [
    "apple",
    "bread",
    "milk",
    "juice",
    "coffee",
    "tea",
    "croissant",
    "cake",
    "sandwich",
    "soup",
];
/// Where a placed clerk and the visitor stand in the store at genesis: clear of every solid of
/// `places/store.yaml`.
pub const CLERK_AT: (i32, i32) = (2_000, 2_000);
pub const VISITOR_AT: (i32, i32) = (2_000, 4_500);

/// The clerk of one world: who, on which topic, in which currency.
#[derive(Clone, Copy, Debug)]
pub struct Clerk {
    pub key: &'static str,
    pub topic: &'static str,
    pub euro: bool,
}

/// W2 / W6: Carol on `prices.ingredients`, in US dollars.
pub const CAROL: Clerk = Clerk {
    key: "carol",
    topic: "prices.ingredients",
    euro: false,
};
/// W3 / W6b: Hana on `market.daily`, in euros; her 08:00 part moves from the workplace to the store.
pub const HANA: Clerk = Clerk {
    key: "hana",
    topic: "market.daily",
    euro: true,
};

/// Which of §8's edits a copy carries.
#[derive(Clone, Copy, Debug)]
pub struct Variant {
    /// `notices` enabled, configured, and its sections written (W4 (a) without).
    pub notices: bool,
    /// `price-survey` enabled and the clerk's section written (W4 (b) without).
    pub survey: bool,
    /// The clerk and the visitor placed in the store at genesis (W6).
    pub placed: bool,
}

/// W2 and W3.
pub const TOWN: Variant = Variant {
    notices: true,
    survey: true,
    placed: false,
};
/// W6 and W6b.
pub const PLACED: Variant = Variant {
    notices: true,
    survey: true,
    placed: true,
};

pub fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// Every file of a pack, relative to it, with its bytes.
pub fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut found = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("lists") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let bytes = std::fs::read(&path).expect("reads");
                found.insert(
                    path.strip_prefix(root).expect("inside").to_path_buf(),
                    bytes,
                );
            }
        }
    }
    found
}

fn edit(path: &Path, change: impl FnOnce(String) -> String) {
    let text = std::fs::read_to_string(path).expect("reads");
    std::fs::write(path, change(text)).expect("writes");
}

/// `text` with `from` replaced by `to`, which must occur exactly once.
pub fn once(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "{from:?} occurs once");
    text.replacen(from, to, 1)
}

/// A person file with its `location:` block moved to the store at `at`.
fn placed_in_store(text: &str, at: (i32, i32)) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|line| *line == "location:")
        .expect("a location block");
    let mut out: Vec<String> = lines[..start]
        .iter()
        .map(|line| (*line).to_owned())
        .collect();
    out.extend([
        "location:".to_owned(),
        "  place: store".to_owned(),
        "  position:".to_owned(),
        format!("    x: {}", at.0),
        format!("    y: {}", at.1),
    ]);
    out.extend(lines[start + 5..].iter().map(|line| (*line).to_owned()));
    out.join("\n") + "\n"
}

/// An organization's `prices: { key: n, … }` line, parsed.
pub fn prices(text: &str) -> BTreeMap<String, u64> {
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with("prices:"))
        .expect("a prices line");
    let inner = line
        .split_once('{')
        .and_then(|(_, rest)| rest.rsplit_once('}'))
        .expect("a flow mapping")
        .0;
    inner
        .split(',')
        .map(|pair| {
            let (key, value) = pair.split_once(':').expect("key: value");
            (
                key.trim().to_owned(),
                value.trim().parse().expect("a price"),
            )
        })
        .collect()
}

/// Every kind's reference value in the copy: the store's and the café's asks at the base.
pub fn references() -> BTreeMap<String, u64> {
    let organizations = Path::new(MARKET_TOWN).join("organizations");
    let read = |name: &str| std::fs::read_to_string(organizations.join(name)).expect("reads");
    let mut references = prices(&read("corner-store.yaml"));
    references.extend(prices(&read("cafe-company.yaml")));
    references
}

/// The copy for `clerk` and `variant`, asserted to differ from Market Town only in the edited files.
#[allow(clippy::too_many_lines)]
pub fn world(scratch: &Path, clerk: Clerk, variant: Variant) -> PathBuf {
    let root = scratch.join("market-town");
    copy_dir(Path::new(MARKET_TOWN), &root);
    let manifest = root.join("world.yaml");
    let mut added = Vec::new();
    if variant.notices {
        added.push("notices");
    }
    if variant.survey {
        added.push("price-survey");
    }
    edit(&manifest, |text| {
        let systems: String = added.iter().map(|pack| format!("  - {pack}\n")).collect();
        let text = if added.is_empty() {
            text
        } else {
            once(
                &text,
                "\n  - posture\n",
                &format!("\n  - posture\n{systems}"),
            )
        };
        if variant.notices {
            once(
                &text,
                "  - needs\n\n# The three organizations",
                "  - needs\n  - notices\n\n# The three organizations",
            )
        } else {
            text
        }
    });
    if variant.notices {
        std::fs::write(
            root.join("configure/notices.yaml"),
            format!(
                "topics:\n  {}:\n    body: prices\n    publishers: [{}]\n    valid_for: {{ min: \"1h\", max: \"48h\" }}\n    per_day: 4\n",
                clerk.topic, clerk.key
            ),
        )
        .expect("writes");
    }
    let people = root.join("people");
    edit(&people.join(format!("{}.yaml", clerk.key)), |text| {
        let text = if clerk.key == "hana" {
            once(
                &text,
                "{ from: \"08:00\", place: workplace, label: work }",
                "{ from: \"08:00\", place: store, label: work }",
            )
        } else {
            text
        };
        let text = if variant.placed {
            placed_in_store(&text, CLERK_AT)
        } else {
            text
        };
        if variant.survey {
            text + "price-survey: { kinds: { categories: [food, drink] } }\n"
        } else {
            text
        }
    });
    if variant.placed {
        edit(&people.join("visitor.yaml"), |text| {
            placed_in_store(&text, VISITOR_AT)
        });
    }
    if variant.notices {
        for keeper in ["alice", "felix"] {
            edit(&people.join(format!("{keeper}.yaml")), |text| {
                text + &format!("notices: {{ subscribe: [{}] }}\n", clerk.topic)
            });
        }
    }
    let references = references();
    let cafe_juice = references["juice"] * CAFE_JUICE_PERMILLE / 1_000;
    let organizations = root.join("organizations");
    edit(&organizations.join("corner-store.yaml"), |text| {
        let line = text
            .lines()
            .find(|line| line.trim_start().starts_with("prices:"))
            .expect("prices")
            .to_owned();
        once(&text, &line, &format!("{line}\n    keepers: [felix]"))
    });
    edit(&organizations.join("cafe-company.yaml"), |text| {
        let line = text
            .lines()
            .find(|line| line.trim_start().starts_with("prices:"))
            .expect("prices")
            .to_owned();
        let priced = line.replacen(" }", &format!(", juice: {cafe_juice} }}"), 1);
        let text = once(&text, &line, &format!("{priced}\n    keepers: [alice]"));
        let held = text
            .lines()
            .find(|line| line.starts_with("holdings:"))
            .expect("holdings")
            .to_owned();
        once(&text, &held, &held.replacen(" }", ", juice: 10 }", 1))
    });
    for kind in FOOD_AND_DRINK {
        edit(&root.join(format!("items/{kind}.yaml")), |text| {
            text + &format!("economy: {{ reference: {} }}\n", references[kind])
        });
    }
    if clerk.euro {
        edit(&root.join("configure/economy.yaml"), |text| {
            once(
                &text,
                "  name: US dollar\n  symbol: \"$\"\n  exponent: 2\n  symbol_at: before\n",
                "  name: euro\n  symbol: \"€\"\n  exponent: 2\n  symbol_at: after\n",
            )
        });
    }
    let (ours, theirs) = (files(&root), files(Path::new(MARKET_TOWN)));
    let differing: BTreeSet<PathBuf> = ours
        .iter()
        .filter(|(path, bytes)| theirs.get(*path) != Some(bytes))
        .map(|(path, _)| path.clone())
        .collect();
    let mut expected: BTreeSet<PathBuf> = [
        "organizations/corner-store.yaml",
        "organizations/cafe-company.yaml",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect();
    if !added.is_empty() {
        expected.insert(PathBuf::from("world.yaml"));
    }
    if variant.notices {
        expected.extend(
            [
                "configure/notices.yaml",
                "people/alice.yaml",
                "people/felix.yaml",
            ]
            .map(PathBuf::from),
        );
    }
    if variant.placed {
        expected.insert(PathBuf::from("people/visitor.yaml"));
    }
    if variant.placed || variant.survey || clerk.key == "hana" {
        expected.insert(PathBuf::from(format!("people/{}.yaml", clerk.key)));
    }
    expected.extend(FOOD_AND_DRINK.map(|kind| PathBuf::from(format!("items/{kind}.yaml"))));
    if clerk.euro {
        expected.insert(PathBuf::from("configure/economy.yaml"));
    }
    assert_eq!(differing, expected, "only the edits differ");
    assert_eq!(
        ours.len(),
        theirs.len() + usize::from(variant.notices),
        "only configure/notices.yaml is added"
    );
    root
}

/// The facts of a save, in order.
pub fn facts(save: &Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

pub fn payload(fact: &EventEnvelope) -> Value {
    serde_json::from_slice(fact.payload().payload()).expect("a JSON payload")
}

/// An entity id as JSON encodes it: a decimal string, or an object with an `entity` field.
pub fn entity(value: &Value) -> EntityId {
    let raw = value["entity"]
        .as_str()
        .or_else(|| value.as_str())
        .unwrap_or_else(|| panic!("an entity: {value}"));
    EntityId::from_raw(raw.parse().expect("a numeric id"))
}

/// One posted notice, as the save states it.
#[derive(Debug, Clone)]
pub struct Posted {
    /// Its index in the save's facts.
    pub index: usize,
    pub at: i64,
    pub topic: String,
    pub publisher: EntityId,
    pub body: Value,
    pub valid_until: i64,
    pub valid_for: i64,
    pub audience: BTreeSet<EntityId>,
}

pub fn notices(facts: &[EventEnvelope]) -> Vec<Posted> {
    facts
        .iter()
        .enumerate()
        .filter(|(_, fact)| fact.event_type().as_str() == "notice-published")
        .map(|(index, fact)| {
            let notice = &payload(fact)["notice"];
            let Visibility::Entities(audience) = fact.visibility() else {
                panic!(
                    "a notice is seen by named entities: {:?}",
                    fact.visibility()
                );
            };
            let valid_until = notice["valid_until"].as_i64().expect("an instant");
            Posted {
                index,
                at: fact.at().seconds(),
                topic: notice["topic"].as_str().expect("a topic").to_owned(),
                publisher: entity(&notice["publisher"]),
                body: notice["body"].clone(),
                valid_until,
                valid_for: valid_until - notice["issued_at"].as_i64().expect("an instant"),
                audience: audience.clone(),
            }
        })
        .collect()
}

/// The subscribers of `topic` just before `at`, folded from the save's subscription facts.
pub fn subscribers(facts: &[EventEnvelope], topic: &str, at: i64) -> BTreeSet<EntityId> {
    let mut found = BTreeSet::new();
    for fact in facts.iter().filter(|fact| fact.at().seconds() <= at) {
        let body = payload(fact);
        match fact.event_type().as_str() {
            "subscriptions-seeded" => {
                let person = entity(&body["person"]);
                let topics = body["topics"].as_array().expect("topics");
                if topics.iter().any(|seeded| seeded == topic) {
                    found.insert(person);
                } else {
                    found.remove(&person);
                }
            }
            "subscribed" if body["topic"] == topic => {
                found.insert(entity(&body["person"]));
            }
            "unsubscribed" if body["topic"] == topic => {
                found.remove(&entity(&body["person"]));
            }
            _ => {}
        }
    }
    found
}

/// Every posted notice is seen by exactly the topic's subscribers at that instant and its publisher.
pub fn assert_audiences(facts: &[EventEnvelope], posted: &[Posted]) {
    for notice in posted {
        let mut expected = subscribers(facts, &notice.topic, notice.at);
        expected.insert(notice.publisher);
        assert_eq!(notice.audience, expected, "the audience of {notice:?}");
    }
}
