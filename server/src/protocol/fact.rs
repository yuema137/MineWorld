//! A recorded fact as a client receives it: `PROTOCOL.md` §5.2 "A fact on the wire".
//!
//! The log keeps a fact's payload as the bytes its owning pack encoded, and the kernel never
//! interprets them. A client has no Rust types, so the payload must reach it as a value it can read —
//! and every pack in this repository encodes JSON, so the bytes are read as JSON here. That is a
//! convention of the packs, not a contract (`DEP-5`), so a payload that is not JSON has a defined
//! form too: `null`, reported to the caller so that it can say so once rather than silently.
//!
//! The envelope is rebuilt **through the contract's own deserialization**: serialize the envelope,
//! replace the payload record's inner payload, read it back as `EventEnvelope<Value>`. The contract
//! offers no way to re-type a payload in place, and adding one is a contract change this crate may not
//! make; going through serde means the result is checked by the same agreement rule as a fact read
//! from disk, rather than assembled by hand here.

use std::collections::BTreeSet;
use std::sync::Mutex;

use mineworld_contracts::{EventEnvelope, EventId, EventTypeId, PerceivedEvent};
use serde_json::Value;

use super::ServerFrame;

/// The most facts one backfill `perceived` frame carries (`PROTOCOL.md` §5.8).
const BACKFILL_FRAME: usize = 256;

/// The `perceived` frames a joining connection is owed before its live stream (`PROTOCOL.md` §5.8),
/// read off the world's thread on a blocking task; none when its join asked for no backfill. A
/// history that cannot be read now is `cursor_unavailable`, and the connection may join again.
pub(crate) async fn read_backfill(
    start: Option<&crate::host::PerceivedStart>,
    observer: mineworld_contracts::EntityId,
) -> Result<Vec<ServerFrame>, super::Refusal> {
    let Some(crate::host::Backfill {
        history,
        since,
        through,
    }) = start.and_then(|start| start.backfill.clone())
    else {
        return Ok(Vec::new());
    };
    let read = tokio::task::spawn_blocking(move || {
        history
            .perceived(observer, since, through)
            .map(|facts| backfill_frames(&facts, through))
    })
    .await;
    let unavailable = || super::Refusal::new(super::RefusalCode::CursorUnavailable);
    match read {
        Ok(Ok(frames)) => Ok(frames),
        Ok(Err(cause)) => Err(unavailable().detailed(cause)),
        Err(cause) => Err(unavailable().detailed(cause)),
    }
}

/// A backfill as `perceived` frames: the facts, in their wire form, in frames of at most
/// [`BACKFILL_FRAME`]; the last frame's `through` is the head, and there is one even when no fact
/// is owed, so that the client learns its cursor.
fn backfill_frames(facts: &[EventEnvelope], head: EventId) -> Vec<ServerFrame> {
    let rendered: Vec<(EventId, PerceivedEvent<Value>)> = facts
        .iter()
        .filter_map(|fact| match wire_fact(fact) {
            Ok((event, form)) => {
                if form == PayloadForm::NotJson {
                    report_not_json(fact.event_type());
                }
                Some((fact.id(), event))
            }
            Err(error) => {
                eprintln!("[session] fact {} could not be written: {error}", fact.id());
                None
            }
        })
        .collect();
    let mut frames: Vec<ServerFrame> = rendered
        .chunks(BACKFILL_FRAME)
        .map(|chunk| ServerFrame::Perceived {
            through: chunk.last().map_or(head, |(id, _)| *id),
            events: chunk.iter().map(|(_, event)| event.clone()).collect(),
        })
        .collect();
    match frames.last_mut() {
        Some(ServerFrame::Perceived { through, .. }) => *through = head,
        _ => frames.push(ServerFrame::Perceived {
            through: head,
            events: Vec::new(),
        }),
    }
    frames
}

/// Says once per event type per process that a payload reached the wire as `null`, so that the
/// defined form is visible rather than silent (`PROTOCOL.md` §5.2).
pub(crate) fn report_not_json(event_type: &EventTypeId) {
    static REPORTED: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());
    let first = REPORTED
        .lock()
        .map(|mut reported| reported.insert(event_type.as_str().to_owned()))
        .unwrap_or(false);
    if first {
        println!("[world] event type {event_type} has a payload that is not JSON");
    }
}

