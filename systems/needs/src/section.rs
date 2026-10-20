//! The `needs:` section, which this pack owns (`ARC-31`, `ARC-284`, `ARC-285`): one type with two
//! forms, one for each kind of file that may carry it (`pr-s35-nb-needs.md` SD-Nb-2, F-Nb-1; the
//! `body:` precedent).
//!
//! ```text
//! person form   hunger, tiredness (both optional)   a person's starting levels   needs-assigned
//! item form     nourishment: { hunger }             one item's effect on hunger  nourishment-assigned
//! ```
//!
//! Mixing the forms, or an unknown key, is refused by the decoder at the section; a number outside
//! 0 … 1 000 at its own line and column; a form on the other kind of file when seeded, with
//! `needs-section-kind`. The item form requires `consumption`: nothing else could ever eat the item.

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityType, ItemId, PersonId, Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_kernel::Emission;
use serde::{Deserialize, Deserializer};

use crate::codec;
use crate::event::{Levels, NeedsAssigned, NourishmentAssigned};
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

/// The item form's one field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredNourishment {
    /// How much one of this item lowers hunger when eaten or drunk, 0 … 1 000.
    hunger: Amount,
}

/// In a person file, the person's starting hunger and tiredness (each 0 … 1 000; `{}` for 300 and 700);
/// in an item file, `nourishment: { hunger }`, how much one lowers hunger. Never both forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Written")]
#[schemars(with = "Written", example = serde_json::json!({ "hunger": 200 }))]
pub enum AuthoredNeeds {
    /// A person's starting levels.
    Person {
        /// Hunger, 0 … 1 000.
        hunger: u16,
        /// Tiredness, 0 … 1 000.
        tiredness: u16,
    },
    /// What one of an item kind does to hunger.
    Item {
        /// Hunger one lowers by.
        hunger: u16,
    },
}

/// The keys as written, before the form is decided.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Written {
    /// A person's starting hunger, 0 (sated) … 1 000 (famished); 300 when not written.
    hunger: Option<Amount>,
    /// A person's starting tiredness, 0 (rested) … 1 000 (exhausted); 700 when not written.
    tiredness: Option<Amount>,
    /// In an item file: what one of this item does to hunger when eaten or drunk.
    nourishment: Option<AuthoredNourishment>,
}

impl TryFrom<Written> for AuthoredNeeds {
    type Error = String;

    fn try_from(written: Written) -> Result<Self, String> {
        match written {
            Written {
                hunger: None,
                tiredness: None,
                nourishment: Some(nourishment),
            } => Ok(Self::Item {
                hunger: nourishment.hunger.get(),
            }),
            Written {
                nourishment: Some(_),
                ..
            } => Err(
                "`needs` is a person's (hunger, tiredness) or an item's (nourishment), never both"
                    .to_owned(),
            ),
            Written {
                hunger, tiredness, ..
            } => Ok(Self::Person {
                hunger: hunger.map_or(GENESIS_HUNGER, Amount::get),
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
    /// needs: { hunger: 200 }                    # in a person file; {} for the defaults
    /// needs: { nourishment: { hunger: 600 } }   # in an item file
    /// ```
    const SECTION: SectionName = SectionName::from_static("needs");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person, ContentKind::Item];

    /// The section in one of its two forms: decoding it is [`AuthoredNeeds`]'s check.
    type Authored = AuthoredNeeds;

    /// The item form requires `consumption`; the person form, nothing.
    fn requires(authored: &AuthoredNeeds) -> Vec<SystemId> {
        match authored {
            AuthoredNeeds::Item { .. } => vec![EATING],
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
        match (*authored, kind) {
            (AuthoredNeeds::Person { hunger, tiredness }, EntityType::Person) => {
                let person = PersonId::new(subject, EntityType::Person)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                let levels = Levels { hunger, tiredness };
                Ok(vec![
                    Emission::new::<NeedsAssigned>(
                        codec::encode(&NeedsAssigned::new(person, levels)),
                        Visibility::Participants,
                    )
                    .about(vec![subject])
                    .with_participants(vec![subject]),
                ])
            }
            (AuthoredNeeds::Item { hunger }, EntityType::Item) => {
                let item = ItemId::new(subject, EntityType::Item)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![Emission::new::<NourishmentAssigned>(
                    codec::encode(&NourishmentAssigned::new(item, hunger)),
                    Visibility::SystemInternal,
                )])
            }
            (AuthoredNeeds::Person { .. }, _) => Err(wrong_kind("person", "a person")),
            (AuthoredNeeds::Item { .. }, _) => Err(wrong_kind("item (nourishment)", "an item")),
        }
    }
}
