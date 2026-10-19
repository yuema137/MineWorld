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
//!            for = max(shortest, min(longest, L, minutes until the commute's leave-by)), where L is
//!            minutes to the next end of the night (`sleep_until: night-end`), or to rested (`rested`)
//! fetch      hunger ≥ fetch_from and nothing nourishing held → buy an unheld kind offered here, else
//!            walk to a doorway's place tagged with a `shop_tags` tag, else out of a one-door place
//! polite     greet, approach and invite skip a person disclosed asleep
//! ```
//!
//! The defaults of `exhausted_from` (1 000), `sleep_until` and the fetch band are operator ruling
//! Q-S35-OP1 (2026-10-19; `ARC-286`, amendment): bed only in the night unless fully exhausted, a sleep
//! begun in the night runs to its end, and a hungry person with no food goes to a disclosed shop.
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
    ActionRequest, Affordance, Component, EntityId, ItemId, LocalPosition, Location, Millimetres,
    Observation, PerceivedEntity, SimDuration, Tag,
};
use mineworld_movement::{Destination, Passage};
use mineworld_sleep::{Asleep, Sleep, SleepTerms, Wake};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::act::{head_for, request, walk_to};
use crate::commute::{Commute, next_part};
use crate::read::{available, destination_tags, disclosed, distance, doorways};

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

/// How long a person who goes to bed inside the night sleeps for (Q-S35-OP1 (a)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SleepUntil {
    /// Until the night ends (`night.until`), or the commute's leave-by, whichever comes first.
    #[default]
    NightEnd,
    /// Until rested (tiredness ÷ the disclosed rest per hour), as before the ruling.
    Rested,
}

/// The most tags `shop_tags` may list, as a `motivated` role (SD-DLb-7).
const MAX_SHOP_TAGS: usize = 8;

/// The default `shop_tags`: the one generic word for a place that sells.
fn default_shop_tags() -> Vec<Tag> {
    vec![Tag::new("shop").expect("a valid tag")]
}

/// `rule`'s needs policy (SD-Nc-1, amended by Q-S35-OP1). Every field may be omitted and keeps its
/// default; the defaults are step-35 §6.2's as amended on 2026-10-19 (`ARC-286`, amendment).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct NeedsPolicy {
    /// Hunger at which a held food is eaten, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub eat_from: u16,
    /// Tiredness at which bed is chosen inside the night, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub tired_from: u16,
    /// Tiredness at which bed is chosen at any hour, 1 … 1 000 and at least `tired_from`. The default,
    /// 1 000, is the scale's top: outside the night only the fully exhausted go to bed (Q-S35-OP1 (a)).
    #[schemars(range(min = 1, max = 1_000))]
    pub exhausted_from: u16,
    /// The hours a tired person goes to bed in.
    pub night: Night,
    /// Minutes, 0 … 240: each person's bedtime is `night.from` plus a seeded offset up to this.
    #[schemars(range(min = 0, max = 240))]
    pub bedtime_spread_minutes: u16,
    /// Whether to sleep no longer than the departure for a punctual next part allows.
    pub wake_for_commute: bool,
    /// `night-end` (sleep from bedtime to the next end of the night, even when exhaustion sent me to
    /// bed before it began) or `rested`.
    pub sleep_until: SleepUntil,
    /// Whether a hungry person who holds nothing nourishing goes for food (Q-S35-OP1 (b)).
    pub fetch_food: bool,
    /// Hunger at which a person holding nothing nourishing goes for food, 1 … 1 000.
    #[schemars(range(min = 1, max = 1_000))]
    pub fetch_from: u16,
    /// The tags of a place that sells, as the world tags its places; at most 8. A doorway to a place
    /// carrying any of them is where food is fetched from.
    #[schemars(with = "Vec<String>", length(max = 8))]
    pub shop_tags: Vec<Tag>,
    /// Millimetres, 0 … 10 000: at bedtime, standing this near a doorway, walk first to my own spot on
    /// the floor clear of every doorway; 0 sleeps where I stand.
    #[schemars(range(min = 0, max = 10_000))]
    pub bed_clear_of_doors_mm: u16,
}