/// Whether a fact's payload reached the wire as the pack wrote it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadForm {
    /// The payload's bytes were JSON and are carried as that value.
    Json,
    /// The payload's bytes were not JSON; the wire carries `null` in their place.
    NotJson,
}

/// `fact` in the form a `perceived` frame and `observation.events` carry it, and whether its payload
/// was JSON.
///
/// # Errors
///
/// When the contract refuses the rebuilt envelope or serde cannot represent it — impossible for an
/// envelope the contract built, so a caller treats it as a defect, not as a client's problem.
pub fn wire_fact(
    fact: &EventEnvelope,
) -> Result<(PerceivedEvent<Value>, PayloadForm), serde_json::Error> {
    let (payload, form) = match serde_json::from_slice::<Value>(fact.payload().payload()) {
        Ok(value) => (value, PayloadForm::Json),
        Err(_) => (Value::Null, PayloadForm::NotJson),
    };
    let mut whole = serde_json::to_value(fact)?;
    if let Some(record) = whole.get_mut("payload").and_then(Value::as_object_mut) {
        record.insert("payload".to_owned(), payload);
    }
    let envelope: EventEnvelope<Value> = serde_json::from_value(whole)?;
    Ok((PerceivedEvent::new(envelope), form))
}

#[cfg(test)]
mod tests {
    use mineworld_contracts::{
        Causation, EntityId, EntityType, Event, EventId, EventRecord, EventSchemaVersion,
        EventTypeId, PlaceId, Provenance, SystemId, Visibility, WorldTime,
    };
    use serde_json::json;

    use super::*;

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Rang;

    impl Event for Rang {
        const EVENT_TYPE: EventTypeId = EventTypeId::from_static("rang");
        const OWNER: SystemId = SystemId::from_static("bells");
        const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    }

    fn fact(payload: &[u8]) -> EventEnvelope {
        let hall = PlaceId::new(EntityId::from_raw(3), EntityType::Place).expect("a place");
        EventEnvelope::new(
            EventId::from_raw(1890),
            WorldTime::from_seconds(4_100),
            EventRecord::new::<Rang>(payload.to_vec()),
            Causation::WorldGenesis,
            Visibility::Place(hall),
            Provenance::new(SystemId::from_static("bells")),
        )
        .about(vec![EntityId::from_raw(5)])
        .with_participants(vec![EntityId::from_raw(5), EntityId::from_raw(7)])
        .at_place(hall)
    }

    #[test]
    fn a_json_payload_is_carried_as_its_value_in_the_contract_envelope() {
        let (event, form) = wire_fact(&fact(br#"{"bell":"low","times":3}"#)).expect("renders");
        assert_eq!(form, PayloadForm::Json);
        let written = serde_json::to_value(&event).expect("serializes");
        let expected = json!({
            "id": "1890",
            "at": 4100,
            "event_type": "rang",
            "subjects": ["5"],
            "participants": ["5", "7"],
            "place": written["place"].clone(),
            "caused_by": written["caused_by"].clone(),
            "payload": { "event_type": "rang", "schema_version": 1,
                         "payload": { "bell": "low", "times": 3 } },
            "visibility": written["visibility"].clone(),
            "provenance": written["provenance"].clone(),
        });
        assert_eq!(written, expected);
        // The fields taken from the output above are the contract's own forms: check them against
        // the contract's serialization of the original envelope, not against themselves.
        let original = serde_json::to_value(fact(b"{}")).expect("serializes");
        for field in ["place", "caused_by", "visibility", "provenance"] {
            assert_eq!(written[field], original[field], "{field}");
        }
    }

    #[test]
    fn a_payload_that_is_not_json_is_null_and_said_so() {
        let (event, form) = wire_fact(&fact(b"\x00\x01 not json")).expect("renders");
        assert_eq!(form, PayloadForm::NotJson);
        assert_eq!(event.envelope().payload().payload(), &Value::Null);
        assert_eq!(event.envelope().event_type().as_str(), "rang");
    }

    #[test]
    fn the_contract_still_refuses_an_envelope_whose_halves_disagree() {
        let (event, _) = wire_fact(&fact(b"{}")).expect("renders");
        let mut tampered = serde_json::to_value(&event).expect("serializes");
        tampered["event_type"] = json!("tolled");
        assert!(serde_json::from_value::<PerceivedEvent<Value>>(tampered).is_err());
    }
}
