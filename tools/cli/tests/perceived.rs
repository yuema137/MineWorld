//! What a Person perceived, from a save: presence's audience rule over a real log, and the offline
//! command that runs it (`DECISIONS.md` `ARC-43`, `MODULE_SPEC.md` §8.1, step-12 §17 C-C3, C-C3b).
//!
//! ```text
//! mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save S
//! mineworld perceived worlds/social-cafe --save S --person KEY [--since ID] [--json]
//! ```
//!
//! The oracles are independent of `audience.rs`: the world the pack authors (its loader's own
//! placement, for genesis), the world the save resumes into (the kernel's reduction, for the fold),
//! and each `arrived` fact's typed payload decoded with presence's published type.

mod headless;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use headless::{PACK, Tables, fresh, mineworld, run, stderr, stdout};
use mineworld_contracts::{
    Causation, EntityId, EntityKey, Event, EventEnvelope, PerceivedEvent, PlaceId, Visibility,
    WorldTime,
};
use mineworld_persistence::{Durability, PersistentWorld, SqliteBackend, format};
use mineworld_persistence::{PersistenceBackend, WorldRevision};
use mineworld_presence::audience::Whereabouts;
use mineworld_presence::{Arrived, Presence};
use mineworld_server::ServerFrame;
use mineworld_worldpack::WorldPack;
use serde_json::Value;
use serde_json::json;
use support::{Client, Server};

