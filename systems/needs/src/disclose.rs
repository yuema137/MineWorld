//! The `needs` record: what the holder alone is told about their own needs (SD-Nb-6, `INV-13`; schema 2,
//! `ARC-370`).
//!
//! A disclosed view of owned state, never the stored [`Needs`] raw: the current levels at the
//! observation's instant, the pack's words for them, whether the person sleeps (as sleep's facts said),
//! the rates and restorers in force, and how much each kind the person carries would nourish them —
//! so a controller, a prompt and a HUD read the same numbers and words and never read an item file or
//! hard-code a rate (F-Nb-4). The thresholds are not disclosed: the words are.
//!
//! In the food-energy model `hunger` is the normalized deficit of the reserve, each kind's `hunger` its
//! motivation score, and `food_energy` carries the kcal a reader shows — computed here, never by a
//! reader.

use mineworld_contracts::{ComponentRecord, EntityId, EntityType, ItemId};
use mineworld_inventory::Holdings;
use mineworld_kernel::WorldRead;
use serde::Serialize;
use serde_json::Value;

use crate::codec;
use crate::component::{Hunger, Needs, Nourishment};
use crate::food_energy::{BAR_MAXIMUM, Reading, Serving, UNIT};
use crate::interactions::{HungerModel, NeedsParameters, Restore, parameters, rates};
use crate::level::{HungerWord, Mode, TirednessWord};

/// The need a food-energy reserve stands for, by the identifier readers already know.
const RESERVE_REPLACES: &str = "hunger";

/// The disclosed record.
#[derive(Debug, Serialize)]
struct Disclosed {
    hunger_model: HungerModel,
    hunger: u16,
    tiredness: u16,
    words: Words,
    asleep: bool,
    rates: DisclosedRates,
    restore: Restore,
    nourishment: Vec<Nourishing>,
    /// How much one of every food kind of this world would nourish me, in `ItemId` order: what a
    /// person knows about food they do not carry, so a controller can tell an apple from a coffee
    /// before buying it and a shop worth walking to from one that is not (S35 N-h R3, F-Nh-2).
    kinds: Vec<Nourishing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    food_energy: Option<FoodEnergy>,
}

#[derive(Debug, Serialize)]
struct Words {
    hunger: HungerWord,
    tiredness: TirednessWord,
}

/// The rates in force; hunger's only in the unitless model.
#[derive(Debug, Serialize)]
struct DisclosedRates {
    #[serde(skip_serializing_if = "Option::is_none")]
    hunger_per_hour: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hunger_asleep_per_hour: Option<u16>,
    tiredness_per_hour: u16,
    rest_per_hour: u16,
}

#[derive(Debug, Serialize)]
struct Nourishing {
    item: ItemId,
    /// The hunger one lowers, or a food-energy kind's motivation score.
    hunger: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    energy_kcal: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    serving: Option<Serving>,
}

#[derive(Debug, Serialize)]
struct FoodEnergy {
    unit: &'static str,
    available_kcal: u32,
    capacity_kcal: u32,
    use_kcal_per_world_day: u32,
    bar: Bar,
    replaces_need: &'static str,
}

#[derive(Debug, Serialize)]
struct Bar {
    value: u16,
    maximum: u16,
}

/// `subject`'s `needs` record for `observer`: only when they are the same person and that person has
/// needs (and, for a reserve, the instant is not before it was stored — never the case in a world
/// whose time only moves forward).
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
    let daily = found.energy_kcal_per_day;
    // `None` only for an instant before the stored reserve, which a world whose time only moves forward
    // never asks for: withheld, never shown as a saturated value.
    let Some((hunger, tiredness)) = needs.current(world.at(), &rates(&found), daily) else {
        return Vec::new();
    };
    let reading = match needs.hunger() {
        Hunger::FoodEnergy(reserve) => reserve.reading(world.at(), daily),
        Hunger::Unitless(_) => None,
    };
    let record = view(world, subject, needs, (hunger, tiredness), reading, &found);
    vec![ComponentRecord::new::<Needs>(
        subject,
        codec::to_value(&record),
    )]
}

fn nourishing(item: ItemId, nourishment: &Nourishment, capacity: u32) -> Nourishing {
    let (energy_kcal, serving) = match nourishment {
        Nourishment::FoodEnergy {
            energy_kcal,
            serving,
        } => (Some(*energy_kcal), Some(serving.clone())),
        Nourishment::Unitless { .. } => (None, None),
    };
    Nourishing {
        item,
        hunger: nourishment.motivation(capacity),
        energy_kcal,
        serving,
    }
}

fn view(
    world: &WorldRead<'_>,
    subject: EntityId,
    needs: &Needs,
    (hunger, tiredness): (u16, u16),
    reading: Option<Reading>,
    found: &NeedsParameters,
) -> Disclosed {
    let thresholds = found.thresholds.0;
    let capacity = found.energy_kcal_per_day;
    let unitless = matches!(needs.hunger(), Hunger::Unitless(_));
    let nourishment = world
        .component::<Holdings>(subject)
        .map(|holdings| {
            holdings
                .held()
                .iter()
                .filter_map(|held| {
                    world
                        .component::<Nourishment>(held.item().entity_id())
                        .map(|nourishment| nourishing(held.item(), nourishment, capacity))
                })
                .collect()
        })
        .unwrap_or_default();
    Disclosed {
        hunger_model: if unitless {
            HungerModel::Unitless
        } else {
            HungerModel::FoodEnergy
        },
        hunger,
        tiredness,
        words: Words {
            hunger: HungerWord::of(hunger, thresholds),
            tiredness: TirednessWord::of(tiredness, thresholds),
        },
        asleep: needs.mode() == Mode::Asleep,
        rates: DisclosedRates {
            hunger_per_hour: unitless.then_some(found.hunger_per_hour),
            hunger_asleep_per_hour: unitless.then_some(found.hunger_asleep_per_hour),
            tiredness_per_hour: found.tiredness_per_hour,
            rest_per_hour: found.rest_per_hour,
        },
        restore: found.restore.clone(),
        nourishment,
        kinds: world
            .components::<Nourishment>()
            .filter_map(|(entity, nourishment)| {
                let item = ItemId::new(entity, EntityType::Item).ok()?;
                Some(nourishing(item, nourishment, capacity))
            })
            .collect(),
        food_energy: reading.map(|reading| FoodEnergy {
            unit: UNIT,
            available_kcal: reading.available_kcal,
            capacity_kcal: reading.capacity_kcal,
            use_kcal_per_world_day: capacity,
            bar: Bar {
                value: reading.bar,
                maximum: BAR_MAXIMUM,
            },
            replaces_need: RESERVE_REPLACES,
        }),
    }
}
