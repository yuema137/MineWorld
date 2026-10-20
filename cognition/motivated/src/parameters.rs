//! The `motivated` parameter block: typed, bounded, decoded where the decision list wrote it
//! (`pr-dl-b-motivated.md` §6, SD-DLb-7 … SD-DLb-9).
//!
//! Decoded by hand, not by `parameters!`, because the block holds maps and lists as well as scalars. Each
//! refusal is raised while the decoder stands on the offending value, so the refusal keeps its line and column
//! (`DEP-10`). The derived [`Settings`] are built once, at decode time, and held by the parameter value; a
//! consult reads them and never derives anything (SD-DLb-9).

use core::fmt::{self, Display};
use std::collections::BTreeMap;

use mineworld_contracts::Tag;
use mineworld_decision::ModelParameters;
use mineworld_rule_controller::commute::Commute;
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserializer, Serialize};

use crate::curve::Curve;
use crate::curves::{EDITION, TABLE, Table};
use crate::settings::{AttributeRule, Base, Roles, Settings};

/// The keys of the block, in the order the documentation lists them.
const KEYS: &[&str] = &[
    "curves",
    "top_band",
    "commit_minutes",
    "social_need_hours",
    "reply_floor",
    "base",
    "roles",
    "curve",
    "commute",
    "attributes",
];

/// The most entries `attributes` may list (S30-f §5.7).
const ATTRIBUTES_AT_MOST: usize = 8;

/// One `attributes` entry as written.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    attribute: String,
    option: String,
    by: BTreeMap<String, u32>,
}

/// The options a `base` entry may name (§5.4; `commute`, `ARC-282`; `sleep`, `ARC-286`).
const OPTIONS: [&str; 13] = [
    "agenda", "stay", "greet", "approach", "invite", "join", "leave", "door", "offered", "wander",
    "commute", "penalize", "sleep",
];

/// The roles a `roles` entry may name (§5.4).
const ROLES: [&str; 5] = ["home", "eat", "work", "shop", "outdoor"];

/// The most tags one role may list (SD-DLb-7).
const TAGS_AT_MOST: usize = 8;

/// The curves a world may override. `dry` is a fixed table by condition, not a curve, so it is not here.
const OVERRIDABLE: [&str; 19] = [
    "hungry",
    "sated",
    "tired",
    "wakeful",
    "strictness",
    "departure",
    "unhurried",
    "sociability",
    "loneliness",
    "familiarity",
    "regard",
    "rest",
    "restless",
    "eating",
    "working",
    "shopping",
    "leisure",
    "lateness",
    "activity_age",
];

/// The curves whose input is an hour of the day, 0 … 23.
const HOUR_CURVES: [&str; 7] = [
    "sociability",
    "rest",
    "restless",
    "eating",
    "working",
    "shopping",
    "leisure",
];

/// The parameters of one `motivated` model: what a decision list wrote, typed and bounded.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
// Every key may be omitted: a key the block omits keeps its default (`ModelParameters::decode`).
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct MotivatedParameters {
    /// The named curve set. Only `atus-a3-2025` ships (SD-DLb-8).
    #[schemars(schema_with = "edition_schema")]
    pub curves: String,
    /// Out of 1 000 (per mille), 0 … 500: the band a draw chooses from.
    #[schemars(range(min = 0, max = 500))]
    pub top_band: u32,
    /// Minutes a door is kept, 15 … 1 440.
    #[schemars(range(min = 15, max = 1_440))]
    pub commit_minutes: i64,
    /// Hours without contact at which loneliness reaches its top, 1 … 72.
    #[schemars(range(min = 1, max = 72))]
    pub social_need_hours: i64,
    /// Per mille, 0 … 1 000: the least chance an addressed person replies.
    #[schemars(range(min = 0, max = 1_000))]
    pub reply_floor: u32,
    /// The base weight of each option, per mille.
    pub base: Base,
    /// The tags each role names.
    pub roles: Roles,
    /// Per-world overrides of single curves, by name, as the points written.
    pub curve: BTreeMap<String, Vec<(i64, u32)>>,
    /// When to set out for a punctual part of the day: the same block as `rule`'s (`ARC-282`).
    pub commute: Commute,
    /// Considerations from the observer's own disclosed attributes, at most 8 (S30-f §5.7): each
    /// `{ attribute, option, by: { <value slug>: <per mille> } }`.
    #[schemars(length(max = 8))]
    pub attributes: Vec<AttributeRule>,
    /// The settings the parameters derive, built at decode time and never serialized.
    #[serde(skip)]
    #[schemars(skip)]
    settings: Settings,
}

/// The one shipped curve set, as the only value `curves` accepts.
fn edition_schema(_: &mut SchemaGenerator) -> Schema {
    schemars::json_schema!({ "type": "string", "enum": [EDITION] })
}

