//! What this pack puts in an observation: the actions it offers, and the state it lets be known.
//!
//! ```text
//! offers      against nobody   standing: one complete sit { point } per sit point of the observer's
//!                              place, in key order, refused for the reason dispatch would give;
//!                              seated: one complete stand {}
//! discloses   about a place    its SitPoints, to whoever perceives the place
//!             about a person   their posture-state, to whoever perceives them
//! ```
//!
//! A client never computes reach, occupancy or stillness: the verdict is the server's (`I-S34-4`).

use mineworld_contracts::{ComponentRecord, EntityId};
use mineworld_kernel::WorldRead;
use mineworld_presence::{Offer, PerceptionProvider};
use serde_json::Value;

use crate::action::{Sit, Stand, sit_requirement, stand_requirement};
use crate::codec;
use crate::component::Posture;
use crate::point::SitPoints;
use crate::system::{PostureSystem, located, person, seated, sit_verdict};

impl PerceptionProvider for PostureSystem {
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        if target.is_some() {
            return Vec::new();
        }
        let (Some(me), Some(here)) = (person(world, observer), located(world, observer)) else {
            return Vec::new();
        };
        if seated(world, me).is_some() {
            return vec![
                Offer::complete(&Stand {}, stand_requirement()).expect("an empty payload encodes"),
            ];
        }
        let place = here.place();
        let Some(points) = world.component::<SitPoints>(place.entity_id()) else {
            return Vec::new();
        };
        points
            .iter()
            .map(|point| {
                let offer = Offer::complete(&Sit::new(point.key().clone()), sit_requirement(place))
                    .expect("a sit payload encodes");
                match sit_verdict(world, me, &here, point.key()) {
                    Ok(_) => offer,
                    Err(reason) => offer.refused(reason),
                }
            })
            .collect()
    }

    fn discloses(
        &self,
        world: &WorldRead<'_>,
        _observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let mut records = Vec::new();
        if let Some(points) = world.component::<SitPoints>(subject) {
            records.push(ComponentRecord::new::<SitPoints>(
                subject,
                codec::to_value(points),
            ));
        }
        if let Some(row) = world.component::<Posture>(subject) {
            records.push(ComponentRecord::new::<Posture>(
                subject,
                codec::to_value(row),
            ));
        }
        records
    }
}
