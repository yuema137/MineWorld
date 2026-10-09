//! The seated connections as the transport knows them: who joined with which nickname, as which
//! seat, since when, and how many stream frames each has been sent (`PROTOCOL.md` §11.2).
//!
//! Transport-side on purpose. The nickname exists only in a connection's own task and never reaches
//! the world thread (step-12 SD-A8, I-5), so `GET /admin/sessions` reads it here; the world thread
//! knows only what it alone can count, each connection's dropped observations, and the admin route
//! merges the two (SD-D5).
//!
//! A `std` mutex, held for a map insert, remove or copy and never across an `.await`.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use mineworld_contracts::{EntityId, EntityKey};

use crate::admission::Nickname;
use crate::protocol::SessionId;

/// Every seated connection the transport is serving.
#[derive(Debug, Default)]
pub(crate) struct Registry {
    sessions: Mutex<BTreeMap<SessionId, Entry>>,
}

/// What a session tells the registry at its welcome.
#[derive(Debug, Clone)]
pub(crate) struct Joined {
    pub(crate) nickname: Nickname,
    pub(crate) seat: EntityKey,
    pub(crate) observer: EntityId,
    /// Wall-clock Unix seconds at the welcome: host state, never a `WorldTime`.
    pub(crate) connected_at: u64,
}

#[derive(Debug)]
struct Entry {
    joined: Joined,
    seq: Arc<AtomicU64>,
}

/// One session as the registry lists it.
#[derive(Debug, Clone)]
pub(crate) struct Listed {
    pub(crate) session: SessionId,
    pub(crate) joined: Joined,
    pub(crate) seq: u64,
}

impl Registry {
    /// Records a seated session. It is listed until the returned guard is dropped.
    pub(crate) fn register(self: &Arc<Self>, session: SessionId, joined: Joined) -> Registered {
        let seq = Arc::new(AtomicU64::new(0));
        self.lock().insert(
            session,
            Entry {
                joined,
                seq: Arc::clone(&seq),
            },
        );
        Registered {
            registry: Arc::clone(self),
            session,
            seq,
        }
    }

    /// Every registered session, in session order.
    pub(crate) fn listed(&self) -> Vec<Listed> {
        self.lock()
            .iter()
            .map(|(session, entry)| Listed {
                session: *session,
                joined: entry.joined.clone(),
                seq: entry.seq.load(Ordering::Relaxed),
            })
            .collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<SessionId, Entry>> {
        // A panic while holding the lock leaves a map that is still a valid map.
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A session's place in the registry, removed when this is dropped — however the session ends.
#[derive(Debug)]
pub(crate) struct Registered {
    registry: Arc<Registry>,
    session: SessionId,
    seq: Arc<AtomicU64>,
}

impl Registered {
    /// Numbers the next stream frame this session sends: 1, 2, 3 …
    pub(crate) fn next_seq(&self) -> u64 {
        self.seq.fetch_add(1, Ordering::Relaxed) + 1
    }
}

impl Drop for Registered {
    fn drop(&mut self) {
        self.registry.lock().remove(&self.session);
    }
}
