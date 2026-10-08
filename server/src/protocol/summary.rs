//! What a world is, as a client or an operator is told: which running world, and what it is composed
//! of.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use mineworld_contracts::{EntityKey, SystemId, WorldTime};
use mineworld_persistence::WorldRevision;
use serde::{Deserialize, Serialize};

use super::ProtocolError;

/// Which running world this is.
///
/// Not a name, not a secret and not a UUID: a value that distinguishes *this* running world from
/// another one. It exists because `docs/MVP.md` §9.1 requires `AC-15`'s evidence to name identity
/// rather than appearance — *same world instance*, first of three lines — and without it the only
/// argument that two clients are connected to one world is that somebody typed one address twice.
/// Two clients that were each talking to their own server would be told two different instances
/// here, which is exactly the false success §9.1 exists to exclude.
///
/// Allocated when a world's thread starts (`runtime::WorldRuntime::new`) from the wall clock, the
/// process and a per-process ordinal, so two worlds never share one — in the same process because
/// the ordinal differs, and across processes because the instant and the process do. It is
/// deliberately **not** derived from the world's content: two worlds loaded from the same World
/// Pack are two instances, and `AC-15` is about the instance.
///
/// On the wire it is a lowercase hexadecimal string, for the reason every identity is a string
/// (`DEP-3`): a JSON parser whose only number type is a double cannot carry 128 bits, and cannot
/// carry 64 either.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct WorldInstanceId(u128);

impl WorldInstanceId {
    /// A new instance identity, for a world being created — by the world thread for a world that is
    /// not persisted, or by whoever creates a save, which then keeps it for the world's whole life.
    pub fn allocate() -> Self {
        // A world within this process, and this process at this instant. Not a cryptographic
        // identity: nothing authenticates with it, and the only property required is that two
        // worlds do not collide.
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let ordinal = u128::from(NEXT.fetch_add(1, Ordering::Relaxed));
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        let process = u128::from(std::process::id());
        Self((nanos << 32) ^ (process << 16) ^ ordinal)
    }

    /// The identity a caller already holds — for a test, or for reading one back off the wire.
    pub const fn from_raw(value: u128) -> Self {
        Self(value)
    }

    /// The identity as the number it is.
    pub const fn raw(self) -> u128 {
        self.0
    }
}

impl fmt::Display for WorldInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl From<WorldInstanceId> for String {
    fn from(value: WorldInstanceId) -> Self {
        value.to_string()
    }
}

impl TryFrom<String> for WorldInstanceId {
    type Error = ProtocolError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        u128::from_str_radix(&value, 16)
            .map(Self)
            .map_err(|_| ProtocolError::InstanceId(value))
    }
}

/// What a world is, as a client or an operator is told: enough to see that it is running and what
/// it is composed of, and nothing that would make this an observation.
///
/// Deliberately not a view of state. It names the systems a world installed — which is public
/// information about its composition, the same information a World Pack states — and counts its
/// entities. It lists no entity, no component and no position, because a client's knowledge of
/// state arrives only through an `Observation` (`INV-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSummary {
    /// Which revision of this protocol the server speaks.
    pub protocol: u32,
    /// Which running world this is — the same value for every client connected to it.
    pub instance: WorldInstanceId,
    /// The world's own clock, as of this answer.
    pub at: WorldTime,
    /// How many entities the world has allocated.
    pub entities: usize,
    /// The systems this world is composed of, in registration order.
    pub systems: Vec<SystemSummary>,
    /// The seats a client may ask for.
    pub seats: Vec<EntityKey>,
    /// How many clients are connected and seated.
    pub clients: usize,
    /// How many observations have been dropped because a client was not reading them.
    ///
    /// Reported rather than hidden: the world delivers observations without waiting for anybody, so
    /// a slow client loses frames, and a number that only ever appeared in a comment would make
    /// that policy invisible to whoever is running the server.
    pub observations_dropped: u64,
    /// How many deferrals had no scheduler to queue them.
    ///
    /// Always zero since S4: the world's own schedule holds every deferral and fires it at its
    /// instant. Kept on the wire until the next protocol revision removes it, because removing a
    /// field is a protocol change (step-04 §8 F-4).
    pub deferrals_unscheduled: u64,
    /// How many dispatches ended in a system breaking its own contract.
    ///
    /// `kernel/src/dispatch.rs`: an error out of dispatch is a bug in a system, not a rejected
    /// request. This server keeps serving and counts them here.
    pub faults: u64,
    /// The world's persisted head — the last revision committed to its save — or `None` for a world
    /// that is not persisted (`PROTOCOL.md` §5, `docs/DECISIONS.md` `ARC-25`).
    pub revision: Option<WorldRevision>,
}

/// One installed system, as a status answer names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSummary {
    /// The system's name.
    pub system: SystemId,
    /// Whether it is currently in the pipeline. A disabled system's actions are `Unavailable`.
    pub enabled: bool,
}
