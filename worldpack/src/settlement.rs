//! Whether a world is a town, and the station-and-landmark contract a town keeps (`DECISIONS.md` `ARC-105`,
//! `MODULE_SPEC.md` §4.3).
//!
//! A World Pack says in `world.yaml` whether it is a town: `settlement: { kind: town, station, landmark }`
//! or `settlement: { kind: none }`. A town names its station hall and its signature landmark, both keys
//! of `places`. The hall heads a **station group**: the Places tagged `station` joined to it by passages
//! between `station`-tagged Places. The group holds a stop (a Place tagged `stop` and `platform`) and a
//! ticket office (a Place tagged `ticket-office`), and every seat can walk to the hall and the landmark.
//!
//! The loader owns this, not a System: it is a fact about how the world is authored, checked from the
//! files' tags and passages alone. It never reads a section (`ARC-31`), so the platform anchor's rule,
//! which needs the platform's floor, is held elsewhere (`CORE_CONCEPTS.md` §6.6).
//!
//! The checks run in this order, and the first failure refuses (read step 9b):
//!
//! ```text
//! 1  station is a key of places                       TOWN_STATION_UNKNOWN
//! 2  landmark is a key of places                      TOWN_LANDMARK_UNKNOWN
//! 3  the hall carries `station` and `public`          TOWN_STATION_UNTAGGED
//! 4  the landmark carries `landmark`                  TOWN_LANDMARK_UNTAGGED
//! 5  the group: from the hall, through station places (with movement); else every station place
//! 6  a place of the group carries `stop` and `platform`   TOWN_STATION_NO_STOP
//! 7  a place of the group carries `ticket-office`         TOWN_STATION_NO_TICKET_OFFICE
//! 8  every station place is in the group (movement)       TOWN_STATION_SCATTERED
//! 9  every located seat reaches the hall (movement)       TOWN_STATION_UNREACHABLE
//! 10 and the landmark, when it is not the hall (movement) TOWN_LANDMARK_UNREACHABLE
//! ```

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;
use std::path::{Path, PathBuf};

use mineworld_contracts::{EntityKey, Tag};
use serde::Deserialize;
use serde::de::{self, Deserializer, MapAccess, Visitor};

use crate::catalog::{Capability, PASSAGE_OWNER};
use crate::error::{ContentKind, PackError};
use crate::format::{AuthoredPerson, AuthoredPlace};

/// What `world.yaml`'s `settlement:` says this world is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettlementKind {
    /// A town: its station hall and its signature landmark, each a key of `places` (they may be equal).
    Town {
        /// The station hall: the entrance Place of the station group.
        station: EntityKey,
        /// The town's signature building.
        landmark: EntityKey,
    },
    /// Not a settlement: a test yard, a lone cabin, a void arena.
    None,
}

/// `world.yaml`'s `settlement:` key (`ARC-105`): whether this world is a town, and if so its station hall
/// and landmark. Optional to the loader; required by `mineworld validate` and `packs validate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settlement {
    kind: SettlementKind,
}

impl Settlement {
    /// What the world is.
    pub fn kind(&self) -> &SettlementKind {
        &self.kind
    }
}

/// The named refusals of the town contract (`MODULE_SPEC.md` §4.3), each with its upper-snake code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SettlementCode {
    /// `world.yaml` does not say whether the world is a town (the authoring gate only).
    Undeclared,
    /// The key is not one of the two shapes (raised while decoding, inside `PackError::Malformed`).
    Malformed,
    /// `station` is not a key of `places`.
    StationUnknown,
    /// `landmark` is not a key of `places`.
    LandmarkUnknown,
    /// The hall lacks `station` or `public`.
    StationUntagged,
    /// The landmark lacks `landmark`.
    LandmarkUntagged,
    /// No Place of the group is a stop (`stop` and `platform`).
    StationNoStop,
    /// No Place of the group is a ticket office.
    StationNoTicketOffice,
    /// A Place tagged `station` is not in the group.
    StationScattered,
    /// A seat's starting place does not reach the hall.
    StationUnreachable,
    /// A seat's starting place does not reach the landmark.
    LandmarkUnreachable,
}

