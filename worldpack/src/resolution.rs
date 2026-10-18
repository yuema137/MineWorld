//! Read step 10b: owners that resolve across subjects, once, before any seeding (`DECISIONS.md` `ARC-121`
//! item 3).
//!
//! ```text
//! subjects   for each enabled owner that resolves, in `systems:` order: every file of the kinds its
//!            section is carried by — items', organizations', places', people's, each in key order —
//!            with the key, the kind, the tags and the owner's decoded section if the file carries it;
//!            never a name, a note or a location (the type has no field for one)
//! resolve    the owner's own resolution, handed its context and configuration; a refusal, or a seed for
//!            a key that is not one of the subjects, is ResolveRefused naming the system
//! seeding    `crate::load` seeds each subject's seed where the owner's section would have been seeded
//! ```
//!
//! The loader holds a resolution erased ([`ResolvedSeeds`]) and never learns a seed's type.

use std::collections::BTreeMap;
use std::sync::Arc;

use mineworld_authoring::{ContentKind, ErasedSubject, ResolveRefusal, ResolvedSeeds};
use mineworld_contracts::{EntityKey, SystemId, Tags};

use crate::catalog::Capability;
use crate::error::PackError;
use crate::format::{FoundSection, SectionState};
use crate::read::WorldPack;

/// Every resolving owner's resolution, by owner.
pub(crate) type Resolutions = BTreeMap<Capability, Arc<dyn ResolvedSeeds>>;

/// The subjects `owner` resolves over: every file of the `carried_by` kinds, in
/// [`WorldPack::sectioned_files`]' order, each with `owner`'s decoded section if it carries one.
pub(crate) fn subjects<'p>(
    pack: &'p WorldPack,
    owner: Capability,
    carried_by: &[ContentKind],
) -> Vec<ErasedSubject<'p>> {
    let items = pack
        .items()
        .iter()
        .map(|(key, item)| (ContentKind::Item, key, &item.tags, &item.sections));
    let organizations = pack.organizations().iter().map(|(key, organization)| {
        (
            ContentKind::Organization,
            key,
            &organization.tags,
            &organization.sections,
        )
    });
    let places = pack
        .places()
        .iter()
        .map(|(key, place)| (ContentKind::Place, key, &place.tags, &place.sections));
    let people = pack
        .people()
        .iter()
        .map(|(key, person)| (ContentKind::Person, key, &person.tags, &person.sections));
    items
        .chain(organizations)
        .chain(places)
        .chain(people)
        .filter(|(kind, ..)| carried_by.contains(kind))
        .map(|(kind, key, tags, sections)| subject(owner, kind, key, tags, sections))
        .collect()
}

fn subject<'p>(
    owner: Capability,
    kind: ContentKind,
    key: &'p EntityKey,
    tags: &'p Tags,
    sections: &'p [FoundSection],
) -> ErasedSubject<'p> {
    let authored = sections.iter().find_map(|section| match &section.state {
        SectionState::Decoded(content) if section.owner == owner => Some(&**content),
        SectionState::Decoded(_) | SectionState::OwnerNotEnabled | SectionState::NotCarriedHere => {
            None
        }
    });
    ErasedSubject {
        key,
        kind,
        tags,
        authored,
    }
}

/// An owner's refusal to resolve, as the loader reports it.
pub(crate) fn refused(system: SystemId, refusal: ResolveRefusal) -> PackError {
    PackError::ResolveRefused {
        system,
        subject: refusal.subject,
        reason: refusal.reason,
    }
}

/// Read step 10b: every enabled resolving owner's resolution, in `systems:` order. A world with no
/// resolving owner resolves nothing.
pub(crate) fn resolve(pack: &WorldPack) -> Result<Resolutions, PackError> {
    let mut resolutions = Resolutions::new();
    for owner in pack
        .systems()
        .iter()
        .copied()
        .filter(|owner| owner.resolves())
    {
        let carried_by = owner
            .section()
            .map_or(&[][..], |section| section.carried_by);
        let subjects = subjects(pack, owner, carried_by);
        let configuration = pack
            .configuration()
            .iter()
            .find(|found| found.owner == owner)
            .map(|found| &*found.configuration);
        let context = pack.context(owner).map(|context| &**context);
        if let Some(resolved) = owner
            .resolve_subjects(context, configuration, &subjects)
            .map_err(|refusal| refused(owner.id(), refusal))?
        {
            resolutions.insert(owner, resolved);
        }
    }
    Ok(resolutions)
}