impl MotivatedParameters {
    /// The settings a consult is decided with.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

impl Default for MotivatedParameters {
    fn default() -> Self {
        let settings = Settings::defaults();
        Self {
            curves: EDITION.to_owned(),
            top_band: settings.top_band,
            commit_minutes: settings.commit_minutes,
            social_need_hours: settings.social_need_hours,
            reply_floor: settings.reply_floor,
            base: settings.base.clone(),
            roles: settings.roles.clone(),
            curve: BTreeMap::new(),
            commute: settings.commute,
            attributes: Vec::new(),
            settings,
        }
    }
}

impl ModelParameters for MotivatedParameters {
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Fields)
    }
}

/// A value inside `low ..= high`, or a refusal naming the bound.
fn within<T, E>(key: &str, value: T, low: T, high: T) -> Result<T, E>
where
    T: PartialOrd + Display + Copy,
    E: DeError,
{
    if value < low || value > high {
        Err(E::custom(format!("'{key}' is {low} … {high}, not {value}")))
    } else {
        Ok(value)
    }
}

/// The map visitor: one arm per key, a refusal for an unknown or repeated key.
struct Fields;

impl<'de> Visitor<'de> for Fields {
    type Value = MotivatedParameters;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a mapping of motivated parameters (curves, top_band, commit_minutes, social_need_hours, reply_floor, base, roles, curve, commute, attributes)")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<MotivatedParameters, A::Error> {
        let mut parameters = MotivatedParameters::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            match key.as_str() {
                "curves" => {
                    let name: String = map.next_value()?;
                    if name != EDITION {
                        return Err(A::Error::custom(format!(
                            "'{name}' is not a shipped curve set; the shipped set is '{EDITION}'"
                        )));
                    }
                    parameters.curves = name;
                }
                "top_band" => {
                    parameters.top_band = within::<u32, A::Error>(&key, map.next_value()?, 0, 500)?;
                }
                "commit_minutes" => {
                    parameters.commit_minutes =
                        within::<i64, A::Error>(&key, map.next_value()?, 15, 1_440)?;
                }
                "social_need_hours" => {
                    parameters.social_need_hours =
                        within::<i64, A::Error>(&key, map.next_value()?, 1, 72)?;
                }
                "reply_floor" => {
                    parameters.reply_floor =
                        within::<u32, A::Error>(&key, map.next_value()?, 0, 1_000)?;
                }
                "base" => {
                    let entries: BTreeMap<String, u32> = map.next_value()?;
                    for (name, value) in entries {
                        let value = within::<u32, A::Error>(&name, value, 0, 1_000)?;
                        set_base::<A::Error>(&mut parameters.base, &name, value)?;
                    }
                }
                "roles" => {
                    let entries: BTreeMap<String, Vec<String>> = map.next_value()?;
                    for (role, names) in entries {
                        if names.len() > TAGS_AT_MOST {
                            return Err(A::Error::custom(format!(
                                "'{role}' lists {} tags, at most {TAGS_AT_MOST}",
                                names.len()
                            )));
                        }
                        let tags = names
                            .iter()
                            .map(|name| {
                                Tag::new(name.as_str()).map_err(|error| {
                                    A::Error::custom(format!("'{name}' is not a tag: {error}"))
                                })
                            })
                            .collect::<Result<Vec<Tag>, A::Error>>()?;
                        set_role::<A::Error>(&mut parameters.roles, &role, tags)?;
                    }
                }
                "curve" => {
                    let entries: BTreeMap<String, Vec<(i64, u32)>> = map.next_value()?;
                    for (name, points) in entries {
                        if !OVERRIDABLE.contains(&name.as_str()) {
                            return Err(A::Error::custom(format!(
                                "'{name}' is not a curve; the curves are {}",
                                OVERRIDABLE.join(", ")
                            )));
                        }
                        check_domain::<A::Error>(&name, &points)?;
                        Curve::new(points.clone()).map_err(|error| {
                            A::Error::custom(format!("curve '{name}': {error}"))
                        })?;
                        parameters.curve.insert(name, points);
                    }
                }
                "commute" => parameters.commute = map.next_value()?,
                "attributes" => {
                    let rules: Vec<RawRule> = map.next_value()?;
                    parameters.attributes = attribute_rules::<A::Error>(rules)?;
                }
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        finish::<A::Error>(parameters)
    }
}

