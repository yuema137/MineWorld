//! `mineworld run` — a World Pack run headless, every seat driven by a seeded rule (`ARC-27`).
//!
//! ```text
//! for each consult instant t, in time order       seat k at genesis + k + m·PACE
//!     advance the world to t                       whatever was due fires first
//!     observe(world, seat, t)                      the pack's own perception
//!     PacedRuleController::decide(observation)     a pure function of seed and observation
//!     ActionIntent::allocate(request, id, t)       the server's step, with this run's allocator
//!     dispatch(intent, t)                          journaled first when the world is saved
//! ```
//!
//! Why not the server: a hosted world's time follows the wall clock and its observations are
//! delivered by `try_send` to tasks, so which observation a controller decides on is a matter of
//! timing (step-08 §2.3). This loop is synchronous, on one thread, and calls only what the server
//! calls — so everything it prints before the `wall` line, and everything it saves, is a function of
//! the pack, the seed, the age and the code (step-08 I-2).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, EntityId, EntityKey, EventEnvelope, SimDuration,
    WorldTime,
};
use mineworld_kernel::{Dispatched, KernelError, World};
use mineworld_persistence::{
    Creation, Durability, JournalEntry, PersistError, PersistenceBackend, PersistentWorld,
    SqliteBackend, WorldInput, WorldRevision, format,
};
use mineworld_presence::{PerceptionProvider, observe};
use mineworld_rule_controller::PacedRuleController;
use mineworld_server::WorldInstanceId;
use mineworld_worldpack::{PackRoots, WorldPack};

use crate::{described, saved_genesis};

/// How often each seat is consulted: every fifteen simulated minutes. Ten in S7 (step-08 HD-2); raised
/// when the twelve-person town made one 300-day debug run take 77 s, under the rule that the pace
/// rises before the days fall (step-08 Q10, step-09 Q4; `DECISIONS.md` `ARC-27` note).
pub const PACE: SimDuration = SimDuration::from_seconds(900);

/// A simulated day, in seconds: the command's unit, never the kernel's (`INV-12`).
const DAY: i64 = 86_400;

/// The width of an activity bucket in the summary, in days: the unit step-08 I-9 is checked in.
const BUCKET_DAYS: i64 = 30;

/// What `run` was asked.
pub struct RunRequest {
    pub world: PathBuf,
    pub seed: u64,
    pub days: u64,
    pub save: Option<PathBuf>,
    /// Where the world's requirements are resolved (`ARC-54`). Never saved: a resume resolves again.
    pub roots: PackRoots,
}

