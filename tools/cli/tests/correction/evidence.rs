//! What the corrected-memory scenario asserts and keeps: the save's facts by turn, the machine half of the §6
//! rubric, the restart and isolation checks, and the evidence files a reviewer scores.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::{Value, json};

use super::{LAST, Run};
use crate::support::MARKET_PACK;
use crate::support::cognition::{
    Spoken, actor_of, cited, decisions, home_file, perceived, recall_ids, recall_names, recalls,
    shown_lines, spoken, store_rows,
};

/// The store resumed: the creation marker is the one it had, every row taken before is the same row after, and
/// the cursor did not go back.
pub fn store_resumed(
    before: &(String, Option<u64>, Vec<(i64, u64)>),
    after: &(String, Option<u64>, Vec<(i64, u64)>),
) {
    assert_eq!(after.0, before.0, "the home was resumed, not recreated");
    assert!(
        before.2.iter().all(|row| after.2.contains(row)),
        "every row held before the stop is the same row after it"
    );
    assert!(after.1 >= before.1, "the cursor did not go back");
}

/// Bytes under a directory (0 when it does not exist).
pub fn dir_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .map(|entry| entry.expect("an entry").path())
        .map(|path| {
            if path.is_dir() {
                dir_size(&path)
            } else {
                path.metadata().map_or(0, |meta| meta.len())
            }
        })
        .sum()
}

/// The facts the assertions read: every spoke between the visitor and Alice, and the lines' ids by turn.
pub struct Facts {
    pub by_turn: Vec<Spoken>,
    pub alice_lines: Vec<Spoken>,
}

pub fn facts(run: &Run) -> Facts {
    let all = spoken(&perceived(MARKET_PACK, &run.save, "visitor"));
    let (alice, visitor) = (run.alice.to_string(), run.visitor.to_string());
    let by_turn = run
        .turns
        .iter()
        .map(|turn| {
            all.iter()
                .find(|fact| fact.speaker == visitor && fact.words == turn.words)
                .unwrap_or_else(|| panic!("turn {}'s line is a fact", turn.n))
                .clone()
        })
        .collect();
    let alice_lines = all
        .into_iter()
        .filter(|fact| fact.speaker == alice && fact.listener == visitor)
        .collect();
    Facts {
        by_turn,
        alice_lines,
    }
}

/// The machine half of the §6 rubric — (a) causality, (c) the original code, (e) both sources shown and cited
/// by valid recalls — as (check, holds, detail).
pub fn machine_checks(run: &Run, facts: &Facts) -> Vec<(&'static str, bool, String)> {
    let Some(last) = run.turns.iter().find(|turn| turn.words == LAST) else {
        return vec![(
            "turn 5 said",
            false,
            "the run stopped before turn 5".to_owned(),
        )];
    };
    let reply = last.reply.clone().unwrap_or_default();
    let fact = facts.alice_lines.iter().find(|fact| fact.words == reply);
    let caused = fact
        .and_then(|fact| fact.action)
        .is_some_and(|action| actor_of(&run.save, action) == run.alice.to_string());
    let consult = last.consult.clone().unwrap_or(Value::Null);
    let (code_line, correction_line) = (facts.by_turn[0].id, facts.by_turn[2].id);
    let shown = if consult.is_null() {
        BTreeSet::new()
    } else {
        cited(&consult)
    };
    let named = recalls(&consult);
    let valid = named.iter().all(|recall| {
        recall_ids(recall).is_some_and(|ids| !ids.is_empty() && ids.is_subset(&shown))
    });
    let cites = |id: u64| named.iter().any(|recall| recall_names(recall, id));
    vec![
        (
            "(a) the reply is a saved spoke caused by Alice's accepted request",
            last.reply.is_some() && caused,
            format!("reply {reply:?}, fact {fact:?}"),
        ),
        (
            "(c) the reply names the original code exactly",
            reply.contains(&run.lines.code),
            format!("code {:?}", run.lines.code),
        ),
        (
            "(e) both sources are shown and cited by valid recalls",
            shown.contains(&code_line)
                && shown.contains(&correction_line)
                && cites(code_line)
                && cites(correction_line)
                && valid,
            format!(
                "code line #{code_line}, correction #{correction_line}, shown {shown:?}, recalls {named:?}"
            ),
        ),
    ]
}

/// Every consult's trigger is a distinct fact, and no consult after the restart answers a line from before it.
pub fn no_trigger_replayed(run: &Run) {
    let log = decisions(&run.store, "alice");
    let triggers: Vec<String> = log
        .iter()
        .map(|consult| consult["input"]["trigger"]["event"].to_string())
        .collect();
    let distinct: BTreeSet<&String> = triggers.iter().collect();
    assert_eq!(
        distinct.len(),
        triggers.len(),
        "no trigger was consulted twice: {triggers:?}"
    );
    let before: BTreeSet<&String> = triggers[..run.logged_before].iter().collect();
    assert!(
        triggers[run.logged_before..]
            .iter()
            .all(|event| !before.contains(event)),
        "the restart replayed no old trigger"
    );
}

