//! The daily pass (design §5.3): at each `day-began`, every source's drafts for its target days, in
//! source order — `calendar`, then `random`, then the world's selected external generators by id (C3)
//! — each admitted or withheld by the host against the almanac's open happenings and those admitted
//! before it. A pure function of the configuration, the day, the calendar's date, the open happenings
//! and, for external generators, the world as read.

use mineworld_calendar::{DAY, DayBegan};
use mineworld_contracts::{Visibility, WorldTime};
use mineworld_kernel::{Emission, WorldRead};

use crate::event::{HappeningAnnounced, HappeningWithheld, HappeningsConfigured};
use crate::generate::{
    CalendarGenerator, DayIndex, DraftAsk, DraftSource, HappeningDraft, HappeningGenerator,
    RandomGenerator,
};
use crate::host::Admission;
use crate::process::AlmanacState;
use crate::system::{about_place, encode};

/// The pass for the day `began` opens, at instant `now`, in a world whose almanac was started at
/// `started` — the first pass is the one on the day it was started (the catch-up).
pub(crate) fn pass(
    world: &WorldRead<'_>,
    now: WorldTime,
    started: WorldTime,
    state: &AlmanacState,
    began: &DayBegan,
) -> Vec<Emission> {
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
    let sources: [(DraftSource, Vec<HappeningDraft>); 2] = [
        (DraftSource::Calendar, calendar.drafts(world, &ask)),
        (DraftSource::Random, random.drafts(world, &ask)),
    ];
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
        for draft in drafts {
            emissions.push(decide(&mut admission, configured, &draft, source.clone()));
        }
    }
    emissions
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
        Err(reason) => Emission::new::<HappeningWithheld>(
            encode(&HappeningWithheld::new(draft, reason, source)),
            Visibility::SystemInternal,
        ),
    }
}
