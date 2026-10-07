//! A day survives a restart: the routine process, saved before a boundary, is woken after it by the
//! owner, in a freshly composed world resumed from the file (`step-09-social.md` C5).

mod support;

use std::path::PathBuf;

use mineworld_contracts::Causation;
use mineworld_persistence::{
    Creation, Durability, PersistenceBackend, PersistentWorld, SqliteBackend, verify,
};
use mineworld_schedule::{Agenda, AgendaChanged};
use support::{GENESIS, Town, compose, t, types};

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0010_0c05;
const SIX_AM: i64 = 6 * 3_600;

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("mineworld-schedule-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("a scratch directory");
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn open(scratch: &Scratch) -> Box<dyn PersistenceBackend> {
    Box::new(SqliteBackend::open(&scratch.0, Durability::ProcessCrash).expect("opens"))
}

#[test]
fn a_routine_saved_before_a_boundary_is_woken_by_its_owner_after_a_restart() {
    let scratch = Scratch::new("routine");
    let (world, providers) = compose();
    let (town, facts) = Town::assemble(world, providers);
    let alice = town.alice;
    let backend = SqliteBackend::create(&scratch.0, Durability::ProcessCrash).expect("creates");
    let (mut persisted, _) = PersistentWorld::create(
        Box::new(backend),
        town.world,
        Creation {
            instance: INSTANCE,
            pack: "schedule-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    let quiet = persisted.advance_to(t(SIX_AM - 1)).expect("advances");
    assert!(quiet.events().is_empty(), "nothing before the boundary");
    let routine = persisted
        .world()
        .read()
        .component::<Agenda>(alice)
        .expect("an agenda")
        .routine();
    drop(persisted);

    let (composed, _) = compose();
    let (mut resumed, how) = PersistentWorld::resume(open(&scratch), composed).expect("resumes");
    println!(
        "resumed: snapshot {:?}, replayed {}",
        how.snapshot, how.replayed
    );
    let found = resumed.world().read().process(routine).cloned();
    assert_eq!(
        found
            .as_ref()
            .and_then(mineworld_kernel::Process::expected_end),
        Some(t(SIX_AM)),
        "the routine process is running in the resumed world, due at the boundary: {found:?}"
    );

    let due = resumed.advance_to(t(SIX_AM)).expect("advances");
    assert_eq!(types(due.events()), ["agenda-changed"]);
    assert_eq!(
        *due.events()[0].caused_by(),
        Causation::Process(routine),
        "caused by the same process, across the restart"
    );
    let changed: AgendaChanged = serde_json::from_slice(
        due.events()[0]
            .payload()
            .payload_for::<AgendaChanged>()
            .expect("schedule's fact"),
    )
    .expect("decodes");
    assert_eq!(changed.label().as_str(), "work");
    drop(resumed);

    let verified =
        verify(open(&scratch).as_ref(), compose().0).expect("the history verifies from genesis");
    println!("verified {} revisions", verified.revisions);
    assert!(
        verified.revisions >= 2,
        "genesis, and the advance that fired the wake"
    );
}