impl SettlementCode {
    /// Every code, in the order `MODULE_SPEC.md` §4.3 lists them.
    pub const ALL: [Self; 11] = [
        Self::Undeclared,
        Self::Malformed,
        Self::StationUnknown,
        Self::LandmarkUnknown,
        Self::StationUntagged,
        Self::LandmarkUntagged,
        Self::StationNoStop,
        Self::StationNoTicketOffice,
        Self::StationScattered,
        Self::StationUnreachable,
        Self::LandmarkUnreachable,
    ];

    /// The code as an author reads it, such as `TOWN_STATION_NO_STOP`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Undeclared => "SETTLEMENT_UNDECLARED",
            Self::Malformed => "SETTLEMENT_MALFORMED",
            Self::StationUnknown => "TOWN_STATION_UNKNOWN",
            Self::LandmarkUnknown => "TOWN_LANDMARK_UNKNOWN",
            Self::StationUntagged => "TOWN_STATION_UNTAGGED",
            Self::LandmarkUntagged => "TOWN_LANDMARK_UNTAGGED",
            Self::StationNoStop => "TOWN_STATION_NO_STOP",
            Self::StationNoTicketOffice => "TOWN_STATION_NO_TICKET_OFFICE",
            Self::StationScattered => "TOWN_STATION_SCATTERED",
            Self::StationUnreachable => "TOWN_STATION_UNREACHABLE",
            Self::LandmarkUnreachable => "TOWN_LANDMARK_UNREACHABLE",
        }
    }
}

impl fmt::Display for SettlementCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The prefix of every decoding refusal, so the parser's line and column travel with the code.
const MALFORMED: &str = "SETTLEMENT_MALFORMED: ";

impl<'de> Deserialize<'de> for Settlement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer
            .deserialize_map(SettlementVisitor)
            .map_err(coded)
    }
}

struct SettlementVisitor;

impl<'de> Visitor<'de> for SettlementVisitor {
    type Value = Settlement;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{MALFORMED}settlement is a map with `kind`")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Settlement, A::Error> {
        let mut kind: Option<String> = None;
        let mut station: Option<EntityKey> = None;
        let mut landmark: Option<EntityKey> = None;
        while let Some(name) = map.next_key::<String>().map_err(coded)? {
            match name.as_str() {
                "kind" => {
                    if kind.is_some() {
                        return Err(twice(&name));
                    }
                    kind = Some(map.next_value().map_err(coded)?);
                }
                "station" | "landmark" => {
                    let slot = if name == "station" {
                        &mut station
                    } else {
                        &mut landmark
                    };
                    if slot.is_some() {
                        return Err(twice(&name));
                    }
                    let text: String = map.next_value().map_err(coded)?;
                    let key = EntityKey::new(text)
                        .map_err(|error| de::Error::custom(format!("{MALFORMED}{error}")))?;
                    *slot = Some(key);
                }
                other => {
                    return Err(de::Error::custom(format!(
                        "{MALFORMED}unknown key `{other}`; expected `kind`, `station`, `landmark`"
                    )));
                }
            }
        }
        let kind = match kind.as_deref() {
            None => {
                return Err(de::Error::custom(format!(
                    "{MALFORMED}`kind` is required: `town` or `none`"
                )));
            }
            Some("town") => {
                let station = station.ok_or_else(|| requires("station"))?;
                let landmark = landmark.ok_or_else(|| requires("landmark"))?;
                SettlementKind::Town { station, landmark }
            }
            Some("none") => {
                for (name, present) in [
                    ("station", station.is_some()),
                    ("landmark", landmark.is_some()),
                ] {
                    if present {
                        return Err(de::Error::custom(format!(
                            "{MALFORMED}kind `none` names no `{name}`"
                        )));
                    }
                }
                SettlementKind::None
            }
            Some(other) => {
                return Err(de::Error::custom(format!(
                    "{MALFORMED}kind `{other}` is neither `town` nor `none`"
                )));
            }
        };
        Ok(Settlement { kind })
    }
}

