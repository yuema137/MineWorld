//! The `needs` record: what the holder alone is told about their own needs (SD-Nb-6, `INV-13`).
//!
//! A disclosed view of owned state, never the stored [`Needs`] raw: the current levels at the
//! observation's instant, the pack's words for them, whether the person sleeps (as sleep's facts said),
//! the rates and restorers in force, and how much each kind the person carries would nourish them —
//! so a controller, a prompt and a HUD read the same numbers and words and never read an item file or
//! hard-code a rate (F-Nb-4). The thresholds are not disclosed: the words are.

use mineworld_contracts::{ComponentRecord, EntityId, ItemId};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use serde::Serialize;
use serde_json::Value;

use crate::codec;
use crate::component::{Needs, Nourishment};
use crate::interactions::{NeedsParameters, Restore, parameters, rates};
use crate::level::{HungerWord, Mode, TirednessWord};

/// The disclosed record.
#[derive(Debug, Serialize)]
struct Disclosed {
    hunger: u16,
    tiredness: u16,
    words: Words,
    asleep: bool,
    rates: DisclosedRates,
    restore: Restore,
    nourishment: Vec<Nourishing>,
}

#[derive(Debug, Serialize)]
struct Words {
    hunger: HungerWord,
    tiredness: TirednessWord,
}

#[derive(Debug, Serialize)]
struct DisclosedRates {
    hunger_per_hour: u16,
    hunger_asleep_per_hour: u16,
    tiredness_per_hour: u16,
    rest_per_hour: u16,
}

#[derive(Debug, Serialize)]
struct Nourishing {
    item: ItemId,
    hunger: u16,
}

/// `subject`'s `needs` record for `observer`: only when they are the same person and that person has
/// needs.
pub(crate) fn disclosed(
    world: &WorldRead<'_>,
    observer: EntityId,
    subject: EntityId,
) -> Vec<ComponentRecord<Value>> {
    if observer != subject {
        return Vec::new();
    }
    let Some(needs) = world.component::<Needs>(subject) else {
        return Vec::new();
    };
    let found = parameters(world, subject);
    let (hunger, tiredness) = needs.current(world.at(), &rates(&found));
    let record = view(world, subject, needs, hunger, tiredness, found);
    vec![ComponentRecord::new::<Needs>(
        subject,
        codec::to_value(&record),
    )]
}

fn view(
    world: &WorldRead<'_>,
    subject: EntityId,
    needs: &Needs,
    hunger: u16,
    tiredness: u16,
    found: NeedsParameters,
) -> Disclosed {
    let thresholds = found.thresholds.0;
    let nourishment = world
        .component::<Holdings>(subject)
        .map(|holdings| {
            holdings
                .held()
                .iter()
                .filter_map(|held| {
                    world
                        .component::<Nourishment>(held.item().entity_id())
                        .map(|nourishment| Nourishing {
                            item: held.item(),
                            hunger: nourishment.hunger(),
                        })
                })
                .collect()
        })
        .unwrap_or_default();
    Disclosed {
        hunger,
        tiredness,
        words: Words {
            hunger: HungerWord::of(hunger, thresholds),
            tiredness: TirednessWord::of(tiredness, thresholds),
        },
        asleep: needs.mode() == Mode::Asleep,
        rates: DisclosedRates {
            hunger_per_hour: found.hunger_per_hour,
            hunger_asleep_per_hour: found.hunger_asleep_per_hour,
            tiredness_per_hour: found.tiredness_per_hour,
            rest_per_hour: found.rest_per_hour,
        },
        restore: found.restore,
        nourishment,
    }
}
