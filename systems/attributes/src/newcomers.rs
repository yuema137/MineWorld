//! Admission is resolved by the attributes owner in the requesting dispatch.

use std::collections::{BTreeMap, BTreeSet};

use mineworld_attributes_model::{
    ATTRIBUTES, AttributesHanded, NewcomerAdmitted, NewcomersProcess, TraitsStated,
};
use mineworld_contracts::{EventEnvelope, SystemId, Visibility};
use mineworld_kernel::{Emission, KernelError, WorldView};

use crate::system::{AttributesSystem, encode};

pub(crate) fn admit(
    world: &mut WorldView<'_, AttributesSystem>,
    event: &EventEnvelope,
    admitted: NewcomerAdmitted,
) -> Result<Vec<Emission>, KernelError> {
    if admitted.because >= event.id() {
        return Err(crate::system::refused(
            event.event_type().clone(),
            "unknown-cause",
            Some(format!(
                "{:?} cites a cause that is not earlier",
                admitted.person
            )),
        ));
    }
    let mut drawn = mineworld_attributes_model::newcomer::draw(
        &world.read(),
        event.provenance().emitted_by(),
        admitted.person,
        admitted.because,
    )
    .map_err(|reason| KernelError::FactRefusedByOwner {
        system: ATTRIBUTES,
        event_type: event.event_type().clone(),
        reason,
    })?;
    let running = mineworld_attributes_model::RunningCatalogue::in_world(&world.read())
        .ok_or_else(|| {
            crate::system::refused(event.event_type().clone(), "attributes-uncomposed", None)
        })?;
    let mut plain = Vec::new();
    let mut owned: BTreeMap<SystemId, Vec<_>> = BTreeMap::new();
    for (id, (value, _)) in drawn.arrival.values {
        let declaration = running.catalogue().get(&id).ok_or_else(|| {
            crate::system::refused(
                event.event_type().clone(),
                "unknown-attribute",
                Some(id.to_string()),
            )
        })?;
        if declaration.owner == ATTRIBUTES {
            plain.push((id, value));
        } else {
            owned
                .entry(declaration.owner.clone())
                .or_default()
                .push((id, value));
        }
    }
    // Update before returning emissions: another request in this generation must see the admission.
    drawn.process.1.arrived.insert(drawn.key);
    world.set_process_state::<NewcomersProcess<AttributesSystem>>(
        drawn.process.0,
        encode(&drawn.process.1),
    )?;
    let person = admitted.person;
    let subject = person.entity_id();
    let mut facts = vec![
        Emission::new::<TraitsStated>(
            encode(&TraitsStated {
                person,
                values: plain,
            }),
            Visibility::Entities(BTreeSet::from([subject])),
        )
        .about(vec![subject]),
    ];
    for (owner, values) in owned {
        facts.push(
            Emission::new::<AttributesHanded>(
                encode(&AttributesHanded {
                    person,
                    owner,
                    values,
                }),
                Visibility::SystemInternal,
            )
            .about(vec![subject]),
        );
    }
    Ok(facts)
}
