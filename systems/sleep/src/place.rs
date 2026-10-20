//! Where one may sleep (SD-Na-3): the world's list decides when it says something; otherwise this
//! pack's tag rule. The one place [`SLEEPING_TAG`] is read.

use mineworld_contracts::{Action, EntityId, PlaceId, Tag};
use mineworld_kernel::WorldRead;
use mineworld_sdk::interactions::{self, Ruling};

use crate::action::Sleep;
use crate::interactions::sleeper;
use crate::system::SleepSystem;

/// What a home is called in this pack's vocabulary: a place carrying this tag is one where people may
/// sleep when the world's list says nothing about it — as `consumption`'s `food` is what is eaten.
pub const SLEEPING_TAG: &str = "residential";

/// Whether `person` may sleep at `place`:
///
/// ```text
/// a rule matches             what the rule says          the author wrote it
/// no rule, default: forbid   no                          the author forbade by default
/// no rule, default unsaid    place tagged residential    nothing said
/// nothing configured         place tagged residential    no configure/sleep.yaml
/// ```
pub(crate) fn allowed(read: &WorldRead<'_>, person: EntityId, place: PlaceId) -> bool {
    match interactions::ruling::<SleepSystem>(read, place, &Sleep::ACTION_TYPE, &sleeper(person)) {
        Ruling::Rule(permitted) => permitted,
        Ruling::Default(false) => false,
        Ruling::Default(true) | Ruling::Unconfigured => residential(read, place),
    }
}

/// Whether `place` carries [`SLEEPING_TAG`].
fn residential(read: &WorldRead<'_>, place: PlaceId) -> bool {
    let tag = Tag::new(SLEEPING_TAG).expect("a legal tag");
    read.entity(place.entity_id())
        .is_some_and(|record| record.tags().contains(&tag))
}
