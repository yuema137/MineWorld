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
//! The check runs over the whole log rather than a sample. The log is streamed, one sealed block at a
//! time (`ARC-89`): the check keeps the set of event ids it has seen, the report the newest N facts.
//!
//! A save may be inspected while a server still commits to it. The journal and the facts are read by
//! two passes, each one read transaction, so the fact pass can see revisions the journal pass did not:
//! their facts would name actions the inspected journal does not hold. Everything is therefore reported
//! for the prefix the journal pass read — its last revision is the head shown, and the fact pass stops
//! after that revision's last fact.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

use mineworld_contracts::{ActionResult, Causation, EventEnvelope};
use mineworld_persistence::backend::SealedSpan;
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
    let sealed = backend.sealed().map_err(damaged)?;
    let sealed_through = sealed.last().map_or(0, |block| block.span.last.raw());
    let journal = Journal::read(&backend, sealed_through)?;
    let head = journal.head;
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
    println!("snapshots  {}", retained(&backend)?);
    println!("log        {}", log_line(&sealed, &journal, head));

    // One streamed pass over the facts (`ARC-89`, `F-S24-1`): counts, the causation check and the
    // newest `last` facts are kept; no fact else is.
    let mut by_type: BTreeMap<String, u64> = BTreeMap::new();
    let mut checked = Causes::new();
    let mut newest: VecDeque<EventEnvelope> = VecDeque::with_capacity(last.min(4_096));
    let mut count: usize = 0;
    for row in backend.facts_after(None).map_err(damaged)? {
        let row = row.map_err(damaged)?;
        if journal.facts_through.is_none_or(|through| row.id > through) {
            // Recorded by a revision committed after the journal pass: not part of what is inspected.
            break;
        }
        let fact: EventEnvelope = format::decode(&row.bytes, "fact").map_err(damaged)?;
        count += 1;
        *by_type
            .entry(fact.event_type().as_str().to_owned())
            .or_default() += 1;
        checked.add(&fact, &journal.actions, &genesis);
        if last > 0 {
            if newest.len() == last {
                newest.pop_front();
            }
            newest.push_back(fact);
        }
    }
    println!("facts      {count}");
    for (event_type, count) in &by_type {
        println!("           {event_type} {count}");
    }

    for (kind, count) in &checked.by_kind {
        println!("causes     {kind} {count}");
    }
    println!("last       {} fact(s)", newest.len());
    for fact in &newest {
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
            "AC-9       every cause resolves: {count} fact(s) checked; {} caused by a process or a \
             system tick are counted, not resolved — the log alone cannot",
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

/// The snapshots the save keeps and what each costs on disk, on one line (`ARC-81`, D-SR-9):
/// `3 kept (zstd): r1 2316 B, r4096 52045 B, r4160 52101 B`.
fn retained(backend: &SqliteBackend) -> Result<String, String> {
    let revisions = backend.snapshot_revisions().map_err(damaged)?;
    let mut kept = Vec::with_capacity(revisions.len());
    for revision in &revisions {
        let bytes = backend
            .snapshot_at(*revision)
            .map_err(damaged)?
            .map_or(0, |stored| stored.len());
        kept.push(format!("{revision} {bytes} B"));
    }
    Ok(format!(
        "{} kept (zstd): {}",
        revisions.len(),
        kept.join(", ")
    ))
}

/// The journal, counted, and the request identities it carries.
struct Journal {
    kinds: BTreeMap<String, u64>,
    actions: BTreeSet<u64>,
    head_at: i64,
    /// The last revision the pass read: the head of the inspected prefix.
    head: WorldRevision,
    /// The newest fact the read revisions recorded, if any.
    facts_through: Option<u64>,
    /// The bytes of the journal entries and facts the sealed blocks hold, uncompressed.
    sealed_row_bytes: u64,
}

impl Journal {
    /// One streamed pass over every revision; rows at or below `sealed_through` are counted as sealed.
    fn read(backend: &SqliteBackend, sealed_through: u64) -> Result<Self, String> {
        let mut journal = Self {
            kinds: BTreeMap::new(),
            actions: BTreeSet::new(),
            head_at: 0,
            head: WorldRevision::GENESIS,
            facts_through: None,
            sealed_row_bytes: 0,
        };
        for stored in backend
            .revisions_after(WorldRevision::from_raw(0))
            .map_err(damaged)?
        {
            let stored = stored.map_err(damaged)?;
            journal.head = stored.revision;
            if let Some(last) = stored.facts.last() {
                journal.facts_through = Some(last.id);
            }
            if stored.revision.raw() <= sealed_through {
                let facts: usize = stored.facts.iter().map(|fact| fact.bytes.len()).sum();
                journal.sealed_row_bytes += (stored.entry.len() + facts) as u64;
            }
            let entry: JournalEntry =
                format::decode(&stored.entry, "journal entry").map_err(damaged)?;
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
    /// Every `EventId` checked so far: 8–40 B a fact, the one thing the check keeps of the whole log.
    seen: BTreeSet<u64>,
}

impl Causes {
    const fn new() -> Self {
        Self {
            by_kind: BTreeMap::new(),
            unresolved: Vec::new(),
            unresolvable: 0,
            seen: BTreeSet::new(),
        }
    }

    /// Checks the next fact of the log, in `EventId` order: an event cause must be an earlier fact.
    fn add(&mut self, fact: &EventEnvelope, actions: &BTreeSet<u64>, genesis: &BTreeSet<u64>) {
        let id = fact.id().raw();
        let (kind, resolved) = match fact.caused_by() {
            Causation::Action(action) => ("action", actions.contains(&action.raw())),
            Causation::Event(event) => ("event", self.seen.contains(&event.raw())),
            Causation::WorldGenesis => ("world genesis", genesis.contains(&id)),
            Causation::Process(_) => {
                self.unresolvable += 1;
                ("process", true)
            }
            Causation::SystemTick { .. } => {
                self.unresolvable += 1;
                ("system tick", true)
            }
        };
        *self.by_kind.entry(kind).or_default() += 1;
        if !resolved {
            self.unresolved
                .push(format!("fact #{id} caused by {}", cause(fact.caused_by())));
        }
        self.seen.insert(id);
    }
}

/// The log's storage on one line (`ARC-89`, D-SRB-10): `146 blocks sealed r1–r598016 (28.1 MB stored,
/// 1012.4 MB of rows); live r598017–r603633`.
fn log_line(sealed: &[SealedSpan], journal: &Journal, head: WorldRevision) -> String {
    let megabytes = |bytes: u64| bytes as f64 / 1_000_000.0;
    let live_from = sealed.last().map_or(1, |block| block.span.last.raw() + 1);
    let live = if live_from <= head.raw() {
        format!("live r{live_from}–{head}")
    } else {
        "nothing live".to_owned()
    };
    match (sealed.first(), sealed.last()) {
        (Some(first), Some(last)) => format!(
            "{} block(s) sealed {}–{} ({:.1} MB stored, {:.1} MB of rows); {live}",
            sealed.len(),
            first.span.first,
            last.span.last,
            megabytes(sealed.iter().map(|block| block.stored_bytes).sum()),
            megabytes(journal.sealed_row_bytes),
        ),
        _ => format!("no block sealed; {live}"),
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
