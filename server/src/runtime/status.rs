//! What the world thread says about the world it hosts: the status answer and the welcome's summary,
//! and the instant its first controllers are bound at.
//!
//! Read-only: nothing here changes the world, the seats or the clock.

use mineworld_contracts::WorldTime;
use mineworld_persistence::WorldRevision;

use super::WorldRuntime;
use crate::host::Hosted;
use crate::protocol::{PROTOCOL_VERSION, SystemSummary, WorldSummary};

impl WorldRuntime {
    /// What the world is, as a status answer or a welcome states it.
    pub(super) fn summary(&self) -> WorldSummary {
        let systems = self.world.world().systems();
        WorldSummary {
            protocol: PROTOCOL_VERSION,
            instance: self.instance,
            at: self.clock.now(),
            time_scale: self.clock.scale().get(),
            paused: self.clock.paused(),
            entities: self.world.world().entities().len(),
            systems: systems
                .order()
                .iter()
                .map(|system| {
                    // The declaration a system made when it was installed: its vocabulary, which is
                    // composition rather than state, so it is public (`PROTOCOL.md` §5.7).
                    let declaration = systems.declaration(system);
                    SystemSummary {
                        system: system.clone(),
                        enabled: systems.is_enabled(system),
                        provides: declaration.map_or_else(Vec::new, |d| d.provides().to_vec()),
                        states: declaration.map_or_else(Vec::new, |d| d.emits().to_vec()),
                    }
                })
                .collect(),
            seats: self.seats.iter().cloned().collect(),
            // Connections only: an in-server controller is not a client (`PROTOCOL.md` §5.7).
            clients: self.subscribers.len(),
            observations_dropped: self.dropped,
            // No fact is delivered to an observer before S11-C, so none is dropped.
            events_dropped: self.events_dropped,
            faults: self.faults,
            revision: self.world.revision(),
        }
    }
}

/// The instant the server's first controllers are bound at (`F-13`): every line heard at or before
/// it was said to whoever drove the Person before this process, and is not theirs to answer.
///
/// For a world resumed with history, that is the world's own instant — its last input. A world that
/// holds nothing but its genesis has heard nothing: its controllers are bound a second before it, so
/// that a line said in the first wall second of hosting, which the host's clock still stamps with the
/// genesis instant, is answered (step-12 D-SB3).
pub(super) fn first_binding(world: &Hosted, epoch: WorldTime) -> WorldTime {
    let only_genesis = match world {
        Hosted::Ephemeral(_) => true,
        Hosted::Persisted(world) => world.revision() == WorldRevision::GENESIS,
    };
    if only_genesis {
        WorldTime::from_seconds(epoch.seconds() - 1)
    } else {
        epoch
    }
}
