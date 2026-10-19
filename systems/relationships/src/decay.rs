//! Forgetting, as this pack does it: pure retention curves, named by a world, applied lazily where a
//! value is read (`docs/DECISIONS.md` `ARC-86`; `pr-fg-a-relationship-forgetting.md` §5.1).
//!
//! ```text
//! curve        remaining(excess, elapsed, T)                      the excess over the floor left
//! none         excess                                             the default
//! linear       excess − min(excess, 1 000 × elapsed ÷ T)           the whole range lost in T
//! half-life    ((excess × HALF[i]) >> 16) >> k,                   halves every T; 0 once k ≥ 16
//!              k = elapsed ÷ T, i = (elapsed mod T) × 64 ÷ T
//! power        excess × T ÷ (T + elapsed)                         (1 + t/T)^−1
//! ```
//!
//! Integer arithmetic only, so every platform computes the same value. A user curve is a
//! [`DecayCurve`] registered through [`register_curves`] — this pack's `ARC-62` extension catalog —
//! and selected by its name in `configure/relationships.yaml`. Whatever any curve returns, the host
//! ([`crate::component::RelationshipValues::reading`]) clamps it, so forgetting only ever lowers a
//! value toward its floor.

use std::borrow::Cow;
use std::sync::OnceLock;

use mineworld_contracts::{SimDuration, SystemId};
use mineworld_sdk::interactions::ParameterBound;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::interactions::RelationshipParameters;

/// The longest curve name, in bytes.
const NAME_MAX: usize = 32;

/// A curve's name as a world writes it: lowercase `a-z`, `0-9` and `-`, 1 … 32 bytes. Checked for its
/// form when decoded; whether it names a curve is the section's bound ([`CurveCatalog`]).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurveName(Cow<'static, str>);

impl CurveName {
    /// Nothing fades: the default.
    pub const NONE: Self = Self::from_static("none");
    /// The whole range is lost, at a constant rate, in one time constant.
    pub const LINEAR: Self = Self::from_static("linear");
    /// Halves every time constant.
    pub const HALF_LIFE: Self = Self::from_static("half-life");
    /// `T ÷ (T + t)`.
    pub const POWER: Self = Self::from_static("power");

    /// A name known when the program is written; a malformed one does not compile.
    ///
    /// # Panics
    ///
    /// At compile time in a `const`, when `name` is not of a curve name's form.
    pub const fn from_static(name: &'static str) -> Self {
        assert!(
            well_formed(name),
            "a curve name is a-z, 0-9 and -, 1 … 32 bytes"
        );
        Self(Cow::Borrowed(name))
    }

    /// A name read at run time.
    ///
    /// # Errors
    ///
    /// The refusal, worded, when `name` is not of a curve name's form.
    pub fn new(name: &str) -> Result<Self, String> {
        if well_formed(name) {
            Ok(Self(Cow::Owned(name.to_owned())))
        } else {
            Err(format!(
                "'{name}' is not a curve name: lowercase a-z, 0-9 and '-', 1 … {NAME_MAX} bytes"
            ))
        }
    }

    /// The name's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const fn well_formed(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > NAME_MAX {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-') {
            return false;
        }
        index += 1;
    }
    true
}

impl core::fmt::Display for CurveName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for CurveName {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// A curve name's form, for the generated reference (`ARC-90`); which names exist is the field's bound
/// ([`CurveCatalog`]'s `enum`).
impl schemars::JsonSchema for CurveName {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("CurveName")
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": "^[a-z0-9-]{1,32}$",
            "description": "The name of a curve, such as none, linear, half-life or power: 1 to 32 of \
                            a-z, 0-9 and -.",
        })
    }
}

impl<'de> Deserialize<'de> for CurveName {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        Self::new(&name).map_err(serde::de::Error::custom)
    }
}

/// A pure retention curve (`ARC-86`): how much of a value's excess over its floor is left after
/// `elapsed` with no contact, given the world's time constant. Implemented by a pack and listed on
/// relationships' extension line in `systems/installed`; selected by name in
/// `configure/relationships.yaml`.
///
/// A curve should not increase with `elapsed` (a value that recovers between contacts reads as
/// nonsense), and is called with `elapsed ≥ 0` and `time_constant > 0`. The host clamps whatever it
/// returns to `0 ..= excess`, so a curve can never raise a value or carry it past its floor.
pub trait DecayCurve: Send + Sync {
    /// The name a world selects it by; never a bundled curve's name.
    fn name(&self) -> CurveName;

    /// The excess left, `0 ..= excess`. Pure: reads only its arguments, keeps nothing.
    fn remaining(&self, excess: u32, elapsed: SimDuration, time_constant: SimDuration) -> u32;

    /// The pack that implements it: the world must enable this pack to select the curve.
    fn curve_of(&self) -> SystemId;
}

