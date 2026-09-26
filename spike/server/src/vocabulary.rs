//! The System Pack vocabulary this spike invents, so the server has something to say.
//!
//! Every type here plays the part a real System Pack would play. None of it belongs to the
//! contract layer: `talk`, `move-to`, `display-name` and `place-extent` are exactly the domain
//! concepts `contracts/src/lib.rs` rule 1 forbids in `mineworld-contracts`. They are declared
//! here to prove the contract layer can carry them without knowing them.

use mineworld_contracts::{
    Action, ActionTypeId, Component, ComponentSchemaVersion, ComponentTypeId, EntityId, Event,
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

/// A sign hanging in a place — and the spike's deliberate trap, kept because it still says
/// something true.
///
/// `catalogue_id` and `id` are above 2^53 and **neither is an entity id**. That is why a wire
/// encoder could not fix the payload problem by guessing: one that stringified every field called
/// `id`, or every large integer, would corrupt these two. It also shows what the contract's fix
/// does *not* claim. `mineworld-contracts` now protects ids because the rule is on the
/// [`EntityId`](mineworld_contracts::EntityId) type, so these plain `u64`s are still plain JSON
/// numbers and a Godot client still reads them imprecisely — correctly, because they are not
/// identities. Compare [`Ownership`], one field away, whose `owner` *is* an `EntityId` and is
/// protected. `FINDINGS.md` F2 records why keying on the type was the only available rule.
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

/// Who owns a thing — the third payload shape `FINDINGS.md` F2 names, and the one that makes the
/// fix observable from inside a client.
///
/// `owner` is an `EntityId` above 2^53 sitting inside a component payload: exactly the position a
/// protocol-level encoder is forbidden to look into. It reaches a client as `"9007199254740995"`
/// rather than as a number, without this server doing anything, because the rule travels with the
/// type. Next to [`Signage`]'s plain `u64`s in the same kind of payload, it is the whole argument
/// in two components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ownership {
    pub owner: EntityId,
}

impl Component for Ownership {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("ownership");
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
