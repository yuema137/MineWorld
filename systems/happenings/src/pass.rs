//! The daily pass (design §5.3): at each `day-began`, every source's drafts for its target days, in
//! source order — `calendar`, then `random`, then the world's selected external generators by id
//! (§5.5) — each admitted or withheld by the host against the almanac's open happenings and those
//! admitted before it. A pure function of the configuration, the day, the calendar's date, the open
//! happenings and, for external generators, the world as read.

use mineworld_calendar::{DAY, DayBegan};
use mineworld_contracts::{SystemId, Visibility, WorldTime};
use mineworld_kernel::{Emission, WorldRead};

use crate::catalog;
use crate::event::{HappeningAnnounced, HappeningWithheld, HappeningsConfigured};
use crate::generate::{
    CalendarGenerator, DayIndex, DraftAsk, DraftSource, HappeningDraft, HappeningGenerator,
    RandomGenerator,
};
use crate::host::{Admission, MAX_DRAFTS_PER_GENERATOR, WithheldReason};
use crate::process::AlmanacState;
use crate::system::{about_place, encode};

/// The pass for the day `began` opens, at instant `now`, in a world whose almanac was started at
/// `started` — the first pass is the one on the day it was started (the catch-up).
///
/// # Errors
///
/// The id of a selected generator this process never registered: a world resumed by a host whose build
/// lacks a generator it selected would otherwise silently draft less than it did (fail closed).
pub(crate) fn pass(
    world: &WorldRead<'_>,
    now: WorldTime,
    started: WorldTime,
    state: &AlmanacState,
    began: &DayBegan,
) -> Result<Vec<Emission>, SystemId> {
    let configured = state.configured();
    let day = DayIndex::new(began.day().day_start().seconds().div_euclid(DAY));
    let first_pass = started.seconds().div_euclid(DAY) == day.get();
    let ask = DraftAsk::new(
        day,
        began.day().date(),
        first_pass,
        configured.kinds(),
        configured.seed(),
    );
    let calendar = CalendarGenerator(configured.calendar());
    let random = RandomGenerator {
        seed: configured.seed(),
        entries: configured.random(),
    };
    let mut sources: Vec<(DraftSource, Vec<HappeningDraft>)> = vec![
        (DraftSource::Calendar, calendar.drafts(world, &ask)),
        (DraftSource::Random, random.drafts(world, &ask)),
    ];
    for generator in catalog::selected(configured.generators())? {
        let id = generator.generator_of();
        let drafts = generator.drafts(world, &ask.for_generator(&id));
        sources.push((DraftSource::Generator(id), drafts));
    }
    // A world begun mid-day opens its first day at its first instant, not at the midnight before it:
    // a happening that would already have begun is the past.
    let mut admission = Admission::new(
        day,
        now,
        configured.kinds(),
        configured.limits(),
        state.open().to_vec(),
    );
    let mut emissions = Vec::new();
    for (source, drafts) in sources {
        let capped = matches!(source, DraftSource::Generator(_));
        for (index, draft) in drafts.iter().enumerate() {
            if capped && index == MAX_DRAFTS_PER_GENERATOR {
                // Every draft beyond the cap is withheld together, in one fact naming the first.
                let excess = drafts.len() - MAX_DRAFTS_PER_GENERATOR;
                emissions.push(withheld(
                    draft,
                    WithheldReason::OverGeneratorCap,
                    source.clone(),
                    Some(excess),
                ));
                break;
            }
            emissions.push(decide(&mut admission, configured, draft, source.clone()));
        }
    }
    Ok(emissions)
}

/// `happening-announced` for an admitted draft, or `happening-withheld` with the reason.
fn decide(
    admission: &mut Admission<'_>,
    configured: &HappeningsConfigured,
    draft: &HappeningDraft,
    source: DraftSource,
) -> Emission {
    match admission.admit(draft) {
        Ok(open) => {
            let title = configured
                .kinds()
                .get(draft.kind())
                .and_then(|kind| kind.title().cloned());
            let happening = open.happening().clone();
            about_place::<HappeningAnnounced>(
                &HappeningAnnounced::new(open, title, source),
                &happening,
                Visibility::Public,
            )
        }
        Err(reason) => withheld(draft, reason, source, None),
    }
}

/// `happening-withheld`, SystemInternal, about nobody; `excess` counts the drafts a cap withheld with it.
fn withheld(
    draft: &HappeningDraft,
    reason: WithheldReason,
    source: DraftSource,
    excess: Option<usize>,
) -> Emission {
    Emission::new::<HappeningWithheld>(
        encode(&HappeningWithheld::new(draft, reason, source, excess)),
        Visibility::SystemInternal,
    )
}