impl Default for NeedsPolicy {
    fn default() -> Self {
        Self {
            eat_from: 600,
            tired_from: 600,
            exhausted_from: 1_000,
            night: Night::default(),
            bedtime_spread_minutes: 60,
            wake_for_commute: true,
            sleep_until: SleepUntil::NightEnd,
            fetch_food: true,
            fetch_from: 600,
            shop_tags: default_shop_tags(),
            bed_clear_of_doors_mm: 1_500,
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
    "sleep_until",
    "fetch_food",
    "fetch_from",
    "shop_tags",
    "bed_clear_of_doors_mm",
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
                "sleep_until" => policy.sleep_until = map.next_value()?,
                "fetch_food" => policy.fetch_food = map.next_value()?,
                "fetch_from" => policy.fetch_from = within(&key, map.next_value()?, 1, 1_000)?,
                "shop_tags" => {
                    let tags: Vec<Tag> = map.next_value()?;
                    if tags.len() > MAX_SHOP_TAGS {
                        return Err(A::Error::custom(format!(
                            "'shop_tags' lists at most {MAX_SHOP_TAGS} tags, not {}",
                            tags.len()
                        )));
                    }
                    policy.shop_tags = tags;
                }
                "bed_clear_of_doors_mm" => {
                    policy.bed_clear_of_doors_mm = within(&key, map.next_value()?, 0, 10_000)?;
                }
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
    sleep_minutes_until(observation, felt, wake_for_commute, commute, None)
}

/// [`sleep_minutes`], with `until` — minutes to the night's end — in place of "minutes to rested" when
/// given (Q-S35-OP1 (a): a sleep begun in the night runs to its end).
fn sleep_minutes_until(
    observation: &Observation<Value>,
    felt: &Felt,
    wake_for_commute: bool,
    commute: Option<(Commute, SimDuration)>,
    until: Option<i64>,
) -> Option<u16> {
    let bounds = terms(observation)?;
    let rest = i64::from(felt.rates.rest_per_hour.max(1));
    let rested = (i64::from(felt.tiredness) * 60 + rest - 1) / rest;
    let mut minutes = i64::from(bounds.longest_minutes).min(until.unwrap_or(rested));
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

    /// Minutes from `at` (world seconds) to the night's end, `night.until`, the next one after `at`.
    pub fn minutes_to_night_end(&self, at: i64) -> i64 {
        let minute = at.rem_euclid(86_400) / 60;
        (i64::from(self.night.until.minutes()) - minute).rem_euclid(DAY_MINUTES)
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
        if !self.bedtime(observation, &felt, seed) {
            return None;
        }
        let now = observation.at().seconds();
        // Q-S35-OP1 (a): a sleep runs to the end of the night — the one I am in, or for the fully
        // exhausted who go to bed before it, the next — so that nobody wakes rested at 03:00.
        let until =
            (self.sleep_until == SleepUntil::NightEnd).then(|| self.minutes_to_night_end(now));
        let minutes =
            sleep_minutes_until(observation, &felt, self.wake_for_commute, commute, until)?;
        Some(request(observation.observer(), &Sleep::new(minutes)))
    }

    /// Whether the bed band would put me to bed now: `sleep` available here, and exhausted, or tired
    /// inside my night.
    fn bedtime(&self, observation: &Observation<Value>, felt: &Felt, seed: u64) -> bool {
        let tired = felt.tiredness;
        let night = self.in_night(seed, observation.observer(), observation.at().seconds());
        available::<Sleep>(observation, None)
            && (tired >= self.exhausted_from || (tired >= self.tired_from && night))
    }

    /// Before the bed band (D-Nc2-4): at bedtime, standing within `bed_clear_of_doors_mm` of a doorway
    /// of this place, walk first to my own spot on the disclosed floor, clear of every doorway — so a
    /// sleeper does not lie across the way out and stall an early riser's walk or wake at each passer-by.
    /// The spot is the first of eight seeded points of the floor (`mix(seed, observer, k)`, the same every
    /// night, no draw) that is at least the clearance from every doorway; none, or no floor or doorway
    /// disclosed, and I sleep where I stand. `Some(None)` when my walk already goes there.
    pub fn to_bed(
        &self,
        observation: &Observation<Value>,
        seed: u64,
    ) -> Option<Option<ActionRequest>> {
        let clear = i64::from(self.bed_clear_of_doors_mm);
        if clear == 0 {
            return None;
        }
        let felt = own(observation)?;
        if !self.bedtime(observation, &felt, seed) {
            return None;
        }
        let here = *observation.self_location()?;
        let at = here.local()?;
        let doors: Vec<LocalPosition> = doorways(observation, here)?
            .iter()
            .filter_map(Passage::here)
            .collect();
        let near = |point: LocalPosition| doors.iter().any(|door| distance(point, *door) < clear);
        if !near(at) {
            return None;
        }
        let (low, high) = floor(observation, here)?;
        let person = mineworld_decision::mix(seed ^ SPOT_TAG, observation.observer().raw());
        let spot = (0..SPOT_TRIES)
            .filter_map(|k| {
                let roll = mineworld_decision::mix(person, k);
                let x = within_span(low.0, high.0, roll)?;
                let y = within_span(low.1, high.1, roll >> 32)?;
                Some(LocalPosition::new(
                    Millimetres::new(x),
                    Millimetres::new(y),
                    at.z(),
                ))
            })
            .find(|spot| !near(*spot))?;
        Some(walk_to(observation, Destination::Place(here.with_local(spot)))?.request())
    }

    /// The fetch band (Q-S35-OP1 (b)): hungry from `fetch_from` and holding nothing that nourishes me,
    /// go for food from what the world discloses — no place is named:
    ///
    /// ```text
    /// a `buy` offered available here     buy a kind I hold none of, the lowest ItemId (a held kind
    ///                                    that does not nourish me is known not to help)
    /// here sells (a shop tag, or a `buy`
    /// offered at all)                    nothing more: the shop has nothing for me
    /// a doorway to a shop-tagged place   walk there (the lowest PlaceId of them)
    /// a place with one doorway           walk out through it: a dead end leads to the rest of the world
    /// ```
    ///
    /// [`None`] when the band does not apply, so the consult goes on; `Some(None)` when my own walk
    /// already goes where the band would send me, so nothing redirects it. No draw.
    pub fn fetch(&self, observation: &Observation<Value>) -> Option<Option<ActionRequest>> {
        if !self.fetch_food {
            return None;
        }
        let felt = own(observation)?;
        if felt.hunger < self.fetch_from || felt.nourishment.iter().any(|food| food.hunger > 0) {
            return None;
        }
        let held = held_kinds(observation);
        let buys: Vec<(ItemId, &Affordance<Value>)> = observation
            .affordances()
            .iter()
            .filter(|affordance| affordance.action_type().as_str() == BUY)
            .filter_map(|affordance| {
                let item =
                    serde_json::from_value(affordance.payload()?.get("item")?.clone()).ok()?;
                Some((item, affordance))
            })
            .collect();
        if let Some((_, offer)) = buys
            .iter()
            .filter(|(item, offer)| offer.is_available() && !held.contains(item))
            .min_by_key(|(item, _)| *item)
        {
            return Some(offer.request(observation.observer(), encoded));
        }
        let here = *observation.self_location()?;
        let sells_here = !buys.is_empty()
            || observation
                .entity(here.place().entity_id())
                .is_some_and(|place| self.shop_tags.iter().any(|tag| place.tags().contains(tag)));
        if sells_here {
            return None;
        }
        let shops = destination_tags(observation, here);
        let shop = shops
            .iter()
            .find(|(_, tags)| tags.iter().any(|tag| self.shop_tags.contains(tag)))
            .map(|(place, _)| *place);
        let to = match shop {
            Some(place) => place,
            None => {
                let doors = doorways(observation, here)?;
                let mut doors = doors.iter();
                let only = doors.next()?;
                if doors.next().is_some() {
                    return None;
                }
                only.to()
            }
        };
        Some(head_for(observation, to)?.request())
    }
}

/// The action type economy offers for a priced kind, by its slug — named, never linked (`ARC-35`
/// check 2).
const BUY: &str = "buy";

/// The component type inventory discloses a holder's holdings on, by its slug.
const HOLDINGS: &str = "holdings";

/// The part of inventory's disclosed `holdings` record this controller reads: which kinds I hold.
#[derive(Deserialize)]
struct DisclosedHoldings {
    #[serde(default)]
    held: Vec<DisclosedHeld>,
}

#[derive(Deserialize)]
struct DisclosedHeld {
    item: ItemId,
}

/// Every kind I hold, as inventory discloses it to me; none when it discloses nothing.
fn held_kinds(observation: &Observation<Value>) -> Vec<ItemId> {
    observation
        .entity(observation.observer())
        .and_then(|me| {
            me.components()
                .iter()
                .find(|record| record.component_type().as_str() == HOLDINGS)
        })
        .and_then(|record| {
            serde_json::from_value::<DisclosedHoldings>(record.payload().clone()).ok()
        })
        .map(|holdings| holdings.held.into_iter().map(|held| held.item).collect())
        .unwrap_or_default()
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

/// The tag mixed into a person's bed spot: "spot" in ASCII.
const SPOT_TAG: u64 = 0x7370_6f74;

/// How many seeded points of the floor are tried for a bed spot clear of the doorways.
const SPOT_TRIES: u64 = 8;

/// How far inside the floor's edge a bed spot lies, in millimetres: a person's radius (250 mm) and the
/// `bodies` gap, so the spot is a place one may stand (movement still moves a goal off any solid).
const SPOT_INSET: i64 = 400;

/// The component type `bodies` discloses a place's shape on, by its slug; only the floor is read.
const PLACE_SHAPE: &str = "place-shape";

#[derive(Deserialize)]
struct DisclosedShape {
    floor: DisclosedFloor,
}

#[derive(Deserialize)]
struct DisclosedFloor {
    min: DisclosedCorner,
    max: DisclosedCorner,
}

#[derive(Deserialize)]
struct DisclosedCorner {
    x: i32,
    y: i32,
}

/// The floor of the place `here`, as `bodies` discloses it: its south-west and north-east corners.
fn floor(observation: &Observation<Value>, here: Location) -> Option<((i32, i32), (i32, i32))> {
    let record = observation
        .entity(here.place().entity_id())?
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == PLACE_SHAPE)?;
    let shape: DisclosedShape = serde_json::from_value(record.payload().clone()).ok()?;
    Some((
        (shape.floor.min.x, shape.floor.min.y),
        (shape.floor.max.x, shape.floor.max.y),
    ))
}

/// A coordinate inside `[low + SPOT_INSET, high − SPOT_INSET)`, chosen by `roll`; none when the floor is
/// too narrow for the inset.
fn within_span(low: i32, high: i32, roll: u64) -> Option<i32> {
    let span = i64::from(high) - i64::from(low) - 2 * SPOT_INSET;
    let span = u64::try_from(span).ok().filter(|span| *span > 0)?;
    let offset = i64::try_from(roll % span).ok()?;
    i32::try_from(i64::from(low) + SPOT_INSET + offset).ok()
}
