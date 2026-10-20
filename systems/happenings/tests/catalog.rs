//! T13 (AC-WE-a-6): user-written generators through the `ARC-62` catalog. Its own test binary, because
//! the catalog is write-once per process: this binary registers two test-only generator packs and
//! composes its worlds by hand, so it never meets the installed set's (empty) registration (U-3).
//!
//! A selected generator's drafts pass the host's admission with typed reasons; an unselected one is never
//! asked; a selected id this build never registered is refused at seeding, by name; a generator whose
//! pack is not in the world is inert.

mod support;

use std::collections::BTreeMap;

use mineworld_authoring::{Attached, ConfigurationContext, EntityClasses, Seeding};
use mineworld_contracts::{EntityType, PlaceId, Rejection, SystemId};
use mineworld_happenings::{
    DraftAsk, DraftSource, HappeningAnnounced, HappeningDraft, HappeningGenerator, HappeningKindId,
    HappeningWithheld, HappeningsSystem, MAX_DRAFTS_PER_GENERATOR, WithheldReason,
    register_generators, registered_generators,
};
use mineworld_kernel::{
    Declarations, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, owned_component,
};
use mineworld_presence::{PerceptionProvider, PresenceSystem};
use serde::{Deserialize, Serialize};
use support::{DAY, of, t};

/// A test-only generator pack: it owns one declared record no entity carries — its "state" — and its
/// generator drafts only where that record is declared, that is, where the pack is installed.
#[derive(Default)]
struct FairMaker;

impl SystemIdentity for FairMaker {
    const ID: SystemId = SystemId::from_static("fair-maker");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FairStall;

owned_component! {
    component = FairStall,
    owner = FairMaker,
    component_type = "fair-stall",
    schema_version = 1,
}

impl System for FairMaker {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<FairStall>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<FairStall>()
    }
}

impl PerceptionProvider for FairMaker {}

/// Whether `owner`'s pack is installed in the world: it declares its record.
fn installed(world: &WorldRead<'_>, owner: &SystemId) -> bool {
    world
        .component_declarations()
        .any(|declaration| declaration.owner() == owner)
}

/// The square's place id in the support town.
fn square(world: &WorldRead<'_>) -> PlaceId {
    let id = world
        .resolve_key(&support::key("square"))
        .expect("the square");
    PlaceId::new(id, EntityType::Place).expect("a place")
}

fn kind(id: &str) -> HappeningKindId {
    HappeningKindId::new(id).expect("an id")
}

/// fair-maker's generator: each day `d`, one admissible fair at the square on `d`, then one draft for
/// each refusal — an undeclared kind, a place outside the kind's, the past, beyond the horizon — then
/// duplicates of the fair until the cap is passed by two. Only where its pack is installed.
#[derive(Default)]
struct FairGenerator;

impl HappeningGenerator for FairGenerator {
    fn generator_of(&self) -> SystemId {
        FairMaker::ID
    }

    fn drafts(&self, world: &WorldRead<'_>, ask: &DraftAsk<'_>) -> Vec<HappeningDraft> {
        if !installed(world, &FairMaker::ID) {
            return Vec::new();
        }
        let (square, d) = (square(world), ask.day);
        let street = PlaceId::new(
            world.resolve_key(&support::key("street")).expect("street"),
            EntityType::Place,
        )
        .expect("a place");
        let mut drafts = vec![
            HappeningDraft::new(kind("fair"), square, d),
            HappeningDraft::new(kind("juggling"), square, d),
            HappeningDraft::new(kind("fair"), street, d),
            HappeningDraft::new(kind("fair"), square, d.plus(-1)),
            HappeningDraft::new(kind("fair"), square, d.plus(5)),
        ];
        // A draw from the ask, so the generator's draws are exercised: a pure function of the day.
        assert!(ask.permille(d, 0) < 1_000);
        while drafts.len() < MAX_DRAFTS_PER_GENERATOR + 2 {
            drafts.push(HappeningDraft::new(kind("fair"), square, d));
        }
        drafts
    }
}

/// quiet-maker's generator: an admissible draft every day — so if it were ever asked, it would show.
#[derive(Default)]
struct QuietGenerator;

impl HappeningGenerator for QuietGenerator {
    fn generator_of(&self) -> SystemId {
        SystemId::from_static("quiet-maker")
    }

    fn drafts(&self, world: &WorldRead<'_>, ask: &DraftAsk<'_>) -> Vec<HappeningDraft> {
        vec![HappeningDraft::new(kind("fair"), square(world), ask.day)]
    }
}

/// Registers this binary's catalog, as `worldpack::compose` registers the installed set's: in any
/// order, any number of times.
fn register() {
    register_generators(vec![Box::new(QuietGenerator), Box::new(FairGenerator)]);
}