/// Runs the world and prints what happened.
pub fn run(request: &RunRequest) -> Result<(), String> {
    let started = Instant::now();
    let pack = WorldPack::read_with(&request.world, &request.roots).map_err(described)?;
    let seats: Vec<EntityKey> = pack.seats().iter().cloned().collect();
    if seats.is_empty() {
        return Err(format!(
            "[mineworld] {} offers no seat, so nothing in it would act",
            pack.id()
        ));
    }
    if i64::try_from(seats.len()).unwrap_or(i64::MAX) >= PACE.seconds() {
        return Err(format!(
            "[mineworld] {} offers {} seats; a headless run consults at most {} (one per second of \
             its pace)",
            pack.id(),
            seats.len(),
            PACE.seconds() - 1,
        ));
    }
    let days = i64::try_from(request.days)
        .ok()
        .filter(|days| *days >= 1 && days.checked_mul(DAY).is_some())
        .ok_or_else(|| format!("[mineworld] --days {} is out of range", request.days))?;

    let mut begun = Begun::new(&pack, request.save.as_deref())?;
    let genesis = begun.genesis;
    let end = genesis + days * DAY;
    println!(
        "[mineworld] run {} ({}): seats {}; seed {}; to day {}; pace {}; {}",
        pack.name(),
        pack.id(),
        listed(&seats),
        request.seed,
        days,
        PACE,
        begun.how,
    );

    let observers: Vec<EntityId> = seats
        .iter()
        .map(|seat| {
            begun
                .driven
                .world()
                .read()
                .resolve_key(seat)
                .map_err(|error| format!("[mineworld] seat {seat}: {error}"))
        })
        .collect::<Result<_, _>>()?;
    let controller = PacedRuleController::new(request.seed, PACE);
    let mut tally = Tally::new(&begun.began);
    let seat_count = i64::try_from(seats.len()).unwrap_or(i64::MAX);

    let mut next_day = ((begun.start - genesis) / DAY + 1).max(1);
    let first_round = ((begun.start - genesis - seat_count) / PACE.seconds()).max(0);
    'rounds: for round in first_round.. {
        for (k, (seat, observer)) in seats.iter().zip(&observers).enumerate() {
            let at = genesis + i64::try_from(k).unwrap_or(0) + round * PACE.seconds();
            if at >= end {
                break 'rounds;
            }
            if at < begun.start {
                continue;
            }
            while next_day <= days && genesis + next_day * DAY <= at {
                tally.day(next_day, begun.driven.revision());
                next_day += 1;
            }
            let fired = begun.driven.advance_to(WorldTime::from_seconds(at))?;
            tally.facts(&fired);

            let borrowed: Vec<&dyn PerceptionProvider> =
                begun.providers.iter().map(AsRef::as_ref).collect();
            let seen = observe(
                begun.driven.world(),
                *observer,
                WorldTime::from_seconds(at),
                &borrowed,
            );
            tally.consults += 1;
            let Some(decided) = controller.decide(&seen) else {
                continue;
            };
            let action = decided.payload().action_type().as_str().to_owned();
            let intent = ActionIntent::allocate(
                decided,
                ActionId::from_raw(begun.next_action),
                WorldTime::from_seconds(at),
            );
            begun.next_action += 1;
            match begun
                .driven
                .dispatch(&intent, WorldTime::from_seconds(at))?
            {
                Some(dispatched) => {
                    let bucket = (at - genesis) / (BUCKET_DAYS * DAY);
                    tally.answered(seat, bucket, &action, &dispatched);
                }
                None => tally.faults += 1,
            }
        }
    }
    let fired = begun.driven.advance_to(WorldTime::from_seconds(end))?;
    tally.facts(&fired);
    while next_day <= days {
        tally.day(next_day, begun.driven.revision());
        next_day += 1;
    }

    // A saved world's history includes what earlier invocations recorded, so it is read back whole.
    let saved = match &request.save {
        None => None,
        Some(save) => Some(Fingerprint::of_save(save)?),
    };
    let history = saved.as_ref().unwrap_or(&tally.history);
    tally.report(&seats, days, history, begun.how.starts_with("resumed"));
    println!("wall       {:.1} s", started.elapsed().as_secs_f64());
    Ok(())
}

/// The world as it stands before the first consult of this invocation, and where to begin.
struct Begun {
    driven: Driven,
    providers: Vec<Box<dyn PerceptionProvider>>,
    /// The world's genesis instant, in seconds: the origin of the pace schedule and of days.
    genesis: i64,
    /// The first instant this invocation may consult at (step-08 HD-7).
    start: i64,
    next_action: u64,
    /// The genesis facts, when this invocation began the world.
    began: Vec<EventEnvelope>,
    how: String,
}

impl Begun {
    fn new(pack: &WorldPack, save: Option<&Path>) -> Result<Self, String> {
        let at = WorldTime::EPOCH;
        let Some(save) = save else {
            let loaded = pack.load(at).map_err(described)?;
            let began = loaded.genesis().to_vec();
            let running = loaded.into_running();
            return Ok(Self {
                driven: Driven::Memory(running.world),
                providers: running.providers,
                genesis: at.seconds(),
                start: at.seconds(),
                next_action: 1,
                began,
                how: "in memory".to_owned(),
            });
        };
        let file = SqliteBackend::file(save).display().to_string();
        if !SqliteBackend::exists(save) {
            let backend = SqliteBackend::create(save, Durability::ProcessCrash).map_err(stopped)?;
            let assembled = pack.assemble().map_err(described)?;
            let (world, began) = PersistentWorld::create(
                Box::new(backend),
                assembled.world,
                Creation {
                    instance: WorldInstanceId::allocate().raw(),
                    pack: pack.id().to_owned(),
                    at,
                    facts: assembled.facts,
                },
            )
            .map_err(stopped)?;
            return Ok(Self {
                driven: Driven::Saved(world),
                providers: assembled.providers,
                genesis: at.seconds(),
                start: at.seconds(),
                next_action: 1,
                began,
                how: format!("created {file}"),
            });
        }

        let backend = SqliteBackend::open(save, Durability::ProcessCrash).map_err(stopped)?;
        let genesis = genesis_instant(&backend)?;
        let head = backend.head().map_err(stopped)?;
        let head_was_a_request = head_input(&backend, head)?;
        pack.check_configuration(&saved_genesis(&backend).map_err(stopped)?)
            .map_err(described)?;
        let composed = pack.compose().map_err(described)?;
        let (world, resumed) =
            PersistentWorld::resume(Box::new(backend), composed.world).map_err(stopped)?;
        let now = world.world().now().seconds();
        let next_action = world
            .highest_action_id()
            .map_err(stopped)?
            .map_or(1, |highest| highest + 1);
        Ok(Self {
            driven: Driven::Saved(world),
            providers: composed.providers,
            genesis,
            // A request at the head was that consult's; anything else leaves the head's instant to
            // be consulted (step-08 HD-7).
            start: if head_was_a_request { now + 1 } else { now },
            next_action,
            began: Vec::new(),
            how: format!(
                "resumed {file} at revision {} (snapshot {} + {} re-executed)",
                resumed.head.raw(),
                resumed.snapshot.raw(),
                resumed.replayed,
            ),
        })
    }
}

