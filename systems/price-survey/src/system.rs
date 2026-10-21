//! The installable system: what it declares, how it judges and resolves `survey`, the facts it
//! reduces, and what it offers and discloses. It dispatches only; the decisions live in
//! `judge.rs` and `witness.rs`.

use mineworld_contracts::{
    ActionIntent, ComponentRecord, EntityId, EntityType, Event, EventEnvelope, EventTypeId,
    LifecycleState, Rejection, SystemId,
};
use mineworld_economy::{EconomySystem, Sold};
use mineworld_item::{ItemKind, ItemSystem};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::action::{Survey, survey_requirement};
use crate::codec;
use crate::component::PriceBook;
use crate::event::{PriceNoted, Source, SurveyBookOpened};
use crate::judge::{codes, judge_survey, living_person, refusal, surveyor, tracked};
use crate::witness::witnesses;

/// Which of this pack's facts belong in a person's objective biography (`ARC-29`): none. Noting a
/// price is not an event in a life.
pub const BIOGRAPHICAL: &[EventTypeId] = &[];

/// A surveyor's price book: the asks it surveyed and the sales it perceived (`ARC-116`).
#[derive(Default)]
pub struct PriceSurveySystem;

impl SystemIdentity for PriceSurveySystem {
    const ID: SystemId = SystemId::from_static("price-survey");
}

/// What the build needs to know about this pack beyond [`System`] (`ARC-33`): it owns the
/// `price-survey:` section of a person's file, and none of its facts is biographical.
impl SystemPack for PriceSurveySystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
    mineworld_sdk::owns_section!();
}

/// A fact this pack may not take: a defect in whoever stated it past the rule (`ARC-26`).
fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: PriceSurveySystem::ID,
        event_type,
        reason,
    }
}

impl System for PriceSurveySystem {
    const VERSION: SystemVersion = SystemVersion::new(1);