/// The curves compiled into this pack, in the order a refusal lists them.
const BUNDLED: [CurveName; 4] = [
    CurveName::NONE,
    CurveName::LINEAR,
    CurveName::HALF_LIFE,
    CurveName::POWER,
];

/// The build's user curves, sorted by name, written once per process.
static CATALOG: OnceLock<Vec<Box<dyn DecayCurve>>> = OnceLock::new();

fn names(curves: &[Box<dyn DecayCurve>]) -> Vec<CurveName> {
    curves.iter().map(|curve| curve.name()).collect()
}

/// Registers the build's user curves: once per process, before any world is read (`ARC-62`; the
/// loader calls it through `Capability::register_extensions`). The list is sorted by name; the same
/// names again do nothing, which is what lets every host register.
///
/// # Panics
///
/// When a curve is named like a bundled curve or two curves share a name, naming it; and when a
/// different list was registered before, naming both. One build has one catalog: a second, different
/// one is a defect of the host, found before any world runs (`ARC-39` item 5).
pub fn register_curves(mut curves: Vec<Box<dyn DecayCurve>>) {
    curves.sort_by_key(|curve| curve.name());
    let offered = names(&curves);
    if let Some(bundled) = offered.iter().find(|name| BUNDLED.contains(name)) {
        panic!(
            "register_curves was given a curve named '{bundled}', which is the name of a curve \
             bundled with relationships (DECISIONS.md ARC-86)"
        );
    }
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_curves was given two curves named '{}': a build has one curve per name \
             (DECISIONS.md ARC-62)",
            pair[0]
        );
    }
    let mut pending = Some(curves);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = names(registered);
    assert!(
        current == offered,
        "register_curves was called with {offered:?}, but this process already registered \
         {current:?}: one build has one catalog of curves (DECISIONS.md ARC-62)"
    );
}

/// The registered user curves' names; [`None`] until something registers.
pub fn registered_curves() -> Option<Vec<CurveName>> {
    CATALOG.get().map(|curves| names(curves))
}

/// A curve a name resolves to.
#[derive(Clone, Copy)]
pub(crate) enum Curve {
    None,
    Linear,
    HalfLife,
    Power,
    Catalog(&'static dyn DecayCurve),
}

impl Curve {
    /// The curve `name` names: a bundled one, else a registered one.
    pub(crate) fn named(name: &CurveName) -> Option<Self> {
        match name.as_str() {
            "none" => Some(Self::None),
            "linear" => Some(Self::Linear),
            "half-life" => Some(Self::HalfLife),
            "power" => Some(Self::Power),
            _ => CATALOG.get().and_then(|curves| {
                curves
                    .iter()
                    .find(|curve| curve.name() == *name)
                    .map(|curve| Self::Catalog(curve.as_ref()))
            }),
        }
    }

    /// Whether nothing fades, so nothing need be computed.
    pub(crate) const fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    /// What the curve says is left, unclamped. `elapsed ≥ 0` and `time_constant > 0` by the caller.
    pub(crate) fn remaining(
        self,
        excess: u32,
        elapsed: SimDuration,
        time_constant: SimDuration,
    ) -> u32 {
        let (t, big_t) = (elapsed.seconds(), time_constant.seconds());
        match self {
            Self::None => excess,
            Self::Linear => {
                let lost = i128::from(t) * 1_000 / i128::from(big_t);
                let left = i128::from(excess) - lost.min(i128::from(excess));
                u32::try_from(left).unwrap_or(0)
            }
            Self::HalfLife => half_life(excess, t, big_t),
            Self::Power => {
                let left =
                    i128::from(excess) * i128::from(big_t) / (i128::from(big_t) + i128::from(t));
                u32::try_from(left).unwrap_or(0)
            }
            Self::Catalog(curve) => curve.remaining(excess, elapsed, time_constant),
        }
    }

