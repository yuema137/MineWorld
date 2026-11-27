//! Door names: the name over each doorway of the place an observer stands in (`ARC-363`).
//!
//! A place's tags reach an observer by two routes: the place's own record, to whoever stands in it,
//! and the destination's tags on a doorway (`to_tags`, `ARC-82`), to whoever perceives the doorway's
//! place. A place's name reaches the same observers by the same two routes. The first needs nothing
//! here: [`DisplayName`] is disclosed about the place one perceives. The second is this record.
//!
//! Movement builds `to_tags` from kernel tags. A name is this pack's state, so movement cannot add it
//! without depending on naming; instead this pack reads movement's public [`Passages`] and discloses
//! the destinations' names itself, so the optional pack depends on the fundamental one.

use mineworld_contracts::PlaceId;
use mineworld_kernel::{WorldRead, owned_component};
use mineworld_movement::Passages;
use serde::{Deserialize, Serialize};

use crate::component::DisplayName;
use crate::name::{Forms, Name};
use crate::system::NamingSystem;

/// The name of the place one doorway leads to: the destination's name and nothing else, never who
/// or what is inside it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoorName {
    /// Where the doorway leads.
    pub to: PlaceId,
    /// The destination's default name.
    pub name: Name,
    /// The destination's name in other languages; left out when empty.
    #[serde(default, skip_serializing_if = "Forms::is_empty")]
    pub forms: Forms,
}

/// `door-names`: one entry per doorway of a place whose destination has a [`DisplayName`], in
/// [`Passages`] order. Disclosed about the place to whoever perceives it — the observers who receive
/// its `passages` — and never stored.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoorNames {
    /// The named doorways.
    pub leads_to: Vec<DoorName>,
}

owned_component! {
    component = DoorNames,
    owner = NamingSystem,
    component_type = "door-names",
    schema_version = 1,
}

impl DoorNames {
    /// The door names of a place with `passages`, read against the world as it is now; `None` when no
    /// destination is named — absence rather than an empty record.
    pub(crate) fn of(world: &WorldRead<'_>, passages: &Passages) -> Option<Self> {
        let leads_to: Vec<DoorName> = passages
            .iter()
            .filter_map(|passage| {
                let to = passage.to();
                world
                    .component::<DisplayName>(to.entity_id())
                    .map(|named| DoorName {
                        to,
                        name: named.name().clone(),
                        forms: named.forms().clone(),
                    })
            })
            .collect();
        (!leads_to.is_empty()).then_some(Self { leads_to })
    }
}
