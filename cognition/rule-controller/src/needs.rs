//! Needs and sleep, as a controller reads them, and `rule`'s needs policy as data (`ARC-286`;
//! `pr-s35-nc-controllers.md` SD-Nc-1 … SD-Nc-6; step-35 §6.2).
//!
//! The world discloses facts only. `needs` tells the holder alone how hungry and tired they are, the
//! rate sleep restores tiredness at, and how much each kind they carry would nourish them; `sleep`
//! tells anyone perceiving a sleeper that they sleep (the alarm to the sleeper only) and tells a person
//! the bounds of `for` where they stand; the server offers `sleep`, `wake`, `eat` and `drink` where the
//! world permits them. This module reads those facts and a [`NeedsPolicy`] — parameters a world writes
//! in `decisions.yaml`, never constants tuned to one town (operator ruling R-LE2-2) — says what to do:
//!
//! ```text
//! asleep     my own `asleep` disclosed      → `wake` when a punctual part's commute is due, else nothing
//! eat        hunger ≥ eat_from               → the held kind that nourishes most (ties: lowest ItemId)
//! bed        `sleep` available here, and tiredness ≥ exhausted_from, or tiredness ≥ tired_from inside
//!            [night.from + offset, night.until), offset = mix(seed, observer) mod (spread + 1) minutes
//!            for = max(shortest, min(longest, minutes to rested, minutes until the commute's leave-by))
//! polite     greet, approach and invite skip a person disclosed asleep
//! ```
//!
//! **Inert when absent (I-Nc-1).** Every band starts from the disclosed `needs` or `asleep` record and
//! returns at once without it; none takes a [`Draw`](mineworld_decision::Draw) at all — the bedtime
//! offset is a mix over `(seed, observer)` — so a world without the packs decides exactly as before.
//!
//! `needs` is read by its slug and JSON, never through the pack's crate: that crate links a market pack
//! (inventory), and this controller may not depend on one (`ARC-35` check 2; F-Nc-3). `sleep` links no
//! market pack, so its types are used directly.

use core::fmt;

use mineworld_contracts::{
    ActionRequest, Affordance, Component, EntityId, ItemId, Observation, PerceivedEntity,
    SimDuration,
};
use mineworld_sleep::{Asleep, Sleep, SleepTerms, Wake};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::act::request;
use crate::commute::{Commute, next_part};
use crate::read::{available, disclosed};

/// The component type `needs` discloses a person's own needs on, by its slug.
pub const NEEDS: &str = "needs";

/// The action types consumption offers for a held kind, by their slugs: both feed a hungry person.
pub const FEEDING: [&str; 2] = ["eat", "drink"];

/// `rest_per_hour` when a `needs` record does not state it: step-35 §5.5's default.
const REST_PER_HOUR: u16 = 125;

/// Minutes in a day.
const DAY_MINUTES: i64 = 1_440;

/// A time of day, in minutes after midnight, written `"HH:MM"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockTime(u16);

impl ClockTime {
    /// The time `hours:minutes`, if it is one.
    pub fn new(hours: u16, minutes: u16) -> Option<Self> {
        (hours < 24 && minutes < 60).then_some(Self(hours * 60 + minutes))
    }

    /// Minutes after midnight.
    pub const fn minutes(self) -> u16 {
        self.0
    }

    /// `"HH:MM"`, or nothing for any other text.
    fn parse(text: &str) -> Option<Self> {
        let (hours, minutes) = text.split_once(':')?;
        if hours.len() != 2 || minutes.len() != 2 {
            return None;
        }
        Self::new(hours.parse().ok()?, minutes.parse().ok()?)
    }
}

impl fmt::Display for ClockTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

impl Serialize for ClockTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ClockTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text)
            .ok_or_else(|| D::Error::custom(format!("'{text}' is not a time of day (\"HH:MM\")")))
    }
}

/// The schema of a time of day.
fn clock_schema(_: &mut SchemaGenerator) -> Schema {
    schemars::json_schema!({
        "type": "string",
        "pattern": "^([01][0-9]|2[0-3]):[0-5][0-9]$",
        "description": "A time of day, \"HH:MM\"."
    })
}

/// The hours a person goes to bed in when tired: from `from` until `until`, which may wrap midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct Night {
    /// When the night begins, `"HH:MM"`.
    #[schemars(schema_with = "clock_schema")]
    pub from: ClockTime,
    /// When it ends, `"HH:MM"`; never equal to `from`.
    #[schemars(schema_with = "clock_schema")]
    pub until: ClockTime,
}