/// A refusal the parser raises inside `settlement:` (a duplicate key, a non-string value) carries the
/// code too. The parser may refuse a duplicate key before this visitor sees it — which of the two
/// fires is the parser's business — so the code must not depend on which one does.
fn coded<E: de::Error + fmt::Display>(error: E) -> E {
    let text = error.to_string();
    if text.contains(MALFORMED.trim_end()) {
        error
    } else {
        E::custom(format!("{MALFORMED}{text}"))
    }
}

fn twice<E: de::Error>(name: &str) -> E {
    E::custom(format!("{MALFORMED}`{name}` is stated twice"))
}

fn requires<E: de::Error>(name: &str) -> E {
    E::custom(format!(
        "{MALFORMED}kind `town` requires `{name}`: a key of `places`"
    ))
}

/// The shape `settlement:` is written in, for the generated reference only (`ARC-90`): the decoding is
/// [`Settlement`]'s own, which also refuses the combinations this shape cannot express.
#[derive(schemars::JsonSchema)]
#[allow(dead_code)]
pub(crate) struct SettlementSchema {
    /// `town` or `none`.
    kind: SettlementKindName,
    /// Kind `town`: the station hall, a key of `places`. Kind `none`: absent.
    #[schemars(with = "Option<mineworld_authoring::schema::Key>")]
    station: Option<String>,
    /// Kind `town`: the town's signature landmark, a key of `places`; it may equal `station`. Kind `none`:
    /// absent.
    #[schemars(with = "Option<mineworld_authoring::schema::Key>")]
    landmark: Option<String>,
}

/// Whether a world is a town.
#[derive(schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub(crate) enum SettlementKindName {
    /// A town: it names its station hall and its landmark (`MODULE_SPEC.md` §4.3).
    Town,
    /// Not a settlement.
    None,
}

/// What the town checks read: the pack's places, people and seats, and whether `movement` is enabled.
pub(crate) struct Town<'a> {
    pub(crate) manifest_path: &'a Path,
    pub(crate) place_path: &'a dyn Fn(ContentKind, &EntityKey) -> PathBuf,
    pub(crate) places: &'a BTreeMap<EntityKey, AuthoredPlace>,
    pub(crate) people: &'a BTreeMap<EntityKey, AuthoredPerson>,
    pub(crate) seats: &'a BTreeSet<EntityKey>,
    pub(crate) systems: &'a [Capability],
}

