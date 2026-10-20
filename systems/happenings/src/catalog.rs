//! The catalog through which other System Packs contribute happening generators (`DECISIONS.md`
//! `ARC-62`; design §5.5).
//!
//! A generator is code that decides *when* something happens — from the weather, from who lives where,
//! from a fixed storyline — written by a pack author who never edits this pack. It is registered once
//! per process from the build's installed set, by the one assembly path every host uses
//! (`worldpack::compose`, through `Capability::register_extensions`), exactly as movement's wayfinders
//! are. It is the build's compiled-in list of generator code, identical for every world in the process.
//!
//! A world **selects** the generators it uses by id in `configure/happenings.yaml`'s `generators:`; an
//! unselected generator is never asked, so installing a generator pack changes no world that does not
//! select it. A selected id must be registered (refused at seeding otherwise) and its pack enabled in
//! the world (the configuration `requires` it, so the loader refuses otherwise). Every draft a generator
//! returns passes this pack's admission.
//!
//! Exactly three functions touch the catalog: [`register_generators`], [`registered_generators`] and
//! the pass's crate-private [`selected`].

use std::sync::OnceLock;

use mineworld_contracts::SystemId;

use crate::generate::HappeningGenerator;

/// The build's generators, sorted by id: written once per process, by [`register_generators`].
static CATALOG: OnceLock<Vec<Box<dyn HappeningGenerator>>> = OnceLock::new();

/// The ids of a list of generators, in its order.
fn ids(generators: &[Box<dyn HappeningGenerator>]) -> Vec<SystemId> {
    generators
        .iter()
        .map(|generator| generator.generator_of())
        .collect()
}

/// Registers the build's generators: once per process, before any world is assembled.
///
/// The list is sorted by [`HappeningGenerator::generator_of`], so the order generators are asked in is
/// a function of their names. Registering the same ids again does nothing, which is what lets every
/// composition register.
///
/// # Panics
///
/// When the list names one id twice, naming it; and when a different list was registered before,
/// naming both (`ARC-62` item 4).
pub fn register_generators(mut generators: Vec<Box<dyn HappeningGenerator>>) {
    generators.sort_by_key(|generator| generator.generator_of());
    let offered = ids(&generators);
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_generators was given two happening generators for '{}': a build has one \
             generator per system (DECISIONS.md ARC-62)",
            pair[0]
        );
    }
    let mut pending = Some(generators);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = ids(registered);
    assert!(
        current == offered,
        "register_generators was called with {offered:?}, but this process already registered \
         {current:?}: one build has one catalog of happening generators (DECISIONS.md ARC-62)"
    );
}

/// The ids of the registered generators, in the order they are asked; [`None`] until something
/// registers.
pub fn registered_generators() -> Option<Vec<SystemId>> {
    CATALOG.get().map(|generators| ids(generators))
}

/// The registered generators a world selects, in ascending id order; the first selected id that is not
/// registered, otherwise.
pub(crate) fn selected(
    selection: &[SystemId],
) -> Result<Vec<&'static dyn HappeningGenerator>, SystemId> {
    let registered = CATALOG.get().map_or(&[][..], Vec::as_slice);
    let mut chosen: Vec<&'static dyn HappeningGenerator> = Vec::new();
    for id in selection {
        let generator = registered
            .iter()
            .find(|generator| generator.generator_of() == *id)
            .ok_or_else(|| id.clone())?;
        chosen.push(generator.as_ref());
    }
    chosen.sort_by_key(|generator| generator.generator_of());
    Ok(chosen)
}
