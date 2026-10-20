//! Shared by the Market Town acceptance tests (CP-4, AC-2 at world level, Milestone C): running the
//! real binary on `worlds/market-town`, and reading the market's facts and components back
//! (step-10 SD-33).
//!
//! # Read by type slug, never through a market crate
//!
//! `AC-1`'s check 2 forbids any code outside `systems/`, `worlds/` and `tests/acceptance/` to name a
//! market pack's crate (`docs/DECISIONS.md` `ARC-35` item 3), and a test that runs the `mineworld`
//! binary must live here. So this module reads market facts and components **by their literal type
//! slugs**, into test-local mirrors over `mineworld-contracts`' id types: the schema version must be
//! the one [`MARKET_SLUGS`] names, and the payload must have exactly the mirror's fields — an unknown or a missing field fails
//! loudly, as `#[serde(deny_unknown_fields)]` would. Literal slugs and shapes are also the oracle
//! the test rules ask for: independent of the code under test (rules §25).

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use mineworld_contracts::{
    Causation, EntityId, EntityType, EventEnvelope, ItemId, OrganizationId, PersonId, PlaceId,
    WorldTime,
};
use mineworld_persistence::{Durability, PersistenceBackend, SqliteBackend, WorldRevision, format};
use serde_json::{Map, Value};

use crate::headless::{mineworld, stderr, stdout};

/// The repository's Market Town.
pub const MARKET_TOWN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");

/// A person carries at most this many items, every kind together: inventory's rule, as a literal
/// (`ARC-37` item 5), so a change to the rule is caught here rather than followed.
pub const PERSON_CAPACITY: u32 = 6;

/// Seconds in a 30-day bucket.
pub const BUCKET: i64 = 30 * 86_400;

/// `run <pack> --headless` with these settings, which must succeed; its stdout.
pub fn run_pack(pack: &Path, seed: u64, days: u64, save: Option<&Path>) -> String {
    let (seed, days) = (seed.to_string(), days.to_string());
    let pack = pack.to_str().expect("a printable path").to_owned();
    let mut arguments = vec!["run", &pack, "--headless", "--seed", &seed, "--days", &days];
    let save = save.map(|save| save.to_str().expect("a printable path").to_owned());
    if let Some(save) = &save {
        arguments.extend(["--save", save]);
    }
    let output = mineworld(&arguments);
    assert!(
        output.status.success(),
        "run failed: {}\n{}",
        stderr(&output),
        stdout(&output)
    );
    stdout(&output)
}

/// [`run_pack`] on the repository's Market Town.
pub fn run_market(seed: u64, days: u64, save: Option<&Path>) -> String {
    run_pack(Path::new(MARKET_TOWN), seed, days, save)
}

/// The pack's identities by key, and the seats it offers, read from the pack itself.
pub struct Town {
    pub ids: BTreeMap<String, EntityId>,
    pub people: BTreeSet<EntityId>,
    pub seats: Vec<String>,
    /// The people whose file carries a `job:` section, read from the files, not from the run.
    pub job_holders: BTreeSet<EntityId>,
}

impl Town {
    pub fn read(pack: &Path) -> Self {
        let read = mineworld_worldpack::WorldPack::read(pack).expect("the World Pack reads");
        let seats = read
            .seats()
            .iter()
            .map(|seat| seat.as_str().to_owned())
            .collect();
        let loaded = read.load(WorldTime::EPOCH).expect("it loads");
        let ids: BTreeMap<String, EntityId> = loaded
            .ids()
            .iter()
            .map(|(key, id)| (key.as_str().to_owned(), *id))
            .collect();
        let people = ids
            .values()
            .copied()
            .filter(|id| {
                loaded
                    .world()
                    .read()
                    .entity(*id)
                    .is_some_and(|entity| entity.entity_type() == EntityType::Person)
            })
            .collect();
        let job_holders = read
            .people()
            .keys()
            .filter(|key| {
                let file = pack.join("people").join(format!("{}.yaml", key.as_str()));
                std::fs::read_to_string(&file)
                    .expect("a person file reads")
                    .lines()
                    .any(|line| line.starts_with("job:"))
            })
            .map(|key| ids[key.as_str()])
            .collect();
        Self {
            ids,
            people,
            seats,
            job_holders,
        }
    }