/// Checks a town's contract in `MODULE_SPEC.md` §4.3's order; `kind: none` checks nothing.
pub(crate) fn check(settlement: &Settlement, town: &Town<'_>) -> Result<(), PackError> {
    let SettlementKind::Town { station, landmark } = &settlement.kind else {
        return Ok(());
    };
    let refuse =
        |code, path: PathBuf, key: Option<&EntityKey>, detail: String| PackError::Settlement {
            code,
            path,
            key: key.cloned(),
            detail,
        };
    let known = || {
        town.places
            .keys()
            .map(|key| format!("'{key}'"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let place_file = |key: &EntityKey| (town.place_path)(ContentKind::Place, key);

    // 1, 2: both are places.
    let Some(hall) = town.places.get(station) else {
        return Err(refuse(
            SettlementCode::StationUnknown,
            town.manifest_path.to_path_buf(),
            Some(station),
            format!(
                "the station hall '{station}' is not one of the places ({})",
                known()
            ),
        ));
    };
    let Some(mark) = town.places.get(landmark) else {
        return Err(refuse(
            SettlementCode::LandmarkUnknown,
            town.manifest_path.to_path_buf(),
            Some(landmark),
            format!(
                "the landmark '{landmark}' is not one of the places ({})",
                known()
            ),
        ));
    };

    // 3, 4: the tags.
    let missing: Vec<&str> = ["station", "public"]
        .into_iter()
        .filter(|text| !hall.tags.contains(&tag(text)))
        .collect();
    if !missing.is_empty() {
        return Err(refuse(
            SettlementCode::StationUntagged,
            place_file(station),
            Some(station),
            format!(
                "the station hall '{station}' lacks the tag(s) {}",
                missing.join(", ")
            ),
        ));
    }
    if !mark.tags.contains(&tag("landmark")) {
        return Err(refuse(
            SettlementCode::LandmarkUntagged,
            place_file(landmark),
            Some(landmark),
            format!("the landmark '{landmark}' lacks the tag landmark"),
        ));
    }

    // 5: the group.
    let movement = town.systems.contains(&PASSAGE_OWNER);
    let station_tag = tag("station");
    let tagged = |key: &EntityKey| {
        town.places
            .get(key)
            .is_some_and(|place| place.tags.contains(&station_tag))
    };
    let adjacency = adjacency(town.places);
    let group: BTreeSet<&EntityKey> = if movement {
        reach(station, &adjacency, |key| tagged(key))
    } else {
        town.places.keys().filter(|key| tagged(key)).collect()
    };

    // 6, 7: a stop and a ticket office in the group.
    let carries = |key: &EntityKey, texts: &[&str]| {
        let place = &town.places[key];
        texts.iter().all(|text| place.tags.contains(&tag(text)))
    };
    if !group.iter().any(|key| carries(key, &["stop", "platform"])) {
        return Err(refuse(
            SettlementCode::StationNoStop,
            town.manifest_path.to_path_buf(),
            Some(station),
            format!("no place of the station '{station}' carries both stop and platform"),
        ));
    }
    if !group.iter().any(|key| carries(key, &["ticket-office"])) {
        return Err(refuse(
            SettlementCode::StationNoTicketOffice,
            town.manifest_path.to_path_buf(),
            Some(station),
            format!("no place of the station '{station}' carries ticket-office"),
        ));
    }

    if !movement {
        return Ok(());
    }

    // 8: nothing tagged `station` lies outside the group.
    if let Some(stray) = town
        .places
        .keys()
        .find(|key| tagged(key) && !group.contains(key))
    {
        return Err(refuse(
            SettlementCode::StationScattered,
            place_file(stray),
            Some(stray),
            format!(
                "'{stray}' is tagged station but is not joined to the station hall '{station}' through station places"
            ),
        ));
    }

    // 9, 10: every located seat walks to the hall, and to the landmark when it is elsewhere.
    for seat in town.seats {
        let Some(start) = town
            .people
            .get(seat)
            .and_then(|person| person.location.as_ref())
            .map(|location| &location.place)
        else {
            continue;
        };
        let reached = reach(start, &adjacency, |_| true);
        for (target, code, what) in [
            (
                station,
                SettlementCode::StationUnreachable,
                "the station hall",
            ),
            (
                landmark,
                SettlementCode::LandmarkUnreachable,
                "the landmark",
            ),
        ] {
            if code == SettlementCode::LandmarkUnreachable && landmark == station {
                continue;
            }
            if !reached.contains(target) {
                return Err(refuse(
                    code,
                    (town.place_path)(ContentKind::Person, seat),
                    Some(seat),
                    format!(
                        "seat '{seat}' starts in '{start}', from which no passages lead to {what} '{target}'"
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// A role tag (or `public`), known to be a valid tag.
fn tag(text: &str) -> Tag {
    Tag::new(text).expect("a role tag is a valid tag")
}

/// Every place's neighbours, a passage holding both ways whichever file states it.
fn adjacency(
    places: &BTreeMap<EntityKey, AuthoredPlace>,
) -> BTreeMap<&EntityKey, BTreeSet<&EntityKey>> {
    let mut adjacency: BTreeMap<&EntityKey, BTreeSet<&EntityKey>> =
        places.keys().map(|key| (key, BTreeSet::new())).collect();
    for (key, place) in places {
        for passage in &place.passages {
            if let Some((to, _)) = places.get_key_value(&passage.to) {
                adjacency.entry(key).or_default().insert(to);
                adjacency.entry(to).or_default().insert(key);
            }
        }
    }
    adjacency
}

/// Every place reached breadth-first from `from`, entering only places `admit` accepts (`from` always).
fn reach<'a>(
    from: &'a EntityKey,
    adjacency: &BTreeMap<&'a EntityKey, BTreeSet<&'a EntityKey>>,
    admit: impl Fn(&EntityKey) -> bool,
) -> BTreeSet<&'a EntityKey> {
    let mut seen = BTreeSet::from([from]);
    let mut queue = VecDeque::from([from]);
    while let Some(here) = queue.pop_front() {
        for next in adjacency.get(here).into_iter().flatten() {
            if admit(next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
    }
    seen
}
