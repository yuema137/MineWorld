//! The `needs:` section, which this pack owns (`ARC-31`, `ARC-284`, `ARC-285`, `ARC-370`): one type with
//! two forms, one for each kind of file that may carry it, each in the unitless or the food-energy model
//! (`pr-s35-nb-needs.md` SD-Nb-2, F-Nb-1; the `body:` precedent).
//!
//! ```text
//! person form   { hunger?, tiredness? }                          unitless   needs-assigned
//!               { food_energy: { initial_kcal }, tiredness? }    food energy
//! item form     { nourishment: { hunger } }                      unitless   nourishment-assigned
//!               { nourishment: { energy_kcal, serving } }        food energy
//! ```
//!
//! Mixing the forms or the models' keys, or an unknown key, is refused by the decoder at the section; a
//! number outside its bound at its own line and column; a form on the other kind of file when seeded,
//! with `needs-section-kind`. Which model a form belongs to is checked against the world's model when
//! genesis is reduced (`src/system.rs`): the world's configuration is not known while seeding. The item
//! form requires `consumption`: nothing else could ever eat the item.

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityType, ItemId, PersonId, Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_kernel::Emission;
use serde::{Deserialize, Deserializer};

use crate::codec;
use crate::component::Nourishment;
use crate::event::{NeedsAssigned, NourishmentAssigned, StartingHunger};
use crate::food_energy::{MAX_KCAL, Serving};
use crate::level::MAX;
use crate::system::NeedsSystem;

/// Why a form was refused on the kind of file it was found in.
pub const SECTION_KIND: RejectionCode = RejectionCode::from_static("needs-section-kind");

/// The system the item form requires: the one that lets a person eat or drink (`ARC-285`).
pub const EATING: SystemId = SystemId::from_static("consumption");

/// A person's starting hunger when the section does not say: a person at midnight (step-35 §5.6).
pub const GENESIS_HUNGER: u16 = 300;

/// A person's starting tiredness when the section does not say: sleepy at midnight (step-35 §5.6).
pub const GENESIS_TIREDNESS: u16 = 700;

/// A deficit or an amount, `0 ..= 1 000`: refused at its own line and column otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct Amount(#[schemars(range(min = 0, max = 1000))] u16);

impl Amount {
    /// The number.
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u32::deserialize(deserializer)?;
        if value > u32::from(MAX) {
            return Err(serde::de::Error::custom(format!(
                "a need's number is 0 … 1 000, not {value}"
            )));
        }
        Ok(Self(u16::try_from(value).unwrap_or(MAX)))
    }
}

/// Whole kcal, `0 ..= 100 000`: refused at its own line and column otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct Kcal(#[schemars(range(min = 0, max = 100_000))] u32);

impl Kcal {
    /// The number.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Kcal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u32::deserialize(deserializer)?;
        if value > MAX_KCAL {
            return Err(serde::de::Error::custom(format!(
                "a kcal number is 0 … 100 000, not {value}"
            )));
        }
        Ok(Self(value))
    }
}

/// A serving, checked where it is written.
#[derive(Debug, Clone, PartialEq, Eq, schemars::JsonSchema)]
#[schemars(transparent)]
pub struct AuthoredServing(Serving);

impl<'de> Deserialize<'de> for AuthoredServing {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let serving = Serving::deserialize(deserializer)?;
        match serving.refusal() {
            Some(refusal) => Err(serde::de::Error::custom(refusal)),
            None => Ok(Self(serving)),
        }
    }
}

/// The person's food-energy form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredFoodEnergy {
    /// The person's reserve at genesis, whole kcal, 0 … their daily use.
    initial_kcal: Kcal,
}

/// The item form's keys, as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WrittenNourishment {
    /// Unitless: how much one of this item lowers hunger when eaten or drunk, 0 … 1 000.
    hunger: Option<Amount>,
    /// Food energy: kcal in one whole serving, 0 … 100 000 (with `serving`).
    energy_kcal: Option<Kcal>,
    /// Food energy: what one inventory unit is (with `energy_kcal`).
    serving: Option<AuthoredServing>,
}

/// In a person file, the person's starting hunger and tiredness; in an item file, `nourishment`, what
/// one does to hunger. Never both forms; never both models' keys.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Written")]
#[schemars(with = "Written", example = serde_json::json!({ "hunger": 200 }))]
pub enum AuthoredNeeds {
    /// A person's starting state.
    Person {
        /// Hunger, in the form written.
        hunger: StartingHunger,
        /// Tiredness, 0 … 1 000.
        tiredness: u16,
    },
    /// What one of an item kind does.
    Item(Nourishment),
}

/// The keys as written, before the form is decided.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Written {
    /// Unitless: a person's starting hunger, 0 (sated) … 1 000 (famished); 300 when not written.
    hunger: Option<Amount>,
    /// Food energy: a person's starting reserve, `{ initial_kcal }`; required in a food-energy world.
    food_energy: Option<AuthoredFoodEnergy>,
    /// A person's starting tiredness, 0 (rested) … 1 000 (exhausted); 700 when not written.
    tiredness: Option<Amount>,
    /// In an item file: what one of this item does to hunger when eaten or drunk.
    nourishment: Option<WrittenNourishment>,
}