    pub fn id(&self, key: &str) -> EntityId {
        *self
            .ids
            .get(key)
            .unwrap_or_else(|| panic!("{key} is in the pack"))
    }

    pub fn person(&self, key: &str) -> PersonId {
        PersonId::new(self.id(key), EntityType::Person).expect("a person")
    }

    pub fn key(&self, id: EntityId) -> String {
        self.ids
            .iter()
            .find(|(_, candidate)| **candidate == id)
            .map_or_else(|| format!("#{}", id.raw()), |(key, _)| key.clone())
    }

    pub fn seat_ids(&self) -> Vec<(String, EntityId)> {
        self.seats
            .iter()
            .map(|seat| (seat.clone(), self.id(seat)))
            .collect()
    }
}

// ── Mirrors ──────────────────────────────────────────────────────────────────────────────────────

/// A payload's fields, which must be exactly `expected`: `deny_unknown_fields`, and no field missing.
fn exactly(value: &Value, what: &str, expected: &[&str]) -> Map<String, Value> {
    let map = value
        .as_object()
        .unwrap_or_else(|| panic!("{what}: not an object: {value}"))
        .clone();
    let found: BTreeSet<&str> = map.keys().map(String::as_str).collect();
    let wanted: BTreeSet<&str> = expected.iter().copied().collect();
    assert_eq!(
        found, wanted,
        "{what}: its fields changed shape — the mirror must change with it: {value}"
    );
    map
}