fn facts_of(save: &std::path::Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

/// `mineworld perceived` for `person`: its exit, stdout lines and stderr.
fn perceived(save: &str, person: &str, extra: &[&str]) -> (bool, Vec<String>, String) {
    let mut arguments = vec!["perceived", PACK, "--save", save, "--person", person];
    arguments.extend_from_slice(extra);
    let output = mineworld(&arguments);
    (
        output.status.success(),
        stdout(&output).lines().map(str::to_owned).collect(),
        stderr(&output),
    )
}

/// The event ids of the plain lines, which begin with one.
fn ids(lines: &[String]) -> Vec<u64> {
    lines
        .iter()
        .map(|line| {
            line.split(' ')
                .next()
                .and_then(|id| id.parse().ok())
                .unwrap_or_else(|| panic!("a line beginning with an event id: {line}"))
        })
        .collect()
}

/// The oracle: which facts of `log` `person` learned of, from the contract's `Visibility` read
/// literally and `person`'s place tracked through `arrived` payloads decoded with presence's own
/// type — a second derivation, sharing no code with `audience.rs`. Also returns the place it put
/// `person` in once the genesis facts were read.
fn learned(log: &[EventEnvelope], person: EntityId) -> (Vec<u64>, Option<PlaceId>) {
    let mut here: Option<PlaceId> = None;
    let mut at_genesis_end = None;
    let mut ids = Vec::new();
    for fact in log {
        if *fact.event_type() == <Arrived as Event>::EVENT_TYPE {
            let arrived: Arrived = serde_json::from_slice(
                fact.payload()
                    .payload_for::<Arrived>()
                    .expect("an arrived record"),
            )
            .expect("an arrived payload");
            if arrived.person().entity_id() == person {
                here = Some(arrived.location().place());
            }
        }
        if *fact.caused_by() != Causation::WorldGenesis && at_genesis_end.is_none() {
            at_genesis_end = Some(here);
        }
        let named = fact.subjects().contains(&person) || fact.participants().contains(&person);
        let heard = match fact.visibility() {
            Visibility::Public => true,
            Visibility::Participants => fact.participants().contains(&person),
            Visibility::Entities(set) => set.contains(&person),
            Visibility::Place(place) => named || here == Some(*place),
            Visibility::SystemInternal => false,
        };
        if heard {
            ids.push(fact.id().raw());
        }
    }
    (ids, at_genesis_end.flatten())
}

/// S10's A-1, the premise of a live fold seeded from components agreeing with an offline fold from
/// facts (step-12 §17.9 R-SC1): over a 30-day run, the fold of every fact equals presence's own
/// components in the world the save resumes into. Failing it is a stop, not a test to adjust.
#[test]
fn the_fold_of_a_whole_log_equals_presence_in_the_resumed_world() {
    let save = fresh("perceived-fold-30");
    run(7, 30, Some(&save));
    let facts = facts_of(&save);

    let mut folded = Whereabouts::new();
    for fact in &facts {
        folded.apply(fact);
    }

    let pack = WorldPack::read(PACK).expect("reads");
    let composed = pack.compose().expect("composes");
    let backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let (world, _) =
        PersistentWorld::resume(Box::new(backend), composed.world).expect("the save resumes");
    let read = world.world().read();
    let seeded = Whereabouts::from_world(&read);
    assert_eq!(
        folded, seeded,
        "the fold from facts and the components disagree"
    );

    // Not vacuous: everybody is somewhere, and people walked after genesis.
    let people = read.components::<Presence>().count();
    assert!(people >= 10, "only {people} people placed");
    let walked = facts
        .iter()
        .filter(|fact| {
            *fact.event_type() == <Arrived as Event>::EVENT_TYPE
                && *fact.caused_by() != Causation::WorldGenesis
        })
        .count();
    assert!(walked > 100, "only {walked} arrivals after genesis");
    eprintln!(
        "{} facts, {people} people placed, {walked} arrivals after genesis",
        facts.len()
    );
}

/// C-C3b: `mineworld perceived` over a 30-day save is exactly the log judged for one person — sound,
/// complete, ordered, resumable by `--since`, refusing an unknown person by name, and in the server's
/// wire form with `--json`.
#[test]
fn the_export_is_the_log_judged_for_one_person() {
    let save = fresh("perceived-export-30");
    run(7, 30, Some(&save));
    let path = save.to_str().expect("a printable path");
    let log = facts_of(&save);
    let wanderer = WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    let authored = wanderer
        .world()
        .read()
        .component::<Presence>(
            wanderer
                .id(&EntityKey::new("wanderer").expect("a key"))
                .expect("declared"),
        )
        .map(|presence| presence.location().place());
    let person = wanderer
        .id(&EntityKey::new("wanderer").expect("a key"))
        .expect("declared");

    let (ok, lines, err) = perceived(path, "wanderer", &[]);
    assert!(ok, "{err}");
    let exported = ids(&lines);

    // Sound and complete against the independent oracle; every id a fact of the save.
    let (expected, placed_at_genesis) = learned(&log, person);
    assert_eq!(
        placed_at_genesis, authored,
        "the oracle's genesis placement is the pack's authored one"
    );
    let in_save: std::collections::BTreeSet<u64> = log.iter().map(|f| f.id().raw()).collect();
    assert!(exported.iter().all(|id| in_save.contains(id)));
    assert!(
        exported.windows(2).all(|pair| pair[0] < pair[1]),
        "ascending, no duplicate"
    );
    assert_eq!(exported, expected, "the export and the oracle differ");

    // Not vacuous: beyond genesis, the wanderer overheard Place facts that do not name them.
    let by_id: std::collections::BTreeMap<u64, &EventEnvelope> =
        log.iter().map(|f| (f.id().raw(), f)).collect();
    let overheard = exported
        .iter()
        .filter(|id| {
            let fact = by_id[id];
            *fact.caused_by() != Causation::WorldGenesis
                && matches!(fact.visibility(), Visibility::Place(_))
                && !fact.subjects().contains(&person)
                && !fact.participants().contains(&person)
        })
        .count();
    eprintln!(
        "wanderer perceived {} of {} facts, {overheard} overheard",
        exported.len(),
        log.len()
    );
    assert!(overheard > 0, "nothing overheard after genesis");

    // --since X: exactly the suffix after X.
    let cursor = exported[exported.len() / 2];
    let (ok, lines, err) = perceived(path, "wanderer", &["--since", &cursor.to_string()]);
    assert!(ok, "{err}");
    let suffix: Vec<u64> = exported.iter().copied().filter(|id| *id > cursor).collect();
    assert_eq!(ids(&lines), suffix);

    // --json: one PerceivedEvent per line, the same facts.
    let (ok, lines, err) = perceived(path, "wanderer", &["--json"]);
    assert!(ok, "{err}");
    let decoded: Vec<u64> = lines
        .iter()
        .map(|line| {
            serde_json::from_str::<PerceivedEvent<Value>>(line)
                .unwrap_or_else(|error| panic!("{error}: {line}"))
                .envelope()
                .id()
                .raw()
        })
        .collect();
    assert_eq!(decoded, exported);

    // An unknown person is refused by name.
    let (ok, lines, err) = perceived(path, "nobody", &[]);
    assert!(!ok && lines.is_empty(), "an unknown person is refused");
    assert!(err.contains("'nobody' is not a person"), "{err}");
}

/// What one connection was sent, in order: perceived frames (their fact ids and cursor) and
/// observation frames (their revision).
#[derive(Debug)]
enum Seen {
    Facts(Vec<u64>, u64),
    Observation(Option<u64>),
}

/// Reads every frame for `window`, keeping the ones the claims are about.
async fn record(client: &mut Client, window: Duration, into: &mut Vec<Seen>) {
    let deadline = Instant::now() + window;
    while Instant::now() < deadline {
        match client.frame().await {
            ServerFrame::Perceived { through, events } => into.push(Seen::Facts(
                events.iter().map(|e| e.envelope().id().raw()).collect(),
                through.raw(),
            )),
            ServerFrame::Observation { revision, .. } => {
                into.push(Seen::Observation(revision.map(WorldRevision::raw)));
            }
            _ => {}
        }
    }
}

fn joining(seat: &str, resume: Option<&str>, since: Value) -> Value {
    json!({ "t": "join", "protocol": 2, "invite": support::INVITE, "nickname": "memory",
            "seat": seat, "resume": resume, "perceived": { "since": since } })
}

/// CA-2, CA-6 and CA-12's persisted half on one recorded run of the real binary.
///
/// ```text
/// CA-2   live + resumed = offline: the ids a client is sent, across a dropped socket and a resume
///        with its cursor, are exactly `mineworld perceived --json` of the same save, up to the last
///        cursor it was sent — no gap, no duplicate, ascending — and beyond genesis
/// CA-6   every perceived fact recorded at revision ≤ R reached the client before any observation
///        of revision R
/// CA-12  on a persisted world, since: null serves from genesis
/// ```
#[tokio::test]
async fn a_resumed_stream_equals_the_offline_export() {
    let save = fresh("perceived-live-cafe");
    let path = save.to_str().expect("a printable path").to_owned();
    let mut server =
        Server::start(&["server", PACK, "--town", "--save", &path, "--hold", "10"]).await;

    let mut seen = Vec::new();
    let mut first = Client::connect(server.address).await;
    let ServerFrame::Welcome { resume, .. } =
        first.join_as(joining("wanderer", None, Value::Null)).await
    else {
        panic!("welcomed");
    };
    let resume = resume.expect("a resume secret").reveal().to_owned();
    record(&mut first, Duration::from_secs(20), &mut seen).await;
    first.disconnect().await;
    let cursor = seen
        .iter()
        .rev()
        .find_map(|item| match item {
            Seen::Facts(_, through) => Some(*through),
            Seen::Observation(_) => None,
        })
        .expect("a cursor");

    let mut second = Client::connect(server.address).await;
    let welcome = second
        .join_as(joining(
            "wanderer",
            Some(&resume),
            json!(cursor.to_string()),
        ))
        .await;
    assert!(
        matches!(welcome, ServerFrame::Welcome { .. }),
        "resumed inside the hold: {welcome:?}"
    );
    record(&mut second, Duration::from_secs(20), &mut seen).await;
    second.send(json!({ "t": "leave" })).await;
    server.kill();

    let received: Vec<u64> = seen
        .iter()
        .filter_map(|item| match item {
            Seen::Facts(ids, _) => Some(ids.clone()),
            Seen::Observation(_) => None,
        })
        .flatten()
        .collect();
    let last = seen
        .iter()
        .rev()
        .find_map(|item| match item {
            Seen::Facts(_, through) => Some(*through),
            Seen::Observation(_) => None,
        })
        .expect("a cursor");

    let (ok, lines, err) = perceived(&path, "wanderer", &["--json"]);
    assert!(ok, "{err}");
    let exported: Vec<u64> = lines
        .iter()
        .map(|line| {
            serde_json::from_str::<PerceivedEvent<Value>>(line)
                .expect("a PerceivedEvent")
                .envelope()
                .id()
                .raw()
        })
        .filter(|id| *id <= last)
        .collect();
    let log = facts_of(&save);
    let genesis = |id: u64| {
        log.iter()
            .any(|fact| fact.id().raw() == id && *fact.caused_by() == Causation::WorldGenesis)
    };
    eprintln!(
        "received {} facts, export {} up to cursor {last}, {} after genesis",
        received.len(),
        exported.len(),
        exported.iter().filter(|id| !genesis(**id)).count()
    );

    // CA-2.
    assert!(
        received.windows(2).all(|pair| pair[0] < pair[1]),
        "ascending, no duplicate"
    );
    assert_eq!(received, exported, "live + resumed = offline");
    let later: Vec<&EventEnvelope> = log
        .iter()
        .filter(|fact| exported.contains(&fact.id().raw()) && !genesis(fact.id().raw()))
        .filter(|fact| ["spoke", "arrived"].contains(&fact.event_type().as_str()))
        .collect();
    assert!(!later.is_empty(), "beyond genesis: a line or an arrival");

    // CA-12, persisted: since null serves from genesis.
    assert_eq!(
        received.first(),
        exported.first(),
        "the first perceived fact is the first one the wanderer learned of"
    );
    assert!(genesis(received[0]), "and it is a genesis fact");

    // CA-6.
    let backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let head = backend.head().expect("a head").raw();
    let mut revision_of: BTreeMap<u64, u64> = BTreeMap::new();
    for revision in 0..=head {
        for row in backend
            .facts_of(WorldRevision::from_raw(revision))
            .expect("readable")
        {
            revision_of.insert(row.id, revision);
        }
    }
    drop(backend);
    let mut arrived: BTreeSet<u64> = BTreeSet::new();
    let mut late = Vec::new();
    for item in &seen {
        match item {
            Seen::Facts(ids, _) => arrived.extend(ids),
            Seen::Observation(Some(revision)) => {
                late.extend(
                    exported
                        .iter()
                        .filter(|id| revision_of.get(id).is_some_and(|at| at <= revision))
                        .filter(|id| !arrived.contains(id))
                        .map(|id| (*id, *revision)),
                );
            }
            Seen::Observation(None) => panic!("a persisted world names its revision"),
        }
    }
    late.dedup();
    assert!(
        late.is_empty(),
        "facts after an observation of their revision: {late:?}"
    );
}
