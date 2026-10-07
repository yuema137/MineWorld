//! **Milestone B** — *Alice and Bob persist, know each other, share an activity, and survive a restart
//! with their history* (`docs/HUMAN_REVIEW_QUEUE.md`; `step-09-social.md` SD-16, I-8, C7).
//!
//! Real processes and S5's real persistence throughout; no in-memory world stands in for a restart.
//!
//! ```text
//! control   mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save C   uninterrupted
//! located   in C's log, by name: alice→bob and bob→alice became-acquainted; a relationship-changed
//!           between them; a group-activity-ended whose members include both
//! killed    the same command with --save K, SIGKILLed once it has printed the day after that history
//!           — the history is shown to be in K before the re-run — then the same command again
//! read      mineworld biography … --person alice / bob, fresh processes that never held the world
//! hosted    mineworld server worlds/social-cafe --save K: alice and bob seated read their own
//!           disclosed relationship values; the server is SIGKILLed and started again on K, and they
//!           read the same values, at the same revision; their biographies are unchanged
//! inspect   mineworld inspect K: every cause resolves; the activities' wakes are process causes
//! ```
//!
//! What would pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! a history made after the restart   the kill day is computed from C's log, and the located facts are
//!                                     found in K, byte-identical to C's, before K is re-run
//! a victim that was never killed      signal 9, no summary printed, K's head short of C's
//! a restart that read nothing back    the re-run reports resuming at K's head on disk; the server
//!                                     reports the same instance and revision after its kill
//! "unchanged" by reading nothing      the values read are located non-trivial: the level of the
//!                                     located crossing, and every value equal before and after
//! an idle world                       the social precondition (every bucket) holds on C first
//! ```

mod headless;
mod social;
mod support;

use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Stdio};

use headless::{BINARY, PACK, Tables, fresh, mineworld, run, stderr, stdout};
use mineworld_contracts::{EntityId, EntityKey, EventEnvelope, PersonId, WorldTime};
use mineworld_group_activity::GroupActivityEnded;
use mineworld_relationships::{Acquaintances, BecameAcquainted, RelationshipChanged};
use mineworld_server::WireObservation;
use social::decoded;
use support::{Client, Server};

const SEED: u64 = 7;
const DAYS: u64 = 30;
const DAY: i64 = 86_400;

