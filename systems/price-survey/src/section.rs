//! The `price-survey:` section of a person's file, which this pack owns (`ARC-31`): it makes the
//! person a surveyor and says which kinds it tracks and how long its notes are kept.

use std::collections::BTreeSet;

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, EntityType, PersonId, Rejection};
use mineworld_item::Category;
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::event::SurveyBookOpened;
use crate::span::{DAY, HOUR, MINUTE, Span};
use crate::system::PriceSurveySystem;

/// The most categories one surveyor tracks.
pub const CATEGORIES_MAX: usize = 16;
/// The most notes kept per kind.
pub const DEPTH_MAX: u8 = 64;
/// Notes kept per kind when the section says nothing (step-31 §9.3).
pub const DEPTH_DEFAULT: u8 = 32;
/// The shortest and longest window, in seconds: 1 hour … 30 days.
pub const WINDOW_BOUNDS: (u32, u32) = (HOUR, 30 * DAY);
/// The window when the section says nothing: seven days, one weekly cycle of a town's shopping.
pub const WINDOW_DEFAULT: u32 = 7 * DAY;
/// The shortest and longest cooldown per place, in seconds: 1 minute … 1 day.
pub const RESURVEY_BOUNDS: (u32, u32) = (MINUTE, DAY);
/// The cooldown when the section says nothing: one hour (Q-S31c-2).
pub const RESURVEY_DEFAULT: u32 = HOUR;

/// A surveyor, as authored.
///
/// ```yaml
/// price-survey:
///   kinds: { categories: [food, drink] }
///   depth: 32
///   window: "7d"
///   resurvey_after: "1h"
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawSurvey")]
#[schemars(with = "RawSurvey")]
pub struct AuthoredSurvey {
    categories: Vec<Category>,
    depth: u8,
    window: Span,
    resurvey_after: Span,
}

/// Which kinds a surveyor tracks.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredKinds {
    /// The item categories whose kinds it notes: 1 to 16, none twice. Each must be the category of
    /// at least one declared kind, or genesis refuses it (`survey-category-unknown`).
    categories: Vec<Category>,
}

/// The section as written.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({
    "kinds": { "categories": ["food", "drink"] },
    "depth": 32,
    "window": "7d",
    "resurvey_after": "1h"
}))]
pub struct RawSurvey {
    /// Which kinds the surveyor tracks, by item category.
    kinds: AuthoredKinds,
    /// How many notes it keeps per kind, the oldest dropped first: 1 to 64. Default 32.
    depth: Option<u64>,
    /// The oldest note it keeps: "1h" to "30d". Default "7d".
    window: Option<String>,
    /// How soon it may survey the same place again: "1m" to "1d". Default "1h".
    resurvey_after: Option<String>,
}

impl TryFrom<RawSurvey> for AuthoredSurvey {
    type Error = String;

    fn try_from(raw: RawSurvey) -> Result<Self, Self::Error> {
        let categories = raw.kinds.categories;
        if categories.is_empty() {
            return Err("kinds.categories: at least one category".to_owned());
        }
        if categories.len() > CATEGORIES_MAX {
            return Err(format!(
                "kinds.categories: at most {CATEGORIES_MAX}, not {}",
                categories.len()
            ));
        }
        let mut seen = BTreeSet::new();
        for category in &categories {
            if !seen.insert(category.clone()) {
                return Err(format!("kinds.categories: `{category}` is listed twice"));
            }
        }
        let depth = match raw.depth {
            None => DEPTH_DEFAULT,
            Some(depth) => u8::try_from(depth)
                .ok()
                .filter(|depth| (1..=DEPTH_MAX).contains(depth))
                .ok_or_else(|| format!("depth: 1 to {DEPTH_MAX}, not {depth}"))?,
        };
        let span = |text: Option<String>, (min, max): (u32, u32), default: u32, key: &str| {
            text.map_or(Ok(Span::of(default)), |text| {
                Span::parse(&text, min, max, key)
            })
        };
        Ok(Self {
            categories: seen.into_iter().collect(),
            depth,
            window: span(raw.window, WINDOW_BOUNDS, WINDOW_DEFAULT, "window")?,
            resurvey_after: span(
                raw.resurvey_after,
                RESURVEY_BOUNDS,
                RESURVEY_DEFAULT,
                "resurvey_after",
            )?,
        })
    }
}

impl AuthoredSurvey {
    /// The tracked categories, sorted.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// Notes kept per kind.
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// The oldest note kept, in seconds.
    pub const fn window(&self) -> u32 {
        self.window.seconds()
    }

    /// The cooldown per place, in seconds.
    pub const fn resurvey_after(&self) -> u32 {
        self.resurvey_after.seconds()
    }
}

impl AuthoredSection for PriceSurveySystem {
    /// `price-survey: { kinds: { categories }, depth?, window?, resurvey_after? }`.
    const SECTION: SectionName = SectionName::from_static("price-survey");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    type Authored = AuthoredSurvey;

    /// One `survey-book-opened` about the person whose file it is, `SystemInternal` (SD-S31c-7).
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredSurvey,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        if entity_type != EntityType::Person {
            return Err(Rejection::PreconditionFailed);
        }
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            SurveyBookOpened::new(
                person,
                authored.categories.clone(),
                authored.depth,
                authored.window(),
                authored.resurvey_after(),
            )
            .emission(),
        ])
    }
}
