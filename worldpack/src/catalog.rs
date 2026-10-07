//! Which System Packs this build provides, and what authored content each one's state comes from.
//!
//! The list itself is not here. It is the build's **installed set**, `systems/installed`
//! (`DECISIONS.md` `ARC-33`): one line per pack, expanded by `mineworld_sdk::installed!` into
//! [`Capability`] and [`AVAILABLE`], which this module re-exports. What the loader asks of each
//! capability — its id, its section, how that section decodes, its biographical facts, how it
//! installs, how it answers perception — each pack says once, in its own `impl SystemPack`. Installing
//! a pack therefore edits nothing in this crate.
//!
//! It is still a closed enum, because in MVP-0 the systems are **linked**: a pack asks for
//! `presence` and gets the `mineworld-presence` crate this binary was built with. `ARC-8` makes system
//! extension a WASM component later, at which point this becomes a registry populated at startup and
//! [`PackError::UnknownSystem`](crate::PackError::UnknownSystem) changes from *this build does not
//! provide it* to *nothing installed it*. The shape of the question does not change, which is why
//! this seam is one type rather than a `match` spread through the loader.
//!
//! # Why the loader knows that a `location` is presence's, and a `passage` movement's
//!
//! A pack's `location:` field exists because some system owns location, and in this build that
//! system is `presence` — so [`located`] is where the pack format's vocabulary meets a System Pack's.
//! [`opened`] is the second such meeting: a place's `passages:` become movement's fact.
//!
//! Those two are fields of the format, mapped here by name. They predate the section seam and stay
//! where they are: moving them would edit presence and movement and change refusals authors already
//! see (`DECISIONS.md` `ARC-31`). They are why this crate depends on those two packs, and on no other.
//!
//! # Sections: content a pack owns, which the loader does not understand
//!
//! Everything newer is a **section** (`ARC-31`): a pack implements
//! [`AuthoredSection`](mineworld_authoring::AuthoredSection) and declares it with
//! `mineworld_sdk::owns_section!`, and the loader asks only [`Capability::section`] and
//! [`Capability::decode_section`]. It decodes a section with the owner's type and seeds it with the
//! owner's function, and never learns what it means. Adding a section-owning pack adds no loader code
//! and no line in this crate.

use mineworld_contracts::{Event, LocalPosition, Location, PersonId, PlaceId};
use mineworld_kernel::{Emission, KernelError, SystemIdentity, WorldRead};
use mineworld_movement::passage;
use mineworld_presence::{Arrived, PresenceSystem, arrival};

pub use mineworld_installed_systems::{AVAILABLE, Capability};
pub use mineworld_sdk::SectionOwner;

/// Which capability owns the state an authored `location` becomes.
///
/// Named here rather than assumed in the loader, so that the refusal an author sees — *you placed
/// somebody without enabling the system that owns placement* — says the name of the system they have
/// to enable.
pub const LOCATION_OWNER: Capability = Capability::Presence;

/// Which capability owns the state an authored `passage` becomes: the places a place opens onto.
pub const PASSAGE_OWNER: Capability = Capability::Movement;

/// The genesis fact an authored passage becomes: `a` opens onto `b`, through a doorway at `a_at` in
/// `a` and `b_at` in `b`. Built by the pack that declared the event type
/// ([`mineworld_movement::passage`]); this function only says which fact a passage is.
pub fn opened(
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
) -> Emission {
    passage(a, a_at, b, b_at)
}

/// The genesis fact an authored location becomes.
///
/// The payload is built by the pack that declared the event type, not here: this function only says
/// *which* fact a location is. See [`mineworld_presence::arrival`]. It is built against the
/// assembled world, so an authored placement passes the same check the owner applies to every other
/// arrival (`DECISIONS.md` `ARC-26`: the owner still decides).
///
/// # Errors
///
/// [`KernelError::FactRefusedByOwner`] when presence refuses the value. `read` has already refused
/// a location naming a place the pack does not declare, so reaching this is the two disagreeing.
pub fn located(
    world: &WorldRead<'_>,
    person: PersonId,
    location: Location,
) -> Result<Emission, KernelError> {
    arrival(world, person, location).map_err(|reason| KernelError::FactRefusedByOwner {
        system: PresenceSystem::ID,
        event_type: Arrived::EVENT_TYPE,
        reason,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::content::{PERSON_FIELDS, PLACE_FIELDS};

    /// Section names are one namespace across every pack this build provides, and none may shadow a
    /// field of the format (`ARC-31`). A structural guard over a closed catalog: in this build a
    /// refusal at read time could never be reached, so the property is held here.
    #[test]
    fn no_two_sections_share_a_name_and_none_shadows_a_field() {
        let sections: Vec<SectionOwner> = AVAILABLE
            .into_iter()
            .filter_map(Capability::section)
            .collect();
        assert!(!sections.is_empty(), "the guard guards something");
        let names: BTreeSet<&str> = sections
            .iter()
            .map(|section| section.name.as_str())
            .collect();
        assert_eq!(
            names.len(),
            sections.len(),
            "two packs claim one section: {names:?}"
        );
        for name in &names {
            assert!(
                !PERSON_FIELDS.contains(name) && !PLACE_FIELDS.contains(name),
                "the section `{name}` shadows a field of the format"
            );
        }
        for section in &sections {
            assert!(
                !section.carried_by.is_empty(),
                "`{}` is carried by no kind of file, so it could never be authored",
                section.name
            );
        }
    }
}