impl Default for Night {
    /// 22:00 – 06:00: ATUS *Sleeping* ≥ 471 ‰ of people from 22:00 to 06:00 (step-35 §6.2).
    fn default() -> Self {
        Self {
            from: ClockTime(22 * 60),
            until: ClockTime(6 * 60),
        }
    }
}

/// The keys of a night block.
const NIGHT_KEYS: &[&str] = &["from", "until"];

impl<'de> Deserialize<'de> for Night {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(NightFields)
    }
}

struct NightFields;

impl<'de> Visitor<'de> for NightFields {
    type Value = Night;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a night block ({})", NIGHT_KEYS.join(", "))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Night, A::Error> {
        let mut night = Night::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "from" => night.from = map.next_value()?,
                "until" => night.until = map.next_value()?,
                _ => return Err(A::Error::unknown_field(&key, NIGHT_KEYS)),
            }
            seen.push(key);
        }
        if night.from == night.until {
            return Err(A::Error::custom(format!(
                "the night is empty: 'from' and 'until' are both {}",
                night.from
            )));
        }
        Ok(night)
    }
}

/// `rule`'s needs policy (SD-Nc-1). Every field may be omitted and keeps its default; the defaults are
/// step-35 §6.2's, fixed before any run (I-S35-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct NeedsPolicy {
    /// Hunger at which a held food is eaten, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub eat_from: u16,
    /// Tiredness at which bed is chosen inside the night, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub tired_from: u16,
    /// Tiredness at which bed is chosen at any hour, 1 … 1 000 and at least `tired_from`.
    #[schemars(range(min = 1, max = 1_000))]
    pub exhausted_from: u16,
    /// The hours a tired person goes to bed in.
    pub night: Night,
    /// Minutes, 0 … 240: each person's bedtime is `night.from` plus a seeded offset up to this.
    #[schemars(range(min = 0, max = 240))]
    pub bedtime_spread_minutes: u16,
    /// Whether to sleep no longer than the departure for a punctual next part allows.
    pub wake_for_commute: bool,
}

impl Default for NeedsPolicy {
    fn default() -> Self {
        Self {
            eat_from: 600,
            tired_from: 600,
            exhausted_from: 900,
            night: Night::default(),
            bedtime_spread_minutes: 60,
            wake_for_commute: true,
        }
    }
}

/// The keys of a needs block.
const KEYS: &[&str] = &[
    "eat_from",
    "tired_from",
    "exhausted_from",
    "night",
    "bedtime_spread_minutes",
    "wake_for_commute",
];

/// A value inside `low ..= high`, or a refusal naming the bound, raised while the decoder stands on
/// the value (`DEP-10`).
fn within<E: DeError>(key: &str, value: i64, low: i64, high: i64) -> Result<u16, E> {
    if value < low || value > high {
        return Err(E::custom(format!("'{key}' is {low} … {high}, not {value}")));
    }
    u16::try_from(value).map_err(E::custom)
}

/// The refusal of a needs block whose exhaustion lies below its tiredness.
fn inverted<E: DeError>(tired_from: u16, exhausted_from: u16) -> E {
    E::custom(format!(
        "'exhausted_from' ({exhausted_from}) is below 'tired_from' ({tired_from})"
    ))
}

impl<'de> Deserialize<'de> for NeedsPolicy {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(PolicyFields)
    }
}

struct PolicyFields;

impl<'de> Visitor<'de> for PolicyFields {
    type Value = NeedsPolicy;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a needs block ({})", KEYS.join(", "))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<NeedsPolicy, A::Error> {
        let mut policy = NeedsPolicy::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "eat_from" => policy.eat_from = within(&key, map.next_value()?, 1, 1_000)?,
                "tired_from" => {
                    policy.tired_from = within(&key, map.next_value()?, 1, 1_000)?;
                    // Refused here, at the value, when the other bound was written before it.
                    if seen.iter().any(|written| written == "exhausted_from")
                        && policy.exhausted_from < policy.tired_from
                    {
                        return Err(inverted(policy.tired_from, policy.exhausted_from));
                    }
                }
                "exhausted_from" => {
                    policy.exhausted_from = within(&key, map.next_value()?, 1, 1_000)?;
                    if seen.iter().any(|written| written == "tired_from")
                        && policy.exhausted_from < policy.tired_from
                    {
                        return Err(inverted(policy.tired_from, policy.exhausted_from));
                    }
                }
                "night" => policy.night = map.next_value()?,
                "bedtime_spread_minutes" => {
                    policy.bedtime_spread_minutes = within(&key, map.next_value()?, 0, 240)?;
                }
                "wake_for_commute" => policy.wake_for_commute = map.next_value()?,
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        // One bound written against the other's default.
        if policy.exhausted_from < policy.tired_from {
            return Err(inverted(policy.tired_from, policy.exhausted_from));
        }
        Ok(policy)
    }
}

