//! An event is an immutable fact. Its cause and its declared audience must not be reachable by
//! assignment: `INV-15` and the declared-audience rule are only guarantees if the value that
//! reached the log cannot be altered afterwards.

use mineworld_contracts::{EventSchemaVersion, 
    Causation, Event, EventEnvelope, EventId, EventRecord, EventTypeId, Provenance, SystemId,
    Visibility, WorldTime,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ExampleHappened {
    count: u32,
}

impl Event for ExampleHappened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("example-happened");
    const OWNER: SystemId = SystemId::from_static("example-system");
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn main() {
    let mut recorded: EventEnvelope = EventEnvelope::new(
        EventId::from_raw(1),
        WorldTime::EPOCH,
        EventRecord::new::<ExampleHappened>(Vec::new()),
        Causation::WorldGenesis,
        Visibility::SystemInternal,
        Provenance::new(SystemId::from_static("example-system")),
    );
    recorded.caused_by = Causation::Event(EventId::from_raw(2));
    recorded.visibility = Visibility::Public;
}