/// A fair at the square, 10:00–12:00, announced up to 2 days ahead; no calendar or random entries.
fn configuration(generators: &str) -> String {
    format!(
        "seed: 7
kinds:
  - {{ id: fair, places: [square], hours: \"10:00-12:00\", announce_days: 2 }}
limits: {{ per_day: 3, per_place_running: 1 }}
generators: {generators}
"
    )
}

/// presence, calendar, happenings, the placer and — if `fair` — fair-maker.
fn compose(fair: bool) -> World {
    let mut world = support::compose(true);
    if fair {
        world.install(FairMaker).expect("fair-maker installs");
    }
    world
}

#[test]
fn the_catalog_is_sorted_and_registering_again_is_a_no_op() {
    register();
    register_generators(vec![Box::new(FairGenerator), Box::new(QuietGenerator)]);
    assert_eq!(
        registered_generators(),
        Some(vec![
            SystemId::from_static("fair-maker"),
            SystemId::from_static("quiet-maker")
        ])
    );
}

/// A selected generator's drafts are admitted or withheld, each with its typed reason, and the drafts
/// beyond the cap are withheld together in one fact; the unselected quiet-maker is never asked.
#[test]
fn a_selected_generator_drafts_through_admission() {
    register();
    let (mut world, _, facts) = support::assemble(
        compose(true),
        Some(support::SAN_DIEGO),
        Some(&configuration("[fair-maker]")),
    );
    let mut all = world.genesis(t(0), facts).expect("genesis");
    all.extend(
        world
            .advance_to(t(3 * DAY - 1))
            .expect("advances")
            .into_events(),
    );

    let fair_maker = DraftSource::Generator(FairMaker::ID);
    let announced = of::<HappeningAnnounced>(&all);
    assert_eq!(announced.len(), 3, "one fair a day, days 0 … 2");
    assert!(
        announced
            .iter()
            .all(|(_, fact)| *fact.source() == fair_maker)
    );

    let withheld = of::<HappeningWithheld>(&all);
    let mut reasons: BTreeMap<String, usize> = BTreeMap::new();
    for (_, fact) in &withheld {
        assert_eq!(*fact.source(), fair_maker, "quiet-maker is never asked");
        *reasons.entry(format!("{:?}", fact.reason())).or_default() += 1;
        if fact.reason() == WithheldReason::OverGeneratorCap {
            assert_eq!(fact.excess(), Some(2));
        } else {
            assert_eq!(fact.excess(), None);
        }
    }
    let per_day: BTreeMap<String, usize> = [
        ("UnknownKind", 1),
        ("PlaceNotAdmitted", 1),
        ("OutsideWindow", 2),
        ("Duplicate", MAX_DRAFTS_PER_GENERATOR - 5),
        ("OverGeneratorCap", 1),
    ]
    .into_iter()
    .map(|(reason, count)| (reason.to_owned(), count * 3))
    .collect();
    assert_eq!(reasons, per_day);
}

/// A world that selects no generator is untouched by registered ones.
#[test]
fn an_unselected_generator_is_never_asked() {
    register();
    let (mut world, _, facts) = support::assemble(
        compose(true),
        Some(support::SAN_DIEGO),
        Some(&configuration("[]")),
    );
    let mut all = world.genesis(t(0), facts).expect("genesis");
    all.extend(
        world
            .advance_to(t(3 * DAY))
            .expect("advances")
            .into_events(),
    );
    assert!(of::<HappeningAnnounced>(&all).is_empty());
    assert!(of::<HappeningWithheld>(&all).is_empty());
}

/// A selected id this build never registered is refused when the world is seeded, naming it.
#[test]
fn a_selected_generator_this_build_lacks_is_refused_at_seeding() {
    register();
    let world = compose(true);
    let read = world.read();
    let keys = BTreeMap::new();
    let (classes, attached) = (EntityClasses::default(), Attached::none());
    let refusal = support::decoded::<HappeningsSystem>(&configuration("[ghost-maker]"))
        .seed(
            &Seeding::new(&read, &keys),
            &ConfigurationContext::new(&classes, &attached),
        )
        .expect_err("refused");
    let Rejection::System { code, detail } = refusal else {
        panic!("a system refusal: {refusal:?}");
    };
    assert_eq!(code.as_str(), "happenings-generator-unknown");
    let detail = detail.expect("a detail");
    assert!(detail.contains("'ghost-maker'"), "{detail}");
    // The configuration requires each selected generator's pack, so the loader refuses a world that
    // does not enable it, by name.
    let configured = support::decoded::<HappeningsSystem>(&configuration("[fair-maker]"));
    assert_eq!(configured.requires(), [FairMaker::ID]);
}

/// A generator whose pack the world does not install drafts nothing: inert where its state is absent.
#[test]
fn a_generator_without_its_pack_is_inert() {
    register();
    let (mut world, _, facts) = support::assemble(
        compose(false),
        Some(support::SAN_DIEGO),
        Some(&configuration("[fair-maker]")),
    );
    let mut all = world.genesis(t(0), facts).expect("genesis");
    all.extend(
        world
            .advance_to(t(3 * DAY))
            .expect("advances")
            .into_events(),
    );
    assert!(of::<HappeningAnnounced>(&all).is_empty());
    assert!(of::<HappeningWithheld>(&all).is_empty());
}