/// How much one held kind would nourish its holder, as `needs` discloses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Nourishing {
    /// The kind.
    pub item: ItemId,
    /// The hunger one of it takes away.
    pub hunger: u16,
}

/// The rates in force for the holder that a controller reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct FeltRates {
    /// Tiredness restored per hour asleep.
    #[serde(default = "rest_per_hour")]
    pub rest_per_hour: u16,
}

impl Default for FeltRates {
    fn default() -> Self {
        Self {
            rest_per_hour: REST_PER_HOUR,
        }
    }
}

const fn rest_per_hour() -> u16 {
    REST_PER_HOUR
}

/// The part of my own disclosed `needs` record a controller reads (`pr-s35-nb-needs.md` SD-Nb-6).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Felt {
    /// Hunger now, 0 … 1 000.
    pub hunger: u16,
    /// Tiredness now, 0 … 1 000.
    pub tiredness: u16,
    /// The rates in force for me.
    #[serde(default)]
    pub rates: FeltRates,
    /// Every kind I carry that nourishes, in `ItemId` order.
    #[serde(default)]
    pub nourishment: Vec<Nourishing>,
}

impl Felt {
    /// How much one of `item` would nourish me: 0 when `needs` names no nourishment for it.
    pub fn nourishment_of(&self, item: ItemId) -> u16 {
        self.nourishment
            .iter()
            .find(|entry| entry.item == item)
            .map_or(0, |entry| entry.hunger)
    }
}

/// My own needs, if the observation discloses them.
pub fn own(observation: &Observation<Value>) -> Option<Felt> {
    let me = observation.entity(observation.observer())?;
    let record = me
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == NEEDS)?;
    serde_json::from_value(record.payload().clone()).ok()
}

/// Whether the observation discloses this entity asleep.
pub fn is_asleep(entity: &PerceivedEntity<Value>) -> bool {
    entity
        .components()
        .iter()
        .any(|record| *record.component_type() == Asleep::COMPONENT_TYPE)
}

/// Whether the observation discloses `person` asleep; nobody is, in a world without `sleep`.
pub fn sleeping(observation: &Observation<Value>, person: EntityId) -> bool {
    observation.entity(person).is_some_and(is_asleep)
}

/// Whether I am asleep, as my own observation discloses.
pub fn i_sleep(observation: &Observation<Value>) -> bool {
    sleeping(observation, observation.observer())
}

/// The bounds of `sleep`'s `for` where I stand, as `sleep` discloses them to me.
pub fn terms(observation: &Observation<Value>) -> Option<SleepTerms> {
    disclosed::<SleepTerms>(observation.entity(observation.observer())?)
}

/// How long to sleep, in minutes (SD-Nc-4): no longer than the world allows, than it takes to be
/// rested, or — with `wake_for_commute` and a punctual next part ahead — than until its leave-by
/// instant; never shorter than the world allows (a refusal would be the world's answer otherwise).
/// Nothing when the world states no bounds where I stand.
pub fn sleep_minutes(
    observation: &Observation<Value>,
    felt: &Felt,
    wake_for_commute: bool,
    commute: Option<(Commute, SimDuration)>,
) -> Option<u16> {
    let bounds = terms(observation)?;
    let rest = i64::from(felt.rates.rest_per_hour.max(1));
    let rested = (i64::from(felt.tiredness) * 60 + rest - 1) / rest;
    let mut minutes = i64::from(bounds.longest_minutes).min(rested);
    let now = observation.at().seconds();
    if wake_for_commute
        && let Some((commute, stride)) = commute
        && let Some(next) = next_part(observation).filter(|next| next.punctual)
        && now < next.from.seconds()
    {
        let leave_by = commute.leave_by(observation, &next, stride);
        minutes = minutes.min((leave_by - now).div_euclid(60));
    }
    let minutes = minutes.max(i64::from(bounds.shortest_minutes));
    u16::try_from(minutes).ok()
}