/// The private exchange (Erin ↔ Dev) and Alice's exchange (visitor ↔ Alice) never cross: neither seat's shown
/// memory nor its access rows hold the other's facts, although both live in one home.
pub fn isolated(run: &Run, facts: &Facts) {
    let (_, erin) = run.pair.expect("the second seat");
    let erin = erin.to_string();
    let private: Vec<u64> = spoken(&perceived(MARKET_PACK, &run.save, "dev"))
        .iter()
        .filter(|fact| fact.speaker == erin || fact.listener == erin)
        .map(|fact| fact.id)
        .collect();
    assert!(!private.is_empty(), "Dev was told the private nonce");
    let alice_rows: BTreeSet<u64> = store_rows(&run.store, "alice")
        .2
        .into_iter()
        .map(|row| row.1)
        .collect();
    assert!(
        private.iter().all(|id| !alice_rows.contains(id)),
        "Alice holds no record of the exchange she never perceived"
    );
    for consult in decisions(&run.store, "alice") {
        assert!(
            shown_lines(&consult)
                .iter()
                .all(|line| !line.contains(&run.lines.private)),
            "the private nonce never appears in Alice's shown memory"
        );
    }
    let hers: Vec<u64> = facts
        .by_turn
        .iter()
        .map(|fact| fact.id)
        .chain(facts.alice_lines.iter().map(|fact| fact.id))
        .collect();
    let dev_rows: BTreeSet<u64> = store_rows(&run.store, "dev")
        .2
        .into_iter()
        .map(|row| row.1)
        .collect();
    assert!(
        hers.iter().all(|id| !dev_rows.contains(id)),
        "Dev holds no record of Alice's exchange, although his rows are in the same home"
    );
    for consult in decisions(&run.store, "dev") {
        assert!(
            shown_lines(&consult)
                .iter()
                .all(|line| !line.contains(&run.lines.code)),
            "Alice's code never appears in Dev's shown memory"
        );
    }
    let home = home_file(&run.store);
    assert!(
        home.is_file(),
        "both seats' rows are in one home: {}",
        home.display()
    );
}

/// Another resident acted while the conversation ran: a journaled request between the first and the last
/// player line whose actor is none of the people the test plays or the LM drives.
pub fn others_acted(run: &Run, facts: &Facts) -> usize {
    let (first, last) = (
        facts.by_turn[0].action.expect("caused"),
        facts.by_turn.last().expect("lines").action.expect("caused"),
    );
    let mut ours = vec![run.alice, run.visitor];
    ours.extend(run.pair.iter().flat_map(|(dev, erin)| [*dev, *erin]));
    let ours: BTreeSet<String> = ours.iter().map(ToString::to_string).collect();
    let connection =
        rusqlite::Connection::open(Path::new(&run.save).join("world.sqlite")).expect("the save");
    let mut statement = connection
        .prepare("SELECT entry FROM journal WHERE action_id > ?1 AND action_id < ?2")
        .expect("the journal");
    statement
        .query_map(
            [
                i64::try_from(first).expect("small"),
                i64::try_from(last).expect("small"),
            ],
            |row| row.get::<_, Vec<u8>>(0),
        )
        .expect("rows")
        .filter_map(Result::ok)
        .filter_map(|entry| serde_json::from_slice::<Value>(&entry).ok())
        .filter(|entry| {
            entry["input"]["dispatch"]["intent"]["actor"]
                .as_str()
                .is_some_and(|actor| !ours.contains(actor))
        })
        .count()
}

/// Writes the run's evidence into `out`: transcript, observations, machine checks, decision logs, cassettes.
pub fn write_evidence(
    run: &Run,
    out: &Path,
    checks: &[(&'static str, bool, String)],
    calls: Value,
) {
    std::fs::create_dir_all(out).expect("the output directory");
    let transcript: Vec<Value> = run
        .turns
        .iter()
        .map(|turn| {
            json!({ "turn": turn.n, "player": turn.player, "world_at": turn.world_at, "said": turn.words,
                    "reply": turn.reply, "wall_ms": turn.wall.as_millis(),
                    "outcome": turn.consult.as_ref().map(|c| c["record"]["outcome"].clone()) })
        })
        .collect();
    let evidence = json!({
        "lines": { "code": run.lines.code, "old": run.lines.old, "new": run.lines.new },
        "transcript": transcript,
        "stopped_at_turn": run.stopped_at,
        "calls": calls,
        "observations": run.observations.iter().map(|(k, v)| json!({ k: v })).collect::<Vec<_>>(),
        "machine_checks": checks.iter().map(|(name, holds, detail)|
            json!({ "check": name, "holds": holds, "detail": detail })).collect::<Vec<_>>(),
    });
    std::fs::write(
        out.join("evidence.json"),
        serde_json::to_string_pretty(&evidence).expect("JSON"),
    )
    .expect("evidence");
    for (from, to) in [
        ("store/alice.decisions.jsonl", "alice.decisions.jsonl"),
        ("cassette-1.jsonl", "cassette-1.jsonl"),
        ("cassette-2.jsonl", "cassette-2.jsonl"),
        ("cognition-1.toml", "cognition-1.toml"),
        ("cognition-2.toml", "cognition-2.toml"),
    ] {
        let source = run.root.join(from);
        if source.is_file() {
            std::fs::copy(&source, out.join(to)).expect("copied");
        }
    }
}