/// The instant a saved world began at: its genesis facts', or genesis's own journal row.
fn genesis_instant(backend: &SqliteBackend) -> Result<i64, String> {
    if let Some(first) = backend
        .facts_of(WorldRevision::GENESIS)
        .map_err(stopped)?
        .first()
    {
        let fact: EventEnvelope = format::decode(&first.bytes, "fact").map_err(stopped)?;
        return Ok(fact.at().seconds());
    }
    let first = backend
        .journal_after(WorldRevision::from_raw(0))
        .map_err(stopped)?
        .into_iter()
        .next()
        .ok_or_else(|| "[mineworld] the save has no genesis".to_owned())?;
    let entry: JournalEntry = format::decode(&first.1, "journal entry").map_err(stopped)?;
    Ok(entry.input.at().seconds())
}

/// Whether the head revision's input was a request.
fn head_input(backend: &SqliteBackend, head: WorldRevision) -> Result<bool, String> {
    if head == WorldRevision::GENESIS {
        return Ok(false);
    }
    let rows = backend
        .journal_after(WorldRevision::from_raw(head.raw() - 1))
        .map_err(stopped)?;
    let (_, bytes) = rows
        .first()
        .ok_or_else(|| "[mineworld] the save's head has no journal row".to_owned())?;
    let entry: JournalEntry = format::decode(bytes, "journal entry").map_err(stopped)?;
    Ok(matches!(entry.input, WorldInput::Dispatch { .. }))
}

fn stopped(error: PersistError) -> String {
    format!("[mineworld] {error}")
}

/// The world, in memory or with its save.
enum Driven {
    Memory(World),
    Saved(PersistentWorld),
}

impl Driven {
    fn world(&self) -> &World {
        match self {
            Self::Memory(world) => world,
            Self::Saved(world) => world.world(),
        }
    }

    fn revision(&self) -> Option<u64> {
        match self {
            Self::Memory(_) => None,
            Self::Saved(world) => Some(world.revision().raw()),
        }
    }

    /// Advances the clock; a system's fault here stops the run, since nothing asked for it.
    fn advance_to(&mut self, at: WorldTime) -> Result<Vec<EventEnvelope>, String> {
        match self {
            Self::Memory(world) => world
                .advance_to(at)
                .map(mineworld_kernel::Advanced::into_events)
                .map_err(|error| format!("[mineworld] a system broke its own contract: {error}")),
            Self::Saved(world) => world
                .advance_to(at)
                .map(mineworld_kernel::Advanced::into_events)
                .map_err(stopped),
        }
    }

    /// Dispatches; `Ok(None)` is a system's fault, counted and survived as a server survives it;
    /// `Err` is a save that can no longer be written, which stops the run.
    fn dispatch(
        &mut self,
        intent: &ActionIntent,
        at: WorldTime,
    ) -> Result<Option<Dispatched>, String> {
        let fault = |error: KernelError| {
            eprintln!("[mineworld] a system broke its own contract: {error}");
        };
        match self {
            Self::Memory(world) => match world.dispatch(intent, at) {
                Ok(dispatched) => Ok(Some(dispatched)),
                Err(error) => {
                    fault(error);
                    Ok(None)
                }
            },
            Self::Saved(world) => match world.dispatch(intent, at) {
                Ok(dispatched) => Ok(Some(dispatched)),
                Err(PersistError::Kernel(error)) => {
                    fault(error);
                    Ok(None)
                }
                Err(other) => Err(stopped(other)),
            },
        }
    }
}