/// The `attributes` entries, checked: at most [`ATTRIBUTES_AT_MOST`], a qualified id, an option, slugs and
/// per-mille values.
fn attribute_rules<E: DeError>(rules: Vec<RawRule>) -> Result<Vec<AttributeRule>, E> {
    if rules.len() > ATTRIBUTES_AT_MOST {
        return Err(E::custom(format!(
            "'attributes' lists {} entries, at most {ATTRIBUTES_AT_MOST}",
            rules.len()
        )));
    }
    rules
        .into_iter()
        .map(|rule| {
            if !OPTIONS.contains(&rule.option.as_str()) {
                return Err(E::custom(format!(
                    "'{}' is not an option; the options are {}",
                    rule.option,
                    OPTIONS.join(", ")
                )));
            }
            for (slug, value) in &rule.by {
                within::<u32, E>(slug, *value, 0, 1_000)?;
            }
            AttributeRule::new(rule.attribute, rule.option, rule.by).map_err(E::custom)
        })
        .collect()
}

/// The option `name` names in `base`, set to `value`.
fn set_base<E: DeError>(base: &mut Base, name: &str, value: u32) -> Result<(), E> {
    let slot = match name {
        "agenda" => &mut base.agenda,
        "stay" => &mut base.stay,
        "greet" => &mut base.greet,
        "approach" => &mut base.approach,
        "invite" => &mut base.invite,
        "join" => &mut base.join,
        "leave" => &mut base.leave,
        "door" => &mut base.door,
        "offered" => &mut base.offered,
        "wander" => &mut base.wander,
        "commute" => &mut base.commute,
        "penalize" => &mut base.penalize,
        "sleep" => &mut base.sleep,
        _ => {
            return Err(E::custom(format!(
                "'{name}' is not an option; the options are {}",
                OPTIONS.join(", ")
            )));
        }
    };
    *slot = value;
    Ok(())
}

/// The role `name` names in `roles`, set to `tags`.
fn set_role<E: DeError>(roles: &mut Roles, name: &str, tags: Vec<Tag>) -> Result<(), E> {
    let slot = match name {
        "home" => &mut roles.home,
        "eat" => &mut roles.eat,
        "work" => &mut roles.work,
        "shop" => &mut roles.shop,
        "outdoor" => &mut roles.outdoor,
        _ => {
            return Err(E::custom(format!(
                "'{name}' is not a role; the roles are {}",
                ROLES.join(", ")
            )));
        }
    };
    *slot = tags;
    Ok(())
}

/// The input domain of a curve's points: the hours of a day for the hour curves, and the domain of each
/// other curve's own input.
fn check_domain<E: DeError>(name: &str, points: &[(i64, u32)]) -> Result<(), E> {
    let (low, high): (i64, i64) = if HOUR_CURVES.contains(&name) {
        (0, 23)
    } else {
        match name {
            "familiarity" => (0, 1_000),
            "regard" => (-1_000, 1_000),
            "lateness" | "departure" | "unhurried" => (0, 1_000),
            // A level of hunger or tiredness, as `needs` discloses it.
            "hungry" | "sated" | "tired" | "wakeful" => (0, 1_000),
            // A count of latenesses within the window: employment keeps at most twenty.
            "strictness" => (0, 20),
            // Loneliness is hours without contact and activity age is minutes: both non-negative.
            _ => (0, i64::MAX),
        }
    };
    for (index, (x, _)) in points.iter().enumerate() {
        if *x < low || *x > high {
            return Err(E::custom(format!(
                "curve '{name}': point {index} has x {x}, outside the input {low} … {high}"
            )));
        }
    }
    Ok(())
}

/// The derived settings, once every key has been read: the ATUS source and the social need, then every value
/// and override over the defaults (SD-DLb-9).
fn finish<E: DeError>(mut parameters: MotivatedParameters) -> Result<MotivatedParameters, E> {
    let table = Table::parse(TABLE).map_err(E::custom)?;
    let mut settings = Settings::with_table(&table, parameters.social_need_hours);
    settings.top_band = parameters.top_band;
    settings.commit_minutes = parameters.commit_minutes;
    settings.reply_floor = parameters.reply_floor;
    settings.base = parameters.base.clone();
    settings.roles = parameters.roles.clone();
    settings.commute = parameters.commute;
    settings.attributes = parameters.attributes.clone();
    for (name, points) in &parameters.curve {
        let curve = Curve::new(points.clone()).map_err(E::custom)?;
        if settings.curves.atus.replace(name, curve.clone()) {
            continue;
        }
        match name.as_str() {
            "loneliness" => settings.curves.loneliness = curve,
            "familiarity" => settings.curves.familiarity = curve,
            "regard" => settings.curves.regard = curve,
            "lateness" => settings.curves.lateness = curve,
            "activity_age" => settings.curves.activity_age = curve,
            "departure" => settings.curves.departure = curve,
            "unhurried" => settings.curves.unhurried = curve,
            "strictness" => settings.curves.strictness = curve,
            "hungry" => settings.curves.hungry = curve,
            "sated" => settings.curves.sated = curve,
            "tired" => settings.curves.tired = curve,
            "wakeful" => settings.curves.wakeful = curve,
            _ => {}
        }
    }
    parameters.settings = settings;
    Ok(parameters)
}
