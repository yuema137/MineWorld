//! The `notices:` section of a person's file, which this pack owns (`ARC-31`): the topics the person
//! subscribes to as the world begins.

use std::collections::BTreeSet;

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PersonId, Rejection};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::configuration::TOPICS_MAX;
use crate::event::{Origin, SubscriptionsSeeded};
use crate::system::NoticesSystem;
use crate::topic::TopicId;

/// A person's subscriptions, as authored.
///
/// ```yaml
/// notices: { subscribe: [prices.ingredients, news.daily] }
/// ```
///
/// When present it replaces the configuration's `defaults.by_tag` for this person, and
/// `subscribe: []` means "none". Each topic must be declared in `configure/notices.yaml`; an undeclared
/// one is refused at genesis as `unknown-topic`, because a section cannot see the configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawNotices")]
#[schemars(with = "RawNotices")]
pub struct AuthoredNotices {
    subscribe: Vec<TopicId>,
}

/// The section as written.
#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = serde_json::json!({ "subscribe": ["prices.ingredients", "news.daily"] }))]
pub struct RawNotices {
    /// The topics this person subscribes to as the world begins: 0 to 32 declared topics, none twice.
    subscribe: Vec<TopicId>,
}

impl TryFrom<RawNotices> for AuthoredNotices {
    type Error = String;

    fn try_from(raw: RawNotices) -> Result<Self, Self::Error> {
        if raw.subscribe.len() > TOPICS_MAX {
            return Err(format!(
                "subscribe lists at most {TOPICS_MAX} topics, not {}",
                raw.subscribe.len()
            ));
        }
        let mut seen = BTreeSet::new();
        for topic in &raw.subscribe {
            if !seen.insert(topic) {
                return Err(format!("subscribe: `{topic}` is listed twice"));
            }
        }
        Ok(Self {
            subscribe: raw.subscribe,
        })
    }
}

impl AuthoredNotices {
    /// The topics, as written.
    pub fn subscribe(&self) -> &[TopicId] {
        &self.subscribe
    }
}

impl AuthoredSection for NoticesSystem {
    /// `notices: { subscribe: [topic] }`.
    const SECTION: SectionName = SectionName::from_static("notices");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    type Authored = AuthoredNotices;

    /// One `subscriptions-seeded { origin: authored }` about the person whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredNotices,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            SubscriptionsSeeded::new(person, authored.subscribe.clone(), Origin::Authored)
                .emission(),
        ])
    }
}
