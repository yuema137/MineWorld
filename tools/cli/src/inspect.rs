//! `mineworld inspect` — what a save holds, and whether every fact in it has a cause (`AC-9`).
//!
//! Reads the save and nothing else: it never resumes the world and never writes a row. It knows no
//! domain concept — it names event types and causes as the log states them, so it works on any world
//! a pack composes.
//!
//! ```text
//! the causation check (MVP.md §9 AC-9, MODULE_SPEC §8.1)
//!   caused by an action     its ActionId is one some journaled request carried
//!   caused by an event      that event is an earlier fact in the log
//!   caused by genesis       the fact was recorded at genesis (revision 1)
//!   process / system tick   counted: the log alone cannot resolve them, and the output says so
//! ```
//!
//! The check runs over the whole log rather than a sample, since reading it all is cheap.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mineworld_contracts::{ActionResult, Causation, EventEnvelope};
use mineworld_persistence::{
    Durability, JournalEntry, Manifest, Outcome, PersistError, PersistenceBackend, SqliteBackend,
    WorldInput, WorldRevision, format,
};

const DAY: i64 = 86_400;

fn damaged(error: PersistError) -> String {
    format!("[mineworld] {error}")
}

/// Reports the save in `save`, printing the last `last` facts.
pub fn inspect(save: &Path, last: usize) -> Result<(), String> {
    if !SqliteBackend::exists(save) {
        return Err(format!(
            "[mineworld] {} holds no save (no {})",
            save.display(),
            SqliteBackend::file(save).display()
        ));
    }
    let backend = SqliteBackend::open(save, Durability::ProcessCrash).map_err(damaged)?;
    let manifest_row = backend.manifest().map_err(damaged)?;
    format::check_format(manifest_row.format).map_err(damaged)?;
    let manifest: Manifest = format::decode(&manifest_row.body, "manifest").map_err(damaged)?;
    let head = backend.head().map_err(damaged)?;

    let journal = Journal::read(&backend)?;
    let all = usize::try_from(i64::MAX).unwrap_or(usize::MAX);
    let facts: Vec<EventEnvelope> = backend
        .last_facts(all)
        .map_err(damaged)?
        .iter()
        .map(|fact| format::decode(&fact.bytes, "fact"))
        .collect::<Result<_, _>>()
        .map_err(damaged)?;
    let genesis: BTreeSet<u64> = backend
        .facts_of(WorldRevision::GENESIS)
        .map_err(damaged)?
        .iter()
        .map(|fact| fact.id)
        .collect();

    println!(
        "save       {} (format {})",
        SqliteBackend::file(save).display(),
        manifest_row.format
    );
    println!(
        "world      {}, instance {}",
        manifest.pack, manifest.instance
    );
    let systems: Vec<String> = manifest
        .composition
        .iter()
        .map(|installed| {
            format!(
                "{} {}{}",
                installed.declaration.system(),
                installed.declaration.version(),
                if installed.enabled { "" } else { " (disabled)" },
            )
        })
        .collect();
    println!("systems    {}", systems.join(", "));
    println!(
        "head       revision {} at t{} (day {}, {})",
        head.raw(),
        journal.head_at,
        journal.head_at / DAY + 1,
        clock(journal.head_at),
    );
    for (kind, count) in &journal.kinds {
        println!("journal    {kind} {count}");
    }

    let mut by_type: BTreeMap<&str, u64> = BTreeMap::new();
    for fact in &facts {
        *by_type.entry(fact.event_type().as_str()).or_default() += 1;
    }
    println!("facts      {}", facts.len());
    for (event_type, count) in &by_type {
        println!("           {event_type} {count}");
    }

    let checked = Causes::check(&facts, &journal.actions, &genesis);
    for (kind, count) in &checked.by_kind {
        println!("causes     {kind} {count}");
    }
    let shown = facts.len().saturating_sub(last);
    println!("last       {} fact(s)", facts.len() - shown);
    for fact in &facts[shown..] {
        println!(
            "  #{:<8} t{:<10} {:<22} caused by {}",
            fact.id().raw(),
            fact.at().seconds(),
            fact.event_type().as_str(),
            cause(fact.caused_by()),
        );
    }

    if checked.unresolved.is_empty() {
        println!(
            "AC-9       every cause resolves: {} fact(s) checked; {} caused by a process or a \
             system tick are counted, not resolved — the log alone cannot",
            facts.len(),
            checked.unresolvable,
        );
        Ok(())
    } else {
        let shown: Vec<String> = checked.unresolved.iter().take(10).cloned().collect();
        Err(format!(
            "[mineworld] AC-9: {} fact(s) have a cause the save cannot resolve, first: {}",
            checked.unresolved.len(),
            shown.join("; "),
        ))
    }
}