    /// The pack implementing it, when it is a user curve.
    pub(crate) fn implemented_by(self) -> Option<SystemId> {
        match self {
            Self::Catalog(curve) => Some(curve.curve_of()),
            _ => None,
        }
    }
}

/// `HALF[i] = round(65 536 × 2^(−i/64))`, `i` in `0 ..= 64` (computed to 60 digits; no value is a
/// tie). Exact halving at every whole half-life, at most 1.1 % below the continuous curve between.
pub(crate) const HALF: [u64; 65] = [
    65_536, 64_830, 64_132, 63_441, 62_757, 62_081, 61_413, 60_751, 60_097, 59_449, 58_809, 58_176,
    57_549, 56_929, 56_316, 55_709, 55_109, 54_515, 53_928, 53_347, 52_773, 52_204, 51_642, 51_085,
    50_535, 49_991, 49_452, 48_920, 48_393, 47_871, 47_356, 46_846, 46_341, 45_842, 45_348, 44_859,
    44_376, 43_898, 43_425, 42_958, 42_495, 42_037, 41_584, 41_136, 40_693, 40_255, 39_821, 39_392,
    38_968, 38_548, 38_133, 37_722, 37_316, 36_914, 36_516, 36_123, 35_734, 35_349, 34_968, 34_591,
    34_219, 33_850, 33_486, 33_125, 32_768,
];

fn half_life(excess: u32, t: i64, big_t: i64) -> u32 {
    let whole = t / big_t;
    if whole >= 16 {
        return 0;
    }
    let step = usize::try_from((t % big_t) * 64 / big_t).expect("0 ..= 63");
    let scaled = (u64::from(excess) * HALF[step]) >> 16;
    u32::try_from(scaled >> whole).expect("no larger than excess")
}

/// How one value fades: its curve, its time constant and its floor, as the world's section says for
/// one directed pair.
#[derive(Clone, Copy)]
pub(crate) struct Decay {
    curve: Curve,
    time_constant: SimDuration,
    floor: u32,
}

/// Seconds in a day: the section's time constants are written in days.
const DAY: i64 = 86_400;

impl Decay {
    fn new(name: &CurveName, days: u16, floor: u16) -> Result<Self, CurveName> {
        Ok(Self {
            curve: Curve::named(name).ok_or_else(|| name.clone())?,
            time_constant: SimDuration::from_seconds(i64::from(days.max(1)) * DAY),
            floor: u32::from(floor),
        })
    }

    /// A magnitude after `elapsed`: unchanged at or below the floor; otherwise the floor plus what the
    /// curve leaves of the excess, **clamped by this host to `0 ..= excess`** — so forgetting never
    /// raises a value and never carries it past its floor, whatever the curve returns (`ARC-86`).
    fn fade(self, magnitude: u32, elapsed: SimDuration) -> u32 {
        if self.curve.is_none() || magnitude <= self.floor {
            return magnitude;
        }
        let excess = magnitude - self.floor;
        let left = self
            .curve
            .remaining(excess, elapsed, self.time_constant)
            .min(excess);
        self.floor + left
    }

    /// Familiarity (`0 ..= 1 000`) after `elapsed`.
    pub(crate) fn familiarity(self, value: i32, elapsed: SimDuration) -> i32 {
        let magnitude = u32::try_from(value).unwrap_or(0);
        i32::try_from(self.fade(magnitude, elapsed)).expect("no larger than the value")
    }

    /// Regard (`−1 000 ..= 1 000`) after `elapsed`: its magnitude fades, its sign is kept.
    pub(crate) fn regard(self, value: i32, elapsed: SimDuration) -> i32 {
        let faded = i32::try_from(self.fade(value.unsigned_abs(), elapsed))
            .expect("no larger than the value");
        if value < 0 { -faded } else { faded }
    }
}

/// How both of a directed pair's values fade.
#[derive(Clone, Copy)]
pub(crate) struct DecayRule {
    pub(crate) familiarity: Decay,
    pub(crate) regard: Decay,
}

impl DecayRule {
    /// The rule a resolved parameter block states; `Err` names a curve this build does not know.
    pub(crate) fn of(parameters: &RelationshipParameters) -> Result<Self, CurveName> {
        Ok(Self {
            familiarity: Decay::new(
                &parameters.familiarity_decay,
                parameters.familiarity_decay_days,
                parameters.familiarity_floor,
            )?,
            regard: Decay::new(
                &parameters.regard_decay,
                parameters.regard_decay_days,
                parameters.regard_floor,
            )?,
        })
    }

    /// Whether nothing fades, so a reading equals the stored values.
    pub(crate) const fn is_none(&self) -> bool {
        self.familiarity.curve.is_none() && self.regard.curve.is_none()
    }
}

/// The bound of a curve-name field: a bundled curve, or one registered in the catalog.
#[derive(Debug, Clone, Copy)]
pub struct CurveCatalog;

impl ParameterBound<CurveName> for CurveCatalog {
    fn admits(&self, value: &CurveName) -> bool {
        Curve::named(value).is_some()
    }

    fn describe(&self) -> String {
        format!("one of {}", Self::names().join(", "))
    }

    /// The same names, as the field's `enum` (`ARC-90`).
    fn schema(&self) -> serde_json::Map<String, serde_json::Value> {
        let names = Self::names().into_iter().map(serde_json::Value::String);
        let mut keywords = serde_json::Map::new();
        keywords.insert("enum".to_owned(), serde_json::Value::Array(names.collect()));
        keywords
    }
}

impl CurveCatalog {
    /// Every curve a field may name: the bundled ones, then those registered.
    fn names() -> Vec<String> {
        let mut all: Vec<String> = BUNDLED.iter().map(ToString::to_string).collect();
        all.extend(
            registered_curves()
                .unwrap_or_default()
                .iter()
                .map(ToString::to_string),
        );
        all
    }
}

#[cfg(test)]
mod tests;