/// FNV-1a 64 over every fact's stored encoding, in `EventId` order — for a person to compare two
/// runs by eye. Not evidence of equality: tests compare the bytes themselves (step-08 §10.1 Q9).
struct Fingerprint {
    hash: u64,
    facts: u64,
}

impl Fingerprint {
    const fn new() -> Self {
        Self {
            hash: 0xcbf2_9ce4_8422_2325,
            facts: 0,
        }
    }

    fn add(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.facts += 1;
    }

    fn add_fact(&mut self, fact: &EventEnvelope) {
        // The same encoding a save stores (`mineworld_persistence::format::encode`).
        let bytes = format::encode(fact).expect("a fact encodes; it was just recorded");
        self.add(&bytes);
    }

    fn of_save(save: &Path) -> Result<Self, String> {
        let backend = SqliteBackend::open(save, Durability::ProcessCrash).map_err(stopped)?;
        let mut fingerprint = Self::new();
        // "All of them": the largest count the backend can state as a stored (signed) integer.
        let all = usize::try_from(i64::MAX).unwrap_or(usize::MAX);
        for fact in backend.last_facts(all).map_err(stopped)? {
            fingerprint.add(&fact.bytes);
        }
        Ok(fingerprint)
    }
}

/// What this invocation did, counted.
struct Tally {
    consults: u64,
    faults: u64,
    recorded: u64,
    requests: BTreeMap<(String, String), u64>,
    by_type: BTreeMap<String, u64>,
    /// Accepted requests by (bucket, seat, action type).
    activity: BTreeMap<(i64, EntityKey, String), u64>,
    history: Fingerprint,
}

impl Tally {
    fn new(began: &[EventEnvelope]) -> Self {
        let mut tally = Self {
            consults: 0,
            faults: 0,
            recorded: 0,
            requests: BTreeMap::new(),
            by_type: BTreeMap::new(),
            activity: BTreeMap::new(),
            history: Fingerprint::new(),
        };
        tally.facts(began);
        tally
    }

    fn facts(&mut self, facts: &[EventEnvelope]) {
        for fact in facts {
            self.recorded += 1;
            *self
                .by_type
                .entry(fact.event_type().as_str().to_owned())
                .or_default() += 1;
            self.history.add_fact(fact);
        }
    }

    fn answered(&mut self, seat: &EntityKey, bucket: i64, action: &str, dispatched: &Dispatched) {
        let outcome = match dispatched.result() {
            ActionResult::Accepted { .. } => {
                *self
                    .activity
                    .entry((bucket, seat.clone(), action.to_owned()))
                    .or_default() += 1;
                "accepted".to_owned()
            }
            ActionResult::Rejected(reason) => format!("rejected {reason:?}"),
            ActionResult::Unavailable => "unavailable".to_owned(),
        };
        *self
            .requests
            .entry((action.to_owned(), outcome))
            .or_default() += 1;
        self.facts(dispatched.events());
    }

    fn day(&self, day: i64, revision: Option<u64>) {
        let revision = revision.map_or_else(|| "-".to_owned(), |revision| revision.to_string());
        println!("day {day}  revision {revision}  facts {}", self.recorded);
    }

    fn report(&self, seats: &[EntityKey], days: i64, history: &Fingerprint, resumed: bool) {
        let scope = if resumed { " (this invocation)" } else { "" };
        println!("consults   {}{scope}", self.consults);
        for ((action, outcome), count) in &self.requests {
            println!("requests   {action} {outcome} {count}");
        }
        for (event_type, count) in &self.by_type {
            println!("facts      {event_type} {count}");
        }
        let buckets = (days + BUCKET_DAYS - 1) / BUCKET_DAYS;
        for bucket in 0..buckets {
            let first = bucket * BUCKET_DAYS + 1;
            let last = ((bucket + 1) * BUCKET_DAYS).min(days);
            let mut line = format!("activity   days {first}-{last}");
            for seat in seats {
                let count = |action: &str| {
                    self.activity
                        .get(&(bucket, seat.clone(), action.to_owned()))
                        .copied()
                        .unwrap_or(0)
                };
                line.push_str(&format!(
                    "  {seat} move {} talk {}",
                    count("move"),
                    count("talk")
                ));
            }
            println!("{line}");
        }
        println!("faults     {}", self.faults);
        println!(
            "history    {} facts, fingerprint {:016x} (FNV-1a 64, for reading; not evidence)",
            history.facts, history.hash,
        );
    }
}

fn listed(seats: &[EntityKey]) -> String {
    seats
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}
