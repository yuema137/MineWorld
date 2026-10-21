//! The `job:` section of a person's file, which this pack owns (`ARC-31`).

use std::collections::BTreeMap;
use std::num::NonZeroU32;

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, ItemId, OrganizationId, PersonId, PlaceId, Rejection,
    Visibility,
};
use mineworld_kernel::Emission;
use mineworld_schedule::TimeOfDay;
use serde::Deserialize;

use crate::codec;
use crate::component::{Job, Produces};
use crate::error::EmploymentError;
use crate::event::Hired;
use crate::system::EmploymentSystem;

/// This person's job: who employs them, where, which hours each day, the wage, and what a shift
/// produces for the employer.
///
/// ```yaml
/// job:
///   employer: cafe-company       # an organization key
///   workplace: cafe              # a place key
///   from: "05:30"                # the shift, within one day
///   until: "14:00"
///   wage: 120                    # minor units per hour
///   produces: { coffee: 4 }      # optional: per full shift, for the employer
///   manages: true                # optional: this employee manages the employer's staff
/// ```
///
/// A shift that does not end after it starts is refused as the section is decoded, with the
/// section's line and column.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawJob")]
#[schemars(with = "RawJob", example = serde_json::json!({
    "employer": "corner-shop-co", "workplace": "shop", "from": "08:00", "until": "16:00",
    "wage": 1200, "produces": { "bread": 4 }
}))]
pub struct AuthoredJob {
    employer: EntityKey,
    workplace: EntityKey,
    from: TimeOfDay,
    until: TimeOfDay,
    wage: u64,
    produces: BTreeMap<EntityKey, NonZeroU32>,
    manages: bool,
    title: Option<String>,
}

/// The fields as written, before the shift's order is checked.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RawJob {
    /// Who employs them: an organization key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    employer: EntityKey,
    /// Where they work: a place key.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    workplace: EntityKey,
    /// When the shift begins, every day.
    from: TimeOfDay,
    /// When the shift ends, the same day: after `from`.
    until: TimeOfDay,
    /// The wage, in minor units per hour worked, paid by the employer.
    wage: u64,
    /// What one full shift produces for the employer: an item key, then how many (at least 1).
    #[serde(default)]
    #[schemars(with = "BTreeMap<mineworld_authoring::schema::Key, NonZeroU32>")]
    produces: BTreeMap<EntityKey, NonZeroU32>,
    /// Whether this employee manages the employer's staff: told of their lateness and offered
    /// `penalize` when lateness costs money. At most one per employer (`ARC-287`).
    #[serde(default)]
    manages: bool,
    /// What the job is called, in prose: 1 to 40 bytes, no control character. Optional; where the
    /// world enables `attributes`, it is the holder's `employment.occupation` (S30-f, Q-S30f-3).
    #[serde(default)]
    title: Option<String>,
}

/// The most bytes a job title may have.
pub const TITLE_MAX_BYTES: usize = 40;

/// `title` if it is 1 to [`TITLE_MAX_BYTES`] bytes with no control character.
fn checked_title(title: String) -> Result<String, EmploymentError> {
    let why = if title.is_empty() {
        "it is empty".to_owned()
    } else if title.len() > TITLE_MAX_BYTES {
        format!("it is {} bytes", title.len())
    } else if title.chars().any(char::is_control) {
        "it holds a control character".to_owned()
    } else {
        return Ok(title);
    };
    Err(EmploymentError::TitleRefused { why })
}

impl TryFrom<RawJob> for AuthoredJob {
    type Error = EmploymentError;

    fn try_from(raw: RawJob) -> Result<Self, Self::Error> {
        if raw.until <= raw.from {
            return Err(EmploymentError::ShiftOutOfOrder {
                from: raw.from.to_string(),
                until: raw.until.to_string(),
            });
        }
        Ok(Self {
            employer: raw.employer,
            workplace: raw.workplace,
            from: raw.from,
            until: raw.until,
            wage: raw.wage,
            produces: raw.produces,
            manages: raw.manages,
            title: raw.title.map(checked_title).transpose()?,
        })
    }
}

impl AuthoredSection for EmploymentSystem {
    /// `job: { employer, workplace, from, until, wage, produces }`.
    const SECTION: SectionName = SectionName::from_static("job");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    type Authored = AuthoredJob;

    /// The employer is an Organization, the workplace a Place, every produced key an Item; the loader
    /// refuses any other by name before a world exists.
    fn references(authored: &AuthoredJob) -> Vec<Reference<'_>> {
        let mut references = vec![
            Reference {
                key: &authored.employer,
                entity_type: EntityType::Organization,
            },
            Reference {
                key: &authored.workplace,
                entity_type: EntityType::Place,
            },
        ];
        references.extend(authored.produces.keys().map(|key| Reference {
            key,
            entity_type: EntityType::Item,
        }));
        references
    }

    /// One `hired` about the person whose file it is, every key resolved. Visible to the employee.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredJob,
    ) -> Result<Vec<Emission>, Rejection> {
        let refused = |_| Rejection::PreconditionFailed;
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        let employee = PersonId::new(subject, entity_type).map_err(refused)?;
        let resolve = |key, kind| {
            seeding
                .resolve(key, kind)
                .ok_or(Rejection::PreconditionFailed)
        };
        let employer = OrganizationId::new(
            resolve(&authored.employer, EntityType::Organization)?,
            EntityType::Organization,
        )
        .map_err(refused)?;
        let workplace = PlaceId::new(
            resolve(&authored.workplace, EntityType::Place)?,
            EntityType::Place,
        )
        .map_err(refused)?;
        let mut produces = authored
            .produces
            .iter()
            .map(|(key, per_shift)| {
                let item = ItemId::new(resolve(key, EntityType::Item)?, EntityType::Item)
                    .map_err(refused)?;
                Ok(Produces::new(item, per_shift.get()))
            })
            .collect::<Result<Vec<_>, Rejection>>()?;
        produces.sort_by_key(Produces::item);
        let job = Job::new(
            employer,
            workplace,
            authored.from,
            authored.until,
            authored.wage,
            produces,
        )
        .managing(authored.manages)
        .titled(authored.title.clone());
        Ok(vec![
            Emission::new::<Hired>(
                codec::encode(&Hired::new(employee, job)),
                Visibility::Entities([subject].into_iter().collect()),
            )
            .about(vec![subject])
            .with_participants(vec![subject]),
        ])
    }
}