/// Starts `run … --save save` and SIGKILLs it once it has printed the line for `day`; asserts the
/// death was real.
fn killed_after(save: &Path, day: i64) {
    let mut child = Command::new(BINARY)
        .args([
            "run",
            PACK,
            "--headless",
            "--seed",
            &SEED.to_string(),
            "--days",
            &DAYS.to_string(),
            "--save",
            save.to_str().expect("path"),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary starts");
    let reader = BufReader::new(child.stdout.take().expect("piped"));
    let wanted = format!("day {day} ");
    let mut printed = Vec::new();
    for line in reader.lines() {
        let line = line.expect("utf-8");
        let reached = line.starts_with(&wanted);
        printed.push(line);
        if reached {
            break;
        }
    }
    child.kill().expect("SIGKILL is delivered");
    let status = child.wait().expect("reaped");
    assert_eq!(status.signal(), Some(9), "killed by SIGKILL, not finished");
    assert!(
        !printed.iter().any(|line| line.starts_with("history ")),
        "the victim did not finish: {printed:?}"
    );
}

/// What Alice and Bob's shared history is made of, located in a log.
struct Located {
    alice_knows_bob: EventEnvelope,
    bob_knows_alice: EventEnvelope,
    crossed: EventEnvelope,
    shared: EventEnvelope,
}

impl Located {
    fn find(facts: &[EventEnvelope], alice: PersonId, bob: PersonId) -> Self {
        let pair = |a: PersonId, b: PersonId| (a == alice && b == bob) || (a == bob && b == alice);
        let acquainted = |from: PersonId, to: PersonId| {
            facts
                .iter()
                .find(|fact| {
                    decoded::<BecameAcquainted>(fact)
                        .is_some_and(|fact| fact.person() == from && fact.counterpart() == to)
                })
                .cloned()
                .unwrap_or_else(|| panic!("no became-acquainted {from:?} → {to:?} in the log"))
        };
        let crossed = facts
            .iter()
            .find(|fact| {
                decoded::<RelationshipChanged>(fact).is_some_and(|fact| {
                    pair(fact.person(), fact.counterpart()) && fact.to() > fact.from()
                })
            })
            .cloned()
            .expect(
                "no relationship-changed between alice and bob in the log: they never grew closer",
            );
        let shared = facts
            .iter()
            .find(|fact| {
                decoded::<GroupActivityEnded>(fact).is_some_and(|ended| {
                    ended.members().contains(&alice) && ended.members().contains(&bob)
                })
            })
            .cloned()
            .expect("no group-activity-ended with both alice and bob among its members");
        Self {
            alice_knows_bob: acquainted(alice, bob),
            bob_knows_alice: acquainted(bob, alice),
            crossed,
            shared,
        }
    }

    fn all(&self) -> [&EventEnvelope; 4] {
        [
            &self.alice_knows_bob,
            &self.bob_knows_alice,
            &self.crossed,
            &self.shared,
        ]
    }
}

fn biography(save: &Path, person: &str) -> String {
    let output = mineworld(&[
        "biography",
        PACK,
        "--save",
        save.to_str().expect("path"),
        "--person",
        person,
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
}

/// The observer's own disclosed relationship values, decoded with relationships' own type.
fn own_acquaintances(observation: &WireObservation) -> Option<Acquaintances> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "acquaintances")?;
    serde_json::from_value(record.payload().clone()).ok()
}

/// Seats a client as `seat` on the server and reads its own acquaintances.
async fn seated(server: &Server, seat: &str) -> (Client, Acquaintances, Option<u64>) {
    let mut client = Client::connect(server.address).await;
    client.join(seat).await;
    let (revision, observation) = client.perceived().await;
    let known = own_acquaintances(&observation)
        .unwrap_or_else(|| panic!("{seat}'s own acquaintances are disclosed to {seat}"));
    (client, known, revision.map(|revision| revision.raw()))
}

#[tokio::test(flavor = "multi_thread")]
async fn alice_and_bob_know_each_other_share_an_activity_and_survive_a_restart_with_their_history()
{
    let loaded = mineworld_worldpack::WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    let person = |key: &str| -> PersonId {
        let id: EntityId = loaded
            .id(&EntityKey::new(key).expect("a key"))
            .expect("in the pack");
        PersonId::new(id, mineworld_contracts::EntityType::Person).expect("a person")
    };
    let (alice, bob) = (person("alice"), person("bob"));

    // ── The control, and the history located in it. ─────────────────────────────────────────
    let control = fresh("milestone-b-control");
    run(SEED, DAYS, Some(&control));
    let control_tables = Tables::read(&control);
    let control_facts = social::facts_of(&control_tables);
    social::precondition(&control_facts, i64::try_from(DAYS).expect("fits"), true);
    let located = Located::find(&control_facts, alice, bob);
    for fact in located.all() {
        eprintln!(
            "located #{} {} at t{} (day {})",
            fact.id().raw(),
            fact.event_type().as_str(),
            fact.at().seconds(),
            fact.at().seconds() / DAY + 1
        );
    }
    let history_day = located
        .all()
        .iter()
        .map(|fact| fact.at().seconds() / DAY + 1)
        .max()
        .expect("four facts");
    let kill_day = history_day + 1;
    assert!(
        kill_day < i64::try_from(DAYS).expect("fits"),
        "the history is made by day {history_day}, early enough to kill the run after it"
    );

    // ── Killed after the history was made, then the same command again. ────────────────────────
    let killed = fresh("milestone-b-killed");
    killed_after(&killed, kill_day);
    let at_kill = Tables::read(&killed);
    let control_head = control_tables.journal.len();
    assert!(
        at_kill.journal.len() < control_head,
        "killed short of the end"
    );
    for fact in located.all() {
        let id = fact.id().raw();
        let stored = at_kill.facts.iter().find(|(row, _)| *row == id);
        let original = control_tables.facts.iter().find(|(row, _)| *row == id);
        assert!(
            stored.is_some() && stored == original,
            "#{id} {} was in the save, as in the control, when the process died",
            fact.event_type().as_str()
        );
    }
    eprintln!(
        "killed after day {kill_day}: {} of {control_head} revisions on disk, the history among them",
        at_kill.journal.len()
    );
    let survivor = run(SEED, DAYS, Some(&killed));
    let header = survivor.lines().next().expect("a header");
    assert!(
        header.contains("resumed")
            && header.contains(&format!("at revision {}", at_kill.journal.len())),
        "the re-run resumed the save on disk: {header}"
    );
    Tables::read(&killed).assert_same_history(&control_tables, "killed and re-run");

    // ── Their biographies, read by processes that never held the world. ──────────────────────
    let alices = biography(&killed, "alice");
    let bobs = biography(&killed, "bob");
    assert_eq!(
        alices,
        biography(&control, "alice"),
        "alice's biography survived the kill"
    );
    assert_eq!(
        bobs,
        biography(&control, "bob"),
        "bob's biography survived the kill"
    );
    for fact in located.all() {
        let id = format!("#{} ", fact.id().raw());
        assert!(
            alices.contains(&id) && bobs.contains(&id),
            "#{} {} is in both biographies",
            fact.id().raw(),
            fact.event_type().as_str()
        );
    }

    // ── Hosted, killed and hosted again on the same save. ────────────────────────────────────
    let save = killed.to_str().expect("path");
    let command = ["server", PACK, "--save", save];
    let mut first = Server::start(&command).await;
    let instance = first.status().await["instance"].clone();
    let (alice_client, alice_knows, alice_revision) = seated(&first, "alice").await;
    let (bob_client, bob_knows, bob_revision) = seated(&first, "bob").await;
    let alice_of_bob = alice_knows.of(bob).cloned().expect("alice knows bob");
    let bob_of_alice = bob_knows.of(alice).cloned().expect("bob knows alice");
    eprintln!(
        "hosted: alice → bob {:?} (familiarity {}, regard {}, exchanges {}, activities {}); \
         bob → alice {:?} (familiarity {}, regard {}, exchanges {}, activities {}); revision {:?}",
        alice_of_bob.level(),
        alice_of_bob.familiarity(),
        alice_of_bob.regard(),
        alice_of_bob.exchanges(),
        alice_of_bob.activities_shared(),
        bob_of_alice.level(),
        bob_of_alice.familiarity(),
        bob_of_alice.regard(),
        bob_of_alice.exchanges(),
        bob_of_alice.activities_shared(),
        alice_revision,
    );
    let crossing = decoded::<RelationshipChanged>(&located.crossed).expect("the located crossing");
    let crossed_holder = if crossing.person() == alice {
        &alice_of_bob
    } else {
        &bob_of_alice
    };
    assert!(
        crossed_holder.level() >= crossing.to(),
        "the located crossing is in the values the world holds, never decaying"
    );
    assert!(alice_of_bob.activities_shared() > 0 && bob_of_alice.activities_shared() > 0);
    assert_eq!(
        mineworld_contracts::Causation::Event(alice_of_bob.first_met_by()),
        *located.alice_knows_bob.caused_by(),
        "the values remember the fact through which they met — the one that caused the located \
         became-acquainted"
    );

    drop((alice_client, bob_client));
    let died = first.kill();
    assert_eq!(
        died.signal(),
        Some(9),
        "the server died of SIGKILL: {died:?}"
    );

    let second = Server::start(&command).await;
    assert_eq!(
        second.status().await["instance"],
        instance,
        "the same world"
    );
    let (_alice_again, alice_after, alice_revision_after) = seated(&second, "alice").await;
    let (_bob_again, bob_after, bob_revision_after) = seated(&second, "bob").await;
    assert_eq!(
        alice_revision_after, alice_revision,
        "at the revision it was killed at"
    );
    assert_eq!(bob_revision_after, bob_revision);
    assert_eq!(
        alice_after, alice_knows,
        "alice's relationships survived the server's kill"
    );
    assert_eq!(
        bob_after, bob_knows,
        "bob's relationships survived the server's kill"
    );
    drop(second);

    assert_eq!(
        biography(&killed, "alice"),
        alices,
        "and her biography is unchanged"
    );
    assert_eq!(biography(&killed, "bob"), bobs, "and his");

    // ── Every cause resolves. ────────────────────────────────────────────────────────────────
    let inspected = mineworld(&["inspect", save, "--last", "0"]);
    assert!(inspected.status.success(), "{}", stderr(&inspected));
    let report = stdout(&inspected);
    assert!(
        report.contains("AC-9       every cause resolves"),
        "{report}"
    );
    let process_causes = report
        .lines()
        .find(|line| line.starts_with("causes     process "))
        .expect("the activities' wakes are process causes");
    eprintln!("inspect: {process_causes}");
}
