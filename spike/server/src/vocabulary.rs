//! The System Pack vocabulary this spike invents, so the server has something to say.
//!
//! Every type here plays the part a real System Pack would play. None of it belongs to the
//! contract layer: `talk`, `move-to`, `display-name` and `place-extent` are exactly the domain
//! concepts `contracts/src/lib.rs` rule 1 forbids in `mineworld-contracts`. They are declared
//! here to prove the contract layer can carry them without knowing them.

use mineworld_contracts::{
    Action, ActionTypeId, Component, ComponentSchemaVersion, ComponentTypeId, Event,
    EventSchemaVersion, EventTypeId, LocalPosition, Orientation, SystemId,
};
use serde::{Deserialize, Serialize};

pub const IDENTITY: SystemId = SystemId::from_static("identity");
pub const GEOGRAPHY: SystemId = SystemId::from_static("geography");
pub const PHYSIQUE: SystemId = SystemId::from_static("physique");
pub const CONVERSATION: SystemId = SystemId::from_static("conversation");
pub const MOVEMENT: SystemId = SystemId::from_static("movement");
pub const INVENTORY: SystemId = SystemId::from_static("inventory");

// -------------------------------------------------------------------------------------------
// Components — the state a client reads in order to be able to draw anything at all.
// -------------------------------------------------------------------------------------------

/// What a thing is called. `DD-13`: the client maps an action type to a verb, and reads the
/// *name* of the target from world data, never inventing it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayName {
    pub name: String,
}

impl Component for DisplayName {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("display-name");
    const OWNER: SystemId = IDENTITY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// How big a body is, in millimetres. A 3D client needs this to size a capsule and a 2D client
/// to size a disc; both need it to be world data, because collision is a world fact and not a
/// rendering preference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub height_mm: i32,
    pub radius_mm: i32,
}

impl Component for Body {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("body");
    const OWNER: SystemId = PHYSIQUE;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// The axis-aligned box a place occupies, in its own local millimetres.
///
/// Invented by this spike because nothing in the contract layer describes the *shape* of a
/// place — `Location` says which place a thing is in and where inside it, and stops. A client
/// that must build a floor and four walls has to get the extent from somewhere; getting it from
/// a component is the only answer that keeps the rule out of the client (see `FINDINGS.md` F3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceExtent {
    pub min: LocalPosition,
    pub max: LocalPosition,
}

impl Component for PlaceExtent {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("place-extent");
    const OWNER: SystemId = GEOGRAPHY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

/// A sign hanging in a place — and the spike's deliberate trap for the `DD-15` wire encoding.
///
/// `catalogue_id` is above 2^53 and is **not** an entity id. A wire encoder that stringifies
/// every field called `id`, or every large integer, corrupts it. The encoder in `wire.rs`
/// therefore refuses to descend into any component payload, and `FINDINGS.md` F2 records why
/// that constraint is forced rather than chosen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signage {
    pub text: String,
    pub catalogue_id: u64,
    pub id: u64,
}

impl Component for Signage {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("signage");
    const OWNER: SystemId = GEOGRAPHY;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

// -------------------------------------------------------------------------------------------
// Actions — what a client may ask for.
// -------------------------------------------------------------------------------------------

/// Start talking to somebody. The action both clients must produce identically (`AC-13`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Talk {
    pub topic: String,
}

impl Action for Talk {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("talk");
    const OWNER: SystemId = CONVERSATION;
}

/// Ask to stand somewhere, facing somewhere.
///
/// One action for both clients: the 2D client asks for the point it clicked, the 3D client asks
/// for the point its own physics body reached. The server walks the authoritative body toward
/// the goal at its own speed; neither client moves the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveTo {
    pub to: LocalPosition,
    pub facing: Option<Orientation>,
}

impl Action for MoveTo {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("move-to");
    const OWNER: SystemId = MOVEMENT;
}

/// Pick an item up. Present so the observation carries more than one kind of affordance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickUp {}

impl Action for PickUp {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("pick-up");
    const OWNER: SystemId = INVENTORY;
}

// -------------------------------------------------------------------------------------------
// Events — what the world records.
// -------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationStarted {
    pub greeting: String,
}

impl Event for ConversationStarted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("conversation-started");
    const OWNER: SystemId = CONVERSATION;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Moved {
    pub to: LocalPosition,
}

impl Event for Moved {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("moved");
    const OWNER: SystemId = MOVEMENT;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}