impl TryFrom<WrittenNourishment> for Nourishment {
    type Error = String;

    fn try_from(written: WrittenNourishment) -> Result<Self, String> {
        match written {
            WrittenNourishment {
                hunger: Some(hunger),
                energy_kcal: None,
                serving: None,
            } => Ok(Self::Unitless {
                hunger: hunger.get(),
            }),
            WrittenNourishment {
                hunger: None,
                energy_kcal: Some(energy_kcal),
                serving: Some(AuthoredServing(serving)),
            } => Ok(Self::FoodEnergy {
                energy_kcal: energy_kcal.get(),
                serving,
            }),
            WrittenNourishment {
                hunger: Some(_), ..
            } => Err(
                "`nourishment` is unitless (hunger) or food energy (energy_kcal, serving), never both"
                    .to_owned(),
            ),
            WrittenNourishment {
                energy_kcal: Some(_),
                serving: None,
                ..
            } => Err("`nourishment.energy_kcal` needs a `serving`".to_owned()),
            WrittenNourishment {
                serving: Some(_), ..
            } => Err("`nourishment.serving` needs an `energy_kcal`".to_owned()),
            WrittenNourishment { .. } => Err(
                "`nourishment` says `hunger`, or `energy_kcal` with a `serving`".to_owned(),
            ),
        }
    }
}

impl TryFrom<Written> for AuthoredNeeds {
    type Error = String;

    fn try_from(written: Written) -> Result<Self, String> {
        match written {
            Written {
                hunger: None,
                food_energy: None,
                tiredness: None,
                nourishment: Some(nourishment),
            } => Ok(Self::Item(Nourishment::try_from(nourishment)?)),
            Written {
                nourishment: Some(_),
                ..
            } => Err(
                "`needs` is a person's (hunger or food_energy, tiredness) or an item's (nourishment), \
                 never both"
                    .to_owned(),
            ),
            Written {
                hunger: Some(_),
                food_energy: Some(_),
                ..
            } => Err(
                "a person's `needs` has unitless `hunger` or `food_energy`, never both".to_owned(),
            ),
            Written {
                hunger,
                food_energy,
                tiredness,
                ..
            } => Ok(Self::Person {
                hunger: match food_energy {
                    Some(energy) => StartingHunger::FoodEnergy {
                        initial_kcal: energy.initial_kcal.get(),
                    },
                    None => StartingHunger::Unitless {
                        hunger: hunger.map_or(GENESIS_HUNGER, Amount::get),
                    },
                },
                tiredness: tiredness.map_or(GENESIS_TIREDNESS, Amount::get),
            }),
        }
    }
}

/// `needs-section-kind`, naming the form and the kind of file it belongs in.
fn wrong_kind(form: &str, belongs: &str) -> Rejection {
    Rejection::System {
        code: SECTION_KIND,
        detail: Some(format!(
            "the {form} form of `needs` belongs in {belongs} file"
        )),
    }
}

impl AuthoredSection for NeedsSystem {
    /// ```yaml
    /// needs: { hunger: 200 }                                  # in a person file; {} for the defaults
    /// needs: { food_energy: { initial_kcal: 1500 } }          # a person, in a food-energy world
    /// needs: { nourishment: { hunger: 600 } }                 # in an item file
    /// needs: { nourishment: { energy_kcal: 160, serving: { label: "2 slices",
    ///          quantity: { unit: g, amount: 64 } } } }         # an item, in a food-energy world
    /// ```
    const SECTION: SectionName = SectionName::from_static("needs");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person, ContentKind::Item];

    /// The section in one of its forms: decoding it is [`AuthoredNeeds`]'s check.
    type Authored = AuthoredNeeds;

    /// The item form requires `consumption`; the person form, nothing.
    fn requires(authored: &AuthoredNeeds) -> Vec<SystemId> {
        match authored {
            AuthoredNeeds::Item(_) => vec![EATING],
            AuthoredNeeds::Person { .. } => Vec::new(),
        }
    }

    /// The person form on a person: `needs-assigned`, to that person. The item form on an item kind:
    /// `nourishment-assigned`, to nobody. Either on the other kind: `needs-section-kind`.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredNeeds,
    ) -> Result<Vec<Emission>, Rejection> {
        let kind = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        match (authored, kind) {
            (AuthoredNeeds::Person { hunger, tiredness }, EntityType::Person) => {
                let person = PersonId::new(subject, EntityType::Person)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![
                    Emission::new::<NeedsAssigned>(
                        codec::encode(&NeedsAssigned::new(person, *hunger, *tiredness)),
                        Visibility::Participants,
                    )
                    .about(vec![subject])
                    .with_participants(vec![subject]),
                ])
            }
            (AuthoredNeeds::Item(nourishment), EntityType::Item) => {
                let item = ItemId::new(subject, EntityType::Item)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![Emission::new::<NourishmentAssigned>(
                    codec::encode(&NourishmentAssigned::new(item, nourishment.clone())),
                    Visibility::SystemInternal,
                )])
            }
            (AuthoredNeeds::Person { .. }, _) => Err(wrong_kind("person", "a person")),
            (AuthoredNeeds::Item(_), _) => Err(wrong_kind("item (nourishment)", "an item")),
        }
    }
}
