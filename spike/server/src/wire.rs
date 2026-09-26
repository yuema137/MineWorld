//! `DD-15` made real: the protocol-boundary encoding that renders 64-bit ids as decimal strings.
//!
//! The decision this implements says the contract layer keeps integer ids and the *wire*
//! representation is the protocol's responsibility, because Godot's `JSON.parse_string` returns
//! every number as a `float`. This module is that responsibility discharged, so that the cost of
//! discharging it can be measured rather than assumed.
//!
//! # It cannot be written generically, and that is the finding
//!
//! Every id newtype in the contract layer is `#[serde(transparent)]` over a `u64`, so the JSON a
//! contract type serializes to carries **no type information at all**: `9007199254740995` is an
//! `EntityId`, and `9007199254740995` is a signage catalogue number, and the JSON cannot tell
//! them apart. An encoder therefore cannot walk the tree looking for large integers, and cannot
//! key on field names either — `ComponentRecord.payload` and `EventRecord.payload` are opaque by
//! design and may contain any field name at all, including `id`.
//!
//! What is left is a **structural mirror**: one function per contract shape, walking exactly the
//! positions the contract puts an id in, and refusing to descend into a payload. That is what
//! follows. It is correct, and it must be re-verified by hand every time a contract type gains a
//! field. `FINDINGS.md` F2 records what that costs.

use serde_json::{Map, Value};

/// Which way a conversion runs.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Contract JSON (numbers) to wire JSON (decimal strings).
    Encode,
    /// Wire JSON (decimal strings) back to contract JSON (numbers).
    Decode,
}

impl Direction {
    /// Converts one id in place. A value that is already in the target form is left alone, so
    /// the walk is idempotent and a partially converted frame cannot be produced silently.
    fn id(self, value: &mut Value) {
        match (self, &*value) {
            (Self::Encode, Value::Number(number)) => {
                *value = Value::String(number.to_string());
            }
            (Self::Decode, Value::String(text)) => {
                if let Ok(parsed) = text.parse::<u64>() {
                    *value = Value::from(parsed);
                } else if let Ok(parsed) = text.parse::<i64>() {
                    *value = Value::from(parsed);
                }
            }
            _ => {}
        }
    }

    fn field(self, object: &mut Map<String, Value>, key: &str) {
        if let Some(value) = object.get_mut(key) {
            self.id(value);
        }
    }

    fn id_list(self, object: &mut Map<String, Value>, key: &str) {
        if let Some(Value::Array(items)) = object.get_mut(key) {
            for item in items {
                self.id(item);
            }
        }
    }

    /// `PersonId`, `PlaceId`, `ItemId`, `OrganizationId` all serialize as
    /// `{"entity": <u64>, "entity_type": "place"}`.
    fn typed_ref(self, value: &mut Value) {
        if let Value::Object(object) = value {
            self.field(object, "entity");
        }
    }

    fn typed_ref_field(self, object: &mut Map<String, Value>, key: &str) {
        if let Some(value) = object.get_mut(key) {
            self.typed_ref(value);
        }
    }

    /// `Location { place: PlaceId, local: Option<LocalPosition>, facing: Option<Orientation> }`.
    /// Only `place` carries an id; the millimetres and millidegrees are plain `i32` and stay
    /// numbers, which is the whole reason the spatial contract is safe across this boundary.
    fn location(self, value: &mut Value) {
        if let Value::Object(object) = value {
            self.typed_ref_field(object, "place");
        }
    }

    fn location_field(self, object: &mut Map<String, Value>, key: &str) {
        if let Some(value) = object.get_mut(key) {
            self.location(value);
        }
    }

    /// `PlaceRequirement` is `"any" | "same_place_as_actor" | {"specific": PlaceId}`.
    fn spatial_requirement(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        if let Some(Value::Object(place)) = object.get_mut("place")
            && let Some(specific) = place.get_mut("specific")
        {
            self.typed_ref(specific);
        }
    }

    /// `PerceivedEntity`. The components' payloads are deliberately **not** walked.
    fn perceived_entity(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        self.field(object, "id");
        self.location_field(object, "location");
        if let Some(Value::Array(components)) = object.get_mut("components") {
            for component in components {
                if let Value::Object(record) = component {
                    self.field(record, "entity");
                    // `record["payload"]` is opaque. Descending into it would corrupt any
                    // payload field that happens to be called `id` or `entity`.
                }
            }
        }
    }

    /// `Affordance`.
    fn affordance(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        self.field(object, "target");
        if let Some(requirement) = object.get_mut("requirement") {
            self.spatial_requirement(requirement);
        }
    }

    /// `Causation` is `"world_genesis" | {"action": ActionId} | {"process": ProcessId} |
    /// {"event": EventId} | {"system_tick": {"system": SystemId}}`.
    fn causation(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        for key in ["action", "process", "event"] {
            self.field(object, key);
        }
    }

    /// `Visibility` is `"public" | "participants" | "system_internal" | {"place": PlaceId} |
    /// {"entities": [EntityId]}`.
    fn visibility(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        self.typed_ref_field(object, "place");
        self.id_list(object, "entities");
    }

    /// `EventEnvelope`, as `PerceivedEvent` transparently wraps one. Payload not walked.
    fn event_envelope(self, value: &mut Value) {
        let Value::Object(object) = value else {
            return;
        };
        self.field(object, "id");
        self.id_list(object, "subjects");
        self.id_list(object, "participants");
        self.typed_ref_field(object, "place");
        if let Some(caused_by) = object.get_mut("caused_by") {
            self.causation(caused_by);
        }
        if let Some(visibility) = object.get_mut("visibility") {
            self.visibility(visibility);
        }
        if let Some(Value::Object(provenance)) = object.get_mut("provenance") {
            self.field(provenance, "controller_decision");
        }
    }
}

/// Converts a serialized `Observation` between contract JSON and wire JSON.
///
/// Nine id-bearing positions, in six nested shapes. That count is the measurement `FINDINGS.md`
/// F2 reports.
pub fn observation(direction: Direction, value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    direction.field(object, "observer");
    direction.location_field(object, "self_location");
    if let Some(Value::Array(entities)) = object.get_mut("entities") {
        for entity in entities {
            direction.perceived_entity(entity);
        }
    }
    if let Some(Value::Array(events)) = object.get_mut("events") {
        for event in events {
            direction.event_envelope(event);
        }
    }
    if let Some(Value::Array(affordances)) = object.get_mut("affordances") {
        for affordance in affordances {
            direction.affordance(affordance);
        }
    }
}

/// Converts a serialized `ActionIntent` between contract JSON and wire JSON.
pub fn action_intent(direction: Direction, value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    direction.field(object, "action_id");
    direction.field(object, "actor");
    direction.field(object, "target");
    direction.location_field(object, "actor_location");
    // `payload` is an `ActionRecord`: opaque, not walked.
}

/// Converts a serialized `ActionResult` between contract JSON and wire JSON.
///
/// `ActionResult::Accepted { events: Vec<EventId> }` is the reason this exists at all: it is the
/// exact frame the 2026-09-25 networking spike watched Godot turn into `[9001.0]`.
pub fn action_result(direction: Direction, value: &mut Value) {
    let Value::Object(object) = value else {
        return;
    };
    if let Some(Value::Object(accepted)) = object.get_mut("accepted") {
        direction.id_list(accepted, "events");
    }
}
