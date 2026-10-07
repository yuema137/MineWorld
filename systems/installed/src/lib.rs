//! The System Packs this build provides — the build's **installed set**, and its only list of packs
//! (`docs/DECISIONS.md` `ARC-33`, `docs/MODULE_SPEC.md` §3.1).
//!
//! Installing a pack is one line in this crate's `Cargo.toml` and one line below, then a rebuild.
//! Nothing else in the build learns the pack: the World Pack loader reads this list through the
//! [`Capability`] it expands to, and the pack says everything else about itself in its own
//! `impl SystemPack`. Installing puts a pack into the build; a world enables it by naming it in its
//! `systems:` list.
//!
//! The order below is the order a refusal lists the available systems in. It is not installation
//! order — a world's `systems:` list is.
//!
//! # Why this crate lives under `systems/`
//!
//! It is the list of System Packs, so installing one is an edit under `systems/` and nowhere else,
//! which is what `ARC-35` measures. Installing without a rebuild is `ARC-8`'s WASM component model,
//! outside MVP-0.
//!
//! A test (`tests/installed.rs`) holds this list and the manifest's dependencies equal, and refuses
//! two packs with one id.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mineworld_sdk::installed! {
    perception: mineworld_presence::PerceptionProvider;
    Presence => mineworld_presence::PresenceSystem,
    Movement => mineworld_movement::MovementSystem,
    Conversation => mineworld_conversation::ConversationSystem,
    GroupActivity => mineworld_group_activity::GroupActivitySystem,
    Relationships => mineworld_relationships::RelationshipsSystem,
    Naming => mineworld_naming::NamingSystem,
    Schedule => mineworld_schedule::ScheduleSystem,
    Item => mineworld_item::ItemSystem,
    Inventory => mineworld_inventory::InventorySystem,
    ItemTransfer => mineworld_item_transfer::ItemTransferSystem,
    Economy => mineworld_economy::EconomySystem,
    Employment => mineworld_employment::EmploymentSystem,
    Consumption => mineworld_consumption::ConsumptionSystem,
}
