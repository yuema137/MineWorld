//! Advance stop adapters: how a host learns, from authoritative state, when a single-player advance
//! must stop (`docs/DECISIONS.md` `ARC-374`; pr-nightlife N2-I4).
//!
//! A host may run a sleeping player's night faster than wall time. The host knows only this trait,
//! never what sleep is: a System Pack implements it over the state it owns and publishes it through one
//! `extension` line of the installed set (`ARC-62`), and the host finds it by name. An adapter answers
//! three questions, each from the world as it stands, and decides nothing:
//!
//! ```text
//! identify  is this Person in the state the adapter stops on the end of, and which occurrence of it
//! horizon   the latest instant that occurrence can last to, when it has one (an alarm)
//! holds     whether that occurrence still holds — false once it ended, however it ended
//! ```
//!
//! The catalog is the build's, the same in every world in the process; a world uses an adapter only
//! when the adapter's [`system`](AdvanceStop::system) is enabled in it.

use std::sync::OnceLock;

use mineworld_contracts::{EntityId, SystemId, WorldTime};
use mineworld_kernel::WorldRead;

/// Which occurrence of a state an advance runs through — an adapter's own opaque number for it (a sleep's
/// Process, say). The host compares it and keeps it; it never interprets it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StopIdentity(u64);

impl StopIdentity {
    /// The identity an adapter writes as `value`.
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Its value.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// When an advance stops, answered by the System Pack that owns the state it stops on.
///
/// Every method reads; none changes the world, and each must answer the same for the same state.
pub trait AdvanceStop: Send + Sync {
    /// The name a request names it by (`until`), unique in the build.
    fn name(&self) -> &'static str;

    /// The system whose state it reads: a world in which that system is not enabled has no such adapter.
    fn system(&self) -> SystemId;

    /// The occurrence `person` is in now, or `None` when they are not in the state at all.
    fn identify(&self, world: &WorldRead<'_>, person: EntityId) -> Option<StopIdentity>;

    /// The latest instant `occurrence` can last to, or `None` when it has ended or has no bound.
    fn horizon(
        &self,
        world: &WorldRead<'_>,
        person: EntityId,
        occurrence: StopIdentity,
    ) -> Option<WorldTime>;

    /// Whether `person` is still in `occurrence`.
    fn holds(&self, world: &WorldRead<'_>, person: EntityId, occurrence: StopIdentity) -> bool;
}

/// The build's adapters, sorted by name: written once per process by [`register_stops`].
static CATALOG: OnceLock<Vec<Box<dyn AdvanceStop>>> = OnceLock::new();

fn names(stops: &[Box<dyn AdvanceStop>]) -> Vec<&'static str> {
    stops.iter().map(|stop| stop.name()).collect()
}

/// Registers the build's advance stop adapters: once per process, before any world is hosted, by the one
/// assembly path every host uses (`Capability::register_extensions`). Registering the same names again
/// does nothing, which lets every composition register.
///
/// # Panics
///
/// When the list names one adapter twice, and when a different list was registered before (`ARC-62`
/// item 4).
pub fn register_stops(mut stops: Vec<Box<dyn AdvanceStop>>) {
    stops.sort_by_key(|stop| stop.name());
    let offered = names(&stops);
    if let Some(pair) = offered.windows(2).find(|pair| pair[0] == pair[1]) {
        panic!(
            "register_stops was given two advance stop adapters named '{}' (DECISIONS.md ARC-374)",
            pair[0]
        );
    }
    let mut pending = Some(stops);
    let registered = CATALOG.get_or_init(|| pending.take().unwrap_or_default());
    if pending.is_none() {
        return;
    }
    let current = names(registered);
    assert!(
        current == offered,
        "register_stops was called with {offered:?}, but this process already registered {current:?}: \
         one build has one catalog of advance stop adapters (DECISIONS.md ARC-62, ARC-374)"
    );
}

/// The registered adapter named `name`, if the build has one.
pub fn stop_named(name: &str) -> Option<&'static dyn AdvanceStop> {
    CATALOG
        .get()?
        .iter()
        .find(|stop| stop.name() == name)
        .map(AsRef::as_ref)
}