impl NeedsPolicy {
    /// Whether `at` (world seconds) lies inside this person's night: from `night.from` plus their
    /// bedtime offset, until `night.until`. The offset is `mix(seed, observer)` modulo the spread plus
    /// one, in minutes — the same for a person every night, and no draw (I-Nc-1).
    pub fn in_night(&self, seed: u64, observer: EntityId, at: i64) -> bool {
        let spread = u64::from(self.bedtime_spread_minutes) + 1;
        let offset = mineworld_decision::mix(seed ^ BEDTIME_TAG, observer.raw()) % spread;
        let offset = i64::try_from(offset).unwrap_or(0);
        let minute = at.rem_euclid(86_400) / 60;
        let from = i64::from(self.night.from.minutes());
        let length = (i64::from(self.night.until.minutes()) - from).rem_euclid(DAY_MINUTES);
        let into = (minute - from).rem_euclid(DAY_MINUTES);
        offset <= into && into < length
    }

    /// Rung 0 (SD-Nc-3): asleep, `wake` when a punctual part's commute is due by `commute` (LE's due
    /// test, unchanged) and the world offers `wake`; otherwise nothing at all — a sleeper answers no
    /// line and takes no initiative. [`None`] when I am not asleep, so the consult goes on.
    pub fn asleep(
        &self,
        observation: &Observation<Value>,
        commute: Option<(Commute, SimDuration)>,
        pace: SimDuration,
    ) -> Option<Option<ActionRequest>> {
        if !i_sleep(observation) {
            return None;
        }
        let woken = self.wake_for_commute
            && commute
                .is_some_and(|(commute, stride)| commute.due(observation, pace, stride).is_some())
            && available::<Wake>(observation, None);
        Some(woken.then(|| request(observation.observer(), &Wake {})))
    }

    /// The eat band (SD-Nc-5): hungry from `eat_from`, the held kind that nourishes most.
    pub fn eat(&self, observation: &Observation<Value>) -> Option<ActionRequest> {
        let felt = own(observation)?;
        if felt.hunger < self.eat_from {
            return None;
        }
        best_meal(observation, &felt)?.request(observation.observer(), encoded)
    }

    /// The bed band (SD-Nc-4): where the world offers `sleep`, exhausted, or tired inside my night —
    /// `sleep` for [`sleep_minutes`].
    pub fn bed(
        &self,
        observation: &Observation<Value>,
        seed: u64,
        commute: Option<(Commute, SimDuration)>,
    ) -> Option<ActionRequest> {
        let felt = own(observation)?;
        if !available::<Sleep>(observation, None) {
            return None;
        }
        let tired = felt.tiredness;
        let now = observation.at().seconds();
        let goes = tired >= self.exhausted_from
            || (tired >= self.tired_from && self.in_night(seed, observation.observer(), now));
        if !goes {
            return None;
        }
        let minutes = sleep_minutes(observation, &felt, self.wake_for_commute, commute)?;
        Some(request(observation.observer(), &Sleep::new(minutes)))
    }
}

/// The item a feeding affordance (`eat`/`drink`) names in its complete payload.
pub fn fed_item(affordance: &Affordance<Value>) -> Option<ItemId> {
    if !FEEDING.contains(&affordance.action_type().as_str()) {
        return None;
    }
    serde_json::from_value(affordance.payload()?.get("item")?.clone()).ok()
}

/// Every available complete `eat`/`drink` offered to me whose kind nourishes me, with its index in the
/// observation's affordances and how much it nourishes, in observation order.
pub fn meals(observation: &Observation<Value>, felt: &Felt) -> Vec<(usize, ItemId, u16)> {
    observation
        .affordances()
        .iter()
        .enumerate()
        .filter(|(_, affordance)| affordance.is_available())
        .filter_map(|(index, affordance)| {
            let item = fed_item(affordance)?;
            let hunger = felt.nourishment_of(item);
            (hunger > 0).then_some((index, item, hunger))
        })
        .collect()
}

/// The meal that nourishes most; ties go to the lowest `ItemId`.
pub fn best_meal<'a>(
    observation: &'a Observation<Value>,
    felt: &Felt,
) -> Option<&'a Affordance<Value>> {
    let (index, _, _) = meals(observation, felt).into_iter().max_by(
        |(_, a, nourishes_a), (_, b, nourishes_b)| {
            nourishes_a.cmp(nourishes_b).then_with(|| b.cmp(a))
        },
    )?;
    observation.affordances().get(index)
}

/// This controller's payload encoding for an offered affordance: JSON bytes.
fn encoded(payload: &Value) -> Vec<u8> {
    serde_json::to_vec(payload).expect("a JSON value is JSON-representable")
}

/// The tag mixed into the bedtime offset: "bed" in ASCII.
const BEDTIME_TAG: u64 = 0x0062_6564;