/// The journal, counted, and the request identities it carries.
struct Journal {
    kinds: BTreeMap<String, u64>,
    actions: BTreeSet<u64>,
    head_at: i64,
}

impl Journal {
    fn read(backend: &SqliteBackend) -> Result<Self, String> {
        let mut journal = Self {
            kinds: BTreeMap::new(),
            actions: BTreeSet::new(),
            head_at: 0,
        };
        for (_, bytes) in backend
            .journal_after(WorldRevision::from_raw(0))
            .map_err(damaged)?
        {
            let entry: JournalEntry = format::decode(&bytes, "journal entry").map_err(damaged)?;
            journal.head_at = entry.input.at().seconds();
            let kind = match (&entry.input, &entry.outcome) {
                (WorldInput::Genesis { .. }, _) => "genesis".to_owned(),
                (WorldInput::Advance { .. }, Outcome::Fault(_)) => "advance faulted".to_owned(),
                (WorldInput::Advance { .. }, _) => "advance".to_owned(),
                (WorldInput::Dispatch { intent, .. }, outcome) => {
                    journal.actions.insert(intent.action_id().raw());
                    let action = intent.action_type().as_str();
                    match outcome {
                        Outcome::Dispatched(ActionResult::Accepted { .. }) => {
                            format!("request {action} accepted")
                        }
                        Outcome::Dispatched(ActionResult::Rejected(reason)) => {
                            format!("request {action} rejected {reason:?}")
                        }
                        Outcome::Dispatched(ActionResult::Unavailable) => {
                            format!("request {action} unavailable")
                        }
                        _ => format!("request {action} faulted"),
                    }
                }
            };
            *journal.kinds.entry(kind).or_default() += 1;
        }
        Ok(journal)
    }
}

/// The `AC-9` check's result.
struct Causes {
    by_kind: BTreeMap<&'static str, u64>,
    unresolved: Vec<String>,
    unresolvable: u64,
}

impl Causes {
    fn check(facts: &[EventEnvelope], actions: &BTreeSet<u64>, genesis: &BTreeSet<u64>) -> Self {
        let mut checked = Self {
            by_kind: BTreeMap::new(),
            unresolved: Vec::new(),
            unresolvable: 0,
        };
        let mut seen: BTreeSet<u64> = BTreeSet::new();
        for fact in facts {
            let id = fact.id().raw();
            let (kind, resolved) = match fact.caused_by() {
                Causation::Action(action) => ("action", actions.contains(&action.raw())),
                Causation::Event(event) => ("event", seen.contains(&event.raw())),
                Causation::WorldGenesis => ("world genesis", genesis.contains(&id)),
                Causation::Process(_) => {
                    checked.unresolvable += 1;
                    ("process", true)
                }
                Causation::SystemTick { .. } => {
                    checked.unresolvable += 1;
                    ("system tick", true)
                }
            };
            *checked.by_kind.entry(kind).or_default() += 1;
            if !resolved {
                checked
                    .unresolved
                    .push(format!("fact #{id} caused by {}", cause(fact.caused_by())));
            }
            seen.insert(id);
        }
        checked
    }
}

fn cause(causation: &Causation) -> String {
    match causation {
        Causation::Action(action) => format!("action {}", action.raw()),
        Causation::Process(process) => format!("process {}", process.raw()),
        Causation::Event(event) => format!("event #{}", event.raw()),
        Causation::SystemTick { system } => format!("a tick of {system}"),
        Causation::WorldGenesis => "world genesis".to_owned(),
    }
}

/// `hh:mm:ss` of a day, for a person reading the head's instant.
fn clock(seconds: i64) -> String {
    let of_day = seconds.rem_euclid(DAY);
    format!(
        "{:02}:{:02}:{:02}",
        of_day / 3_600,
        of_day % 3_600 / 60,
        of_day % 60
    )
}