    /// Depends on economy (whose `sold` it hears and whose disclosure it reads), item (categories)
    /// and presence (the audience rule and where the surveyor stands) — SD-S31c-5.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([EconomySystem::ID, ItemSystem::ID, PresenceSystem::ID])
            .owning::<PriceBook>()
            .providing::<Survey>()
            .emitting::<SurveyBookOpened>()
            .emitting::<PriceNoted>()
            .subscribing_to::<SurveyBookOpened>()
            .subscribing_to::<PriceNoted>()
            .subscribing_to::<Sold>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<PriceBook>()
    }

    /// The payload decodes (`PreconditionFailed` otherwise), there is no target
    /// (`NoSupportedInteraction`), then [`judge_survey`], then the requirement — the pack's own
    /// reasons before the requirement, as the offer reports them (SD-S31a-2).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let _: Survey = codec::read_action(intent)?;
        if intent.target().is_some() {
            return Err(Rejection::NoSupportedInteraction);
        }
        let plan = judge_survey(world, intent.actor())?;
        let here = world
            .component::<Presence>(intent.actor())
            .map(Presence::location)
            .ok_or(Rejection::PreconditionFailed)?;
        survey_requirement(plan.place).evaluate(&here, None, true)
    }

    /// One `price-noted { source: ask }`, judged again on the state `validate` saw.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read = world.read();
        let plan = judge_survey(&read, intent.actor())
            .map_err(|reason| refused(PriceNoted::EVENT_TYPE, reason))?;
        Ok(vec![
            PriceNoted::new(plan.surveyor, plan.place, Source::Ask, plan.notes).emission(),
        ])
    }

    /// The only writes of [`PriceBook`] (I-c-2), and the witness rule:
    ///
    /// - `survey-book-opened` writes an empty book on a living Person that has none, every category
    ///   the category of some declared kind (`survey-category-unknown` otherwise);
    /// - `price-noted` appends its notes, re-checked as the owner (§5.4);
    /// - economy's `sold` is answered with one `price-noted` per surveyor that perceived it.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type().clone();
        if kind == Sold::EVENT_TYPE {
            let sold: Sold = codec::event_payload(event.payload())?;
            return Ok(witnesses(&world.read(), event, &sold));
        }
        if kind == SurveyBookOpened::EVENT_TYPE {
            let opened: SurveyBookOpened = codec::event_payload(event.payload())?;
            let surveyor = opened.surveyor().entity_id();
            let read = world.read();
            if living_person(&read, surveyor).is_none()
                || read.component::<PriceBook>(surveyor).is_some()
            {
                return Err(refused(kind, Rejection::PreconditionFailed));
            }
            if let Some(unknown) = opened
                .categories()
                .iter()
                .find(|category| !declared(&read, category))
            {
                return Err(refused(
                    kind,
                    Rejection::System {
                        code: codes::SURVEY_CATEGORY_UNKNOWN,
                        detail: Some(format!("no declared kind is of category `{unknown}`")),
                    },
                ));
            }
            world.insert(
                surveyor,
                PriceBook::opened(
                    opened.categories().to_vec(),
                    opened.depth(),
                    opened.window_s(),
                    opened.resurvey_after_s(),
                ),
            )?;
        } else if kind == PriceNoted::EVENT_TYPE {
            let noted: PriceNoted = codec::event_payload(event.payload())?;
            let surveyor = noted.surveyor().entity_id();
            let read = world.read();
            let Some(book) = read.component::<PriceBook>(surveyor) else {
                return Err(refused(kind, Rejection::PreconditionFailed));
            };
            let notes = noted.notes();
            let sorted = notes.windows(2).all(|pair| {
                (pair[0].item(), pair[0].seller()) < (pair[1].item(), pair[1].seller())
            });
            let sound = !notes.is_empty()
                && sorted
                && notes
                    .iter()
                    .all(|note| note.price() >= 1 && tracked(&read, book, note.item()))
                && (noted.source() == Source::Ask
                    || (notes.len() == 1 && notes[0].in_stock().is_none()));
            if !sound {
                return Err(refused(kind, Rejection::PreconditionFailed));
            }
            let mut book = book.clone();
            book.note(event.at(), noted.place(), noted.source(), notes);
            world.insert(surveyor, book)?;
        }
        Ok(Vec::new())
    }
}

/// Whether some living Item of the world is a kind of `category`.
fn declared(world: &WorldRead<'_>, category: &mineworld_item::Category) -> bool {
    world.components::<ItemKind>().any(|(item, kind)| {
        kind.category() == category
            && world.entity(item).is_some_and(|record| {
                record.entity_type() == EntityType::Item
                    && record.lifecycle() != LifecycleState::Destroyed
            })
    })
}

impl PerceptionProvider for PriceSurveySystem {
    /// With no target, to a living surveyor where tracked asks are on display: one **complete**
    /// `survey`, refused `surveyed-recently` while the place cools down (Q-S31a-3, Q-S31c-2). Nothing
    /// otherwise.
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let Ok((_, _, place)) = surveyor(world, observer) else {
            return Vec::new();
        };
        let refusal = match judge_survey(world, observer) {
            Ok(_) => None,
            Err(reason) if reason == refusal(codes::SURVEYED_RECENTLY) => Some(reason),
            Err(_) => return Vec::new(),
        };
        let Ok(offer) = Offer::complete(&Survey {}, survey_requirement(place)) else {
            return Vec::new();
        };
        vec![match refusal {
            Some(reason) => offer.refused(reason),
            None => offer,
        }]
    }

    /// The book, as of this read's instant, to its holder only (I-c-5); even when it holds no note.
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        if observer != subject {
            return Vec::new();
        }
        world
            .component::<PriceBook>(subject)
            .map(|book| {
                vec![ComponentRecord::new::<PriceBook>(
                    subject,
                    codec::to_value(&book.visible_at(world.at())),
                )]
            })
            .unwrap_or_default()
    }
}