fn entity(map: &Map<String, Value>, key: &str) -> EntityId {
    serde_json::from_value(map[key].clone()).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn person(map: &Map<String, Value>, key: &str) -> PersonId {
    serde_json::from_value(map[key].clone()).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn organization(map: &Map<String, Value>, key: &str) -> OrganizationId {
    serde_json::from_value(map[key].clone()).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn item(map: &Map<String, Value>, key: &str) -> ItemId {
    serde_json::from_value(map[key].clone()).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn place(map: &Map<String, Value>, key: &str) -> PlaceId {
    serde_json::from_value(map[key].clone()).unwrap_or_else(|error| panic!("{key}: {error}"))
}

fn number(map: &Map<String, Value>, key: &str) -> u64 {
    map[key]
        .as_u64()
        .unwrap_or_else(|| panic!("{key}: not an unsigned integer: {}", map[key]))
}

fn count(map: &Map<String, Value>, key: &str) -> u32 {
    u32::try_from(number(map, key)).expect("a count fits in u32")
}

/// One market fact, mirrored from its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketFact {
    Funded {
        holder: EntityId,
        balance: u64,
    },
    ShopOpened {
        place: PlaceId,
        operator: OrganizationId,
        keepers: Vec<PersonId>,
    },
    Listed {
        seller: EntityId,
        item: ItemId,
        price: u64,
    },
    MoneyTransferred {
        from: EntityId,
        to: EntityId,
        amount: u64,
    },
    Sold {
        seller: EntityId,
        buyer: EntityId,
        item: ItemId,
        count: u32,
        price: u64,
        place: PlaceId,
    },
    WageDue {
        employee: PersonId,
        employer: OrganizationId,
        amount: u64,
    },
    WageUnpaid {
        employee: PersonId,
        employer: OrganizationId,
        amount: u64,
    },
    Hired {
        employee: PersonId,
        employer: OrganizationId,
        workplace: PlaceId,
    },
    Stocked {
        holder: EntityId,
        item: ItemId,
        count: u32,
    },
    ItemsTransferred {
        from: EntityId,
        to: EntityId,
        item: ItemId,
        count: u32,
    },
    ItemsProduced {
        holder: EntityId,
        item: ItemId,
        count: u32,
    },
    ItemsConsumed {
        holder: EntityId,
        item: ItemId,
        count: u32,
    },
}

/// The market fact types the tests read, by slug, each with the schema version the mirror reads.
/// `shop-opened` is at 2 since S31-a: its prices left it, as the operator's `listed` (`ARC-112`).
pub const MARKET_SLUGS: [(&str, u32); 12] = [
    ("funded", 1),
    ("shop-opened", 2),
    ("listed", 1),
    ("sold", 1),
    ("money-transferred", 1),
    ("wage-due", 1),
    ("wage-unpaid", 1),
    ("hired", 1),
    ("stocked", 1),
    ("items-transferred", 1),
    ("items-produced", 1),
    ("items-consumed", 1),
];

/// A fact as a [`MarketFact`], if its type is one of [`MARKET_SLUGS`]; decoded only after its slug
/// matched, and only at that slug's schema version.
pub fn market_fact(fact: &EventEnvelope) -> Option<MarketFact> {
    let slug = fact.event_type().as_str();
    let (_, version) = MARKET_SLUGS.iter().find(|(known, _)| *known == slug)?;
    let record = fact.payload();
    assert_eq!(
        record.schema_version().get(),
        *version,
        "#{} {slug}: schema version {} — the mirror reads version {version}",
        fact.id().raw(),
        record.schema_version().get()
    );
    let value: Value = serde_json::from_slice(record.payload())
        .unwrap_or_else(|error| panic!("#{} {slug}: {error}", fact.id().raw()));
    let what = format!("#{} {slug}", fact.id().raw());
    let quantity = |map: &Map<String, Value>| {
        (
            entity(map, "holder"),
            item(map, "item"),
            count(map, "count"),
        )
    };
    Some(match slug {
        "funded" => {
            let map = exactly(&value, &what, &["holder", "balance"]);
            MarketFact::Funded {
                holder: entity(&map, "holder"),
                balance: number(&map, "balance"),
            }
        }
        "shop-opened" => {
            let map = exactly(&value, &what, &["place", "operator", "keepers"]);
            let keepers = map["keepers"]
                .as_array()
                .expect("keepers is a list")
                .iter()
                .map(|keeper| {
                    serde_json::from_value(keeper.clone())
                        .unwrap_or_else(|error| panic!("{what} keeper: {error}"))
                })
                .collect();
            MarketFact::ShopOpened {
                place: place(&map, "place"),
                operator: organization(&map, "operator"),
                keepers,
            }
        }
        "listed" => {
            let map = exactly(&value, &what, &["seller", "item", "price"]);
            MarketFact::Listed {
                seller: entity(&map, "seller"),
                item: item(&map, "item"),
                price: number(&map, "price"),
            }
        }
        "sold" => {
            let map = exactly(
                &value,
                &what,
                &["seller", "buyer", "item", "count", "price", "place"],
            );
            MarketFact::Sold {
                seller: entity(&map, "seller"),
                buyer: entity(&map, "buyer"),
                item: item(&map, "item"),
                count: count(&map, "count"),
                price: number(&map, "price"),
                place: place(&map, "place"),
            }
        }
        "money-transferred" => {
            let map = exactly(&value, &what, &["from", "to", "amount"]);
            MarketFact::MoneyTransferred {
                from: entity(&map, "from"),
                to: entity(&map, "to"),
                amount: number(&map, "amount"),
            }
        }
        "wage-due" | "wage-unpaid" => {
            let map = exactly(&value, &what, &["employee", "employer", "amount"]);
            let (employee, employer, amount) = (
                person(&map, "employee"),
                organization(&map, "employer"),
                number(&map, "amount"),
            );
            if slug == "wage-due" {
                MarketFact::WageDue {
                    employee,
                    employer,
                    amount,
                }
            } else {
                MarketFact::WageUnpaid {
                    employee,
                    employer,
                    amount,
                }
            }
        }
        "hired" => {
            let map = exactly(&value, &what, &["employee", "job"]);
            let job = exactly(
                &map["job"],
                &format!("{what} job"),
                &["employer", "workplace", "from", "until", "wage", "produces"],
            );
            MarketFact::Hired {
                employee: person(&map, "employee"),
                employer: organization(&job, "employer"),
                workplace: place(&job, "workplace"),
            }
        }
        "items-transferred" => {
            let map = exactly(&value, &what, &["from", "to", "item", "count"]);
            MarketFact::ItemsTransferred {
                from: entity(&map, "from"),
                to: entity(&map, "to"),
                item: item(&map, "item"),
                count: count(&map, "count"),
            }
        }
        _ => {
            let map = exactly(&value, &what, &["holder", "item", "count"]);
            let (holder, item, count) = quantity(&map);
            match slug {
                "stocked" => MarketFact::Stocked {
                    holder,
                    item,
                    count,
                },
                "items-produced" => MarketFact::ItemsProduced {
                    holder,
                    item,
                    count,
                },
                _ => MarketFact::ItemsConsumed {
                    holder,
                    item,
                    count,
                },
            }
        }
    })
}

/// A disclosed or stored `wallet` component's balance (`{ balance }`).
pub fn wallet_balance(value: &Value) -> u64 {
    number(&exactly(value, "wallet", &["balance"]), "balance")
}

/// A `holdings` component (`{ held: [{ item, count }] }`), by item.
pub fn holdings(value: &Value) -> BTreeMap<EntityId, u32> {
    let map = exactly(value, "holdings", &["held"]);
    map["held"]
        .as_array()
        .expect("held is a list")
        .iter()
        .map(|held| {
            let held = exactly(held, "held", &["item", "count"]);
            (item(&held, "item").entity_id(), count(&held, "count"))
        })
        .collect()
}

/// A disclosed `shop` listing (`{ operator, listed: [{ item, price, in_stock }] }`): the operator,
/// and per item its price and stock.
pub fn listing(value: &Value) -> (OrganizationId, BTreeMap<EntityId, (u64, u32)>) {
    let map = exactly(value, "shop listing", &["operator", "listed"]);
    let listed = map["listed"]
        .as_array()
        .expect("listed is a list")
        .iter()
        .map(|listed| {
            let listed = exactly(listed, "listed", &["item", "price", "in_stock"]);
            (
                item(&listed, "item").entity_id(),
                (number(&listed, "price"), count(&listed, "in_stock")),
            )
        })
        .collect();
    (organization(&map, "operator"), listed)
}

// ── The ledger: wallets and holdings replayed from the facts ─────────────────────────────────────

/// Wallets and holdings, replayed from `funded`, `money-transferred`, `stocked`, `items-transferred`,
/// `items-produced` and `items-consumed` in fact order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Ledger {
    pub wallets: BTreeMap<EntityId, u64>,
    pub holdings: BTreeMap<EntityId, BTreeMap<EntityId, u32>>,
}

impl Ledger {
    /// Applies one fact; a debit below zero is a failure of the history, named.
    pub fn apply(&mut self, fact: &MarketFact) -> Result<(), String> {
        match *fact {
            MarketFact::Funded { holder, balance } => {
                self.wallets.insert(holder, balance);
            }
            MarketFact::MoneyTransferred { from, to, amount } => {
                let payer = self.wallets.entry(from).or_default();
                *payer = payer
                    .checked_sub(amount)
                    .ok_or_else(|| format!("#{} paid {amount} holding {payer}", from.raw()))?;
                *self.wallets.entry(to).or_default() += amount;
            }
            MarketFact::Stocked {
                holder,
                item,
                count,
            }
            | MarketFact::ItemsProduced {
                holder,
                item,
                count,
            } => self.add(holder, item.entity_id(), count),
            MarketFact::ItemsConsumed {
                holder,
                item,
                count,
            } => self.remove(holder, item.entity_id(), count)?,
            MarketFact::ItemsTransferred {
                from,
                to,
                item,
                count,
            } => {
                self.remove(from, item.entity_id(), count)?;
                self.add(to, item.entity_id(), count);
            }
            _ => {}
        }
        Ok(())
    }

    fn add(&mut self, holder: EntityId, item: EntityId, count: u32) {
        *self
            .holdings
            .entry(holder)
            .or_default()
            .entry(item)
            .or_default() += count;
    }

    fn remove(&mut self, holder: EntityId, item: EntityId, count: u32) -> Result<(), String> {
        let held = self.holdings.entry(holder).or_default();
        let left = held
            .get(&item)
            .copied()
            .unwrap_or(0)
            .checked_sub(count)
            .ok_or_else(|| {
                format!(
                    "#{} gave up {count} of item #{} it did not hold",
                    holder.raw(),
                    item.raw()
                )
            })?;
        if left == 0 {
            held.remove(&item);
        } else {
            held.insert(item, left);
        }
        Ok(())
    }

    /// How many items `holder` carries, every kind together.
    pub fn carried(&self, holder: EntityId) -> u32 {
        self.holdings
            .get(&holder)
            .map_or(0, |held| held.values().sum())
    }

    /// The holdings with every empty holder dropped, to compare with stored state.
    pub fn non_empty_holdings(&self) -> BTreeMap<EntityId, BTreeMap<EntityId, u32>> {
        self.holdings
            .iter()
            .filter(|(_, held)| !held.is_empty())
            .map(|(holder, held)| (*holder, held.clone()))
            .collect()
    }
}

// ── CP-4: the market lives ───────────────────────────────────────────────────────────────────────

/// Which 30-day bucket an instant falls in; the run's final instant belongs to the last bucket
/// (`social::per_bucket`'s rule).
fn bucket_of(at: WorldTime, days: i64) -> i64 {
    (at.seconds() / BUCKET).min((days * 86_400 - 1) / BUCKET)
}

/// P-5's conditions over a run of `days` (step-10 §4.6.3), located before counted, each bucket's
/// counts printed with every zero shown (`ARC-23`):
///
/// ```text
/// every bucket   ≥ 1 purchase (a money-transferred caused by an action); every job holder — exactly
///                two, read from `hired` — paid ≥ 1 wage (a money-transferred to them caused by a
///                wage-due naming them); ≥ 1 items-produced; ≥ 1 items-consumed; every seat gave ≥ 1
///                (an items-transferred from that seat caused by an action)
/// the run        zero wage-unpaid; no wallet ever below the cheapest price any shop opened with;
///                no person ever carrying more than six
/// ```
///
/// Returns the replayed ledger at the end and the number of buckets read.
pub fn market_lives(town: &Town, facts: &[EventEnvelope], days: i64) -> (Ledger, usize) {
    let buckets = usize::try_from((days * 86_400 + BUCKET - 1) / BUCKET).expect("positive");
    let types: BTreeMap<u64, (&str, Option<MarketFact>)> = facts
        .iter()
        .map(|fact| {
            (
                fact.id().raw(),
                (fact.event_type().as_str(), market_fact(fact)),
            )
        })
        .collect();
    let holders: BTreeSet<EntityId> = types
        .values()
        .filter_map(|(_, fact)| match fact {
            Some(MarketFact::Hired { employee, .. }) => Some(employee.entity_id()),
            _ => None,
        })
        .collect();
    assert_eq!(
        holders, town.job_holders,
        "the job holders read from `hired` are the people whose file carries `job:`"
    );
    // The cheapest price any shop opened with: the least genesis `listed` price of a shop's operator
    // (a shop's prices are its operator's asks since S31-a, `ARC-112`).
    let operators: BTreeSet<EntityId> = types
        .values()
        .filter_map(|(_, fact)| match fact {
            Some(MarketFact::ShopOpened { operator, .. }) => Some(operator.entity_id()),
            _ => None,
        })
        .collect();
    let cheapest = facts
        .iter()
        .filter(|fact| *fact.caused_by() == Causation::WorldGenesis)
        .filter_map(|fact| match &types[&fact.id().raw()].1 {
            Some(MarketFact::Listed { seller, price, .. }) if operators.contains(seller) => {
                Some(*price)
            }
            _ => None,
        })
        .min()
        .expect("a shop opened at genesis, with a price");

    let mut counted: Vec<BTreeMap<String, u64>> = vec![BTreeMap::new(); buckets];
    let mut ledger = Ledger::default();
    let mut unpaid = Vec::new();
    // The first violation per holder, so one drained wallet does not bury the rest of the report.
    let mut drained: BTreeMap<EntityId, String> = BTreeMap::new();
    let mut overfull: BTreeMap<EntityId, String> = BTreeMap::new();
    for fact in facts {
        let Some(market) = &types[&fact.id().raw()].1 else {
            continue;
        };
        let bucket = usize::try_from(bucket_of(fact.at(), days)).expect("non-negative");
        let mut tally = |name: String| *counted[bucket].entry(name).or_default() += 1;
        match (market, fact.caused_by()) {
            (MarketFact::MoneyTransferred { .. }, Causation::Action(_)) => {
                tally("purchase".to_owned());
            }
            (MarketFact::MoneyTransferred { to, .. }, Causation::Event(cause)) => {
                if let Some((_, Some(MarketFact::WageDue { employee, .. }))) =
                    types.get(&cause.raw())
                    && employee.entity_id() == *to
                {
                    tally(format!("wage paid to {}", town.key(*to)));
                }
            }
            (MarketFact::ItemsProduced { .. }, _) => tally("items-produced".to_owned()),
            (MarketFact::ItemsConsumed { .. }, _) => tally("items-consumed".to_owned()),
            (MarketFact::ItemsTransferred { from, .. }, Causation::Action(_))
                if town.people.contains(from) =>
            {
                tally(format!("gave: {}", town.key(*from)));
            }
            (MarketFact::WageUnpaid { .. }, _) => unpaid.push(fact.id().raw()),
            _ => {}
        }
        ledger
            .apply(market)
            .unwrap_or_else(|error| panic!("#{} replays: {error}", fact.id().raw()));
        if let MarketFact::Funded { holder: payer, .. }
        | MarketFact::MoneyTransferred { from: payer, .. } = market
        {
            let balance = ledger.wallets[payer];
            if balance < cheapest && !drained.contains_key(payer) {
                drained.insert(
                    *payer,
                    format!(
                        "#{}: {}'s wallet fell to {balance}, below the cheapest price {cheapest} \
                         (day {})",
                        fact.id().raw(),
                        town.key(*payer),
                        fact.at().seconds() / 86_400 + 1
                    ),
                );
            }
        }
        for holder in [market_holder(market), market_receiver(market)]
            .into_iter()
            .flatten()
        {
            if town.people.contains(&holder)
                && ledger.carried(holder) > PERSON_CAPACITY
                && !overfull.contains_key(&holder)
            {
                overfull.insert(
                    holder,
                    format!(
                        "#{}: {} carries {} items, more than {PERSON_CAPACITY}",
                        fact.id().raw(),
                        town.key(holder),
                        ledger.carried(holder)
                    ),
                );
            }
        }
    }

    let mut required: Vec<String> = vec![
        "purchase".to_owned(),
        "items-produced".to_owned(),
        "items-consumed".to_owned(),
    ];
    required.extend(
        holders
            .iter()
            .map(|holder| format!("wage paid to {}", town.key(*holder))),
    );
    required.extend(
        town.seat_ids()
            .iter()
            .map(|(seat, _)| format!("gave: {seat}")),
    );
    eprintln!("market facts per 30-day bucket ({days} days; a zero is shown, never omitted):");
    for (bucket, counts) in counted.iter().enumerate() {
        let shown: Vec<String> = required
            .iter()
            .map(|name| format!("{name} {}", counts.get(name).copied().unwrap_or(0)))
            .collect();
        eprintln!(
            "  days {}-{}: {}",
            bucket * 30 + 1,
            (bucket + 1) * 30,
            shown.join(", ")
        );
    }
    // Every failure, reported together: the per-bucket absences first, then the run's conditions.
    let mut failures: Vec<String> = Vec::new();
    for (bucket, counts) in counted.iter().enumerate() {
        for name in &required {
            if counts.get(name).copied().unwrap_or(0) == 0 {
                failures.push(format!(
                    "bucket {bucket} (days {}-{}): no {name}",
                    bucket * 30 + 1,
                    (bucket + 1) * 30
                ));
            }
        }
    }
    if !unpaid.is_empty() {
        failures.push(format!(
            "wage-unpaid facts, an employer that could not pay: {unpaid:?}"
        ));
    }
    failures.extend(drained.into_values());
    failures.extend(overfull.into_values());
    assert!(
        failures.is_empty(),
        "CP-4 fails — the market does not live ({} condition(s)):\n{}",
        failures.len(),
        failures.join("\n")
    );
    sales_are_atomic(facts);
    (ledger, buckets)
}

/// I-a-3, I-a-4 (S31-a, `ARC-112`): every `sold` shares its cause with exactly one
/// `money-transferred` buyer → seller of its price and one `items-transferred` seller → buyer of its
/// item, count 1; and every payment caused by a request that is not a wage's has its `sold`. Returns
/// how many sales the history holds.
pub fn sales_are_atomic(facts: &[EventEnvelope]) -> usize {
    let mut by_cause: BTreeMap<String, Vec<MarketFact>> = BTreeMap::new();
    for fact in facts {
        if let Some(mirrored) = market_fact(fact)
            && matches!(fact.caused_by(), Causation::Action(_))
        {
            by_cause
                .entry(format!("{:?}", fact.caused_by()))
                .or_default()
                .push(mirrored);
        }
    }
    let mut sales = 0;
    for (cause, mirrored) in &by_cause {
        let sold: Vec<&MarketFact> = mirrored
            .iter()
            .filter(|fact| matches!(fact, MarketFact::Sold { .. }))
            .collect();
        let paid = mirrored
            .iter()
            .filter(|fact| matches!(fact, MarketFact::MoneyTransferred { .. }))
            .count();
        if sold.is_empty() {
            assert_eq!(
                paid, 0,
                "{cause}: a payment caused by a request with no `sold`"
            );
            continue;
        }
        assert_eq!(sold.len(), 1, "{cause}: one sale per request");
        let MarketFact::Sold {
            seller,
            buyer,
            item,
            count,
            price,
            ..
        } = *sold[0]
        else {
            unreachable!()
        };
        assert_eq!(count, 1, "{cause}: a sale of one");
        let money: Vec<&MarketFact> = mirrored
            .iter()
            .filter(|fact| matches!(fact, MarketFact::MoneyTransferred { .. }))
            .collect();
        assert_eq!(
            money,
            [&MarketFact::MoneyTransferred {
                from: buyer,
                to: seller,
                amount: price
            }],
            "{cause}: the sale's one payment, of its price"
        );
        let goods: Vec<&MarketFact> = mirrored
            .iter()
            .filter(|fact| matches!(fact, MarketFact::ItemsTransferred { .. }))
            .collect();
        assert_eq!(
            goods,
            [&MarketFact::ItemsTransferred {
                from: seller,
                to: buyer,
                item,
                count: 1
            }],
            "{cause}: the sale's one hand-over"
        );
        sales += 1;
    }
    sales
}

/// Who an item fact adds to, if anyone.
fn market_receiver(fact: &MarketFact) -> Option<EntityId> {
    match fact {
        MarketFact::ItemsTransferred { to, .. } => Some(*to),
        _ => None,
    }
}

fn market_holder(fact: &MarketFact) -> Option<EntityId> {
    match fact {
        MarketFact::Stocked { holder, .. } | MarketFact::ItemsProduced { holder, .. } => {
            Some(*holder)
        }
        _ => None,
    }
}

/// The money every `funded` fact put into the world.
pub fn genesis_money(facts: &[EventEnvelope]) -> u64 {
    facts
        .iter()
        .filter_map(market_fact)
        .map(|fact| match fact {
            MarketFact::Funded { balance, .. } => balance,
            _ => 0,
        })
        .sum()
}

/// The state economy and inventory actually wrote, read from the save's newest snapshot, against the
/// replay of exactly the facts at or before that snapshot's revision: every wallet and every holding
/// equal, and the wallets' total equal to the genesis total (money conserved). The facts alone
/// conserve money by construction; the stored state is what can show a payer never debited (F-63).
/// Returns the snapshot's revision.
pub fn state_matches_replay(town: &Town, save: &Path, facts: &[EventEnvelope]) -> u64 {
    let backend = SqliteBackend::open(save, Durability::ProcessCrash).expect("the save opens");
    let revision = *backend
        .snapshot_revisions()
        .expect("snapshot revisions")
        .last()
        .expect("a save holds a snapshot");
    let head = backend.head().expect("a head");
    let later: BTreeSet<u64> = (revision.raw() + 1..=head.raw())
        .flat_map(|after| {
            backend
                .facts_of(WorldRevision::from_raw(after))
                .expect("a revision's facts")
        })
        .map(|row| row.id)
        .collect();
    let cutoff = later.iter().next().copied().unwrap_or(u64::MAX);
    let mut replayed = Ledger::default();
    for fact in facts.iter().filter(|fact| fact.id().raw() < cutoff) {
        if let Some(market) = market_fact(fact) {
            replayed.apply(&market).expect("replays");
        }
    }

    let snapshot: mineworld_kernel::WorldSnapshot = format::decode_snapshot(
        &backend
            .snapshot_at(revision)
            .expect("the snapshot")
            .expect("present"),
        revision,
    )
    .expect("the snapshot decodes");
    let mut wallets = BTreeMap::new();
    let mut stored = BTreeMap::new();
    for record in &snapshot.components {
        let value = || -> Value {
            serde_json::from_slice(record.payload()).expect("a component payload is JSON")
        };
        match record.component_type().as_str() {
            "wallet" => {
                wallets.insert(record.entity(), wallet_balance(&value()));
            }
            "holdings" => {
                let held = holdings(&value());
                if !held.is_empty() {
                    stored.insert(record.entity(), held);
                }
            }
            _ => {}
        }
    }
    let differ: Vec<String> = wallets
        .keys()
        .chain(replayed.wallets.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|holder| wallets.get(*holder) != replayed.wallets.get(*holder))
        .map(|holder| {
            format!(
                "#{}: stored {:?}, replayed {:?}",
                holder.raw(),
                wallets.get(holder),
                replayed.wallets.get(holder)
            )
        })
        .collect();
    let total: u64 = wallets.values().sum();
    let genesis = genesis_money(facts);
    eprintln!(
        "snapshot at revision {} (head {}): {} wallets, total {total} (genesis {genesis}); {} holders",
        revision.raw(),
        head.raw(),
        wallets.len(),
        stored.len()
    );
    assert!(
        differ.is_empty(),
        "a wallet economy wrote differs from the replay of its facts at revision {}: {}",
        revision.raw(),
        differ.join("; ")
    );
    assert_eq!(
        total, genesis,
        "money is not conserved: the wallets economy wrote total {total}, genesis funded {genesis}"
    );
    assert_eq!(
        stored,
        replayed.non_empty_holdings(),
        "the holdings inventory wrote differ from the replay of its facts at revision {}",
        revision.raw()
    );
    for (holder, held) in stored
        .iter()
        .filter(|(holder, _)| town.people.contains(*holder))
    {
        let carried: u32 = held.values().sum();
        assert!(
            carried <= PERSON_CAPACITY,
            "{} holds {carried} in the stored state, more than {PERSON_CAPACITY}",
            town.key(*holder)
        );
    }
    revision.raw()
}
