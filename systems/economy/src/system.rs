//! The installable system: what it declares, how it decides a purchase, how it answers a wage, the
//! facts it reduces, and what it offers and discloses.

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Reference, Seeding};
use mineworld_contracts::{
    ActionIntent, ComponentRecord, EntityId, EntityKey, EntityType, Event, EventEnvelope,
    EventTypeId, LifecycleState, PlaceId, Rejection, RejectionCode, SystemId, Visibility,
};
<<<<<<< HEAD
use mineworld_employment::{WageDeductionDue, WageDue};
=======
use mineworld_employment::WageDue;
use mineworld_happenings::{HappeningBegan, HappeningEnded, HappeningRef, HappeningsSystem};
>>>>>>> origin/mvp0/pr-le
use mineworld_inventory::{InventorySystem, ItemsTransferred};
use mineworld_kernel::{
    Declarations, Emission, KernelError, ProcessKind, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, Presence, PresenceSystem};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::action::{Buy, buy_requirement, purchasable};
use crate::book::{PriceBookProcess, PriceBookState};
use crate::codec::{self, read_buy};
use crate::component::{Shop, Wallet};
<<<<<<< HEAD
use crate::event::{DeductionUnpaid, Funded, MoneyTransferred, ShopOpened, WageUnpaid};
=======
use crate::configuration::EconomyConfiguration;
use crate::event::{
    EconomyConfigured, Funded, MoneyTransferred, PricesAdjusted, PricesRestored, ShopOpened,
    WageUnpaid,
};
>>>>>>> origin/mvp0/pr-le
use crate::money::{admit_payment, balance, is_holder, pay};
use crate::offer::{buys, listing, living_person, shop_here};

/// Money: wallets, shops, buying, and wages.
#[derive(Default)]
pub struct EconomySystem;

impl SystemIdentity for EconomySystem {
    const ID: SystemId = SystemId::from_static("economy");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing
/// biographical — purchases and wages are thousands of facts that would bury a biography (step-10
/// QS-49) — the `economy:` section, which its `AuthoredSection` impl (`src/section.rs`) describes,
/// and the optional `configure/economy.yaml` (`src/configuration.rs`, `ARC-61`).
impl SystemPack for EconomySystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
    mineworld_sdk::configures!();
}

impl PackConfiguration for EconomySystem {
    type Configuration = EconomyConfiguration;
    const FACTS: &'static [EventTypeId] = &[EconomyConfigured::EVENT_TYPE];

    /// Each modifier's shop, which must be a Place.
    fn references(configuration: &EconomyConfiguration) -> Vec<Reference<'_>> {
        configuration
            .price_modifiers()
            .iter()
            .map(|modifier| Reference {
                key: modifier.shop(),
                entity_type: EntityType::Place,
            })
            .collect()
    }

    /// `happenings`, whenever a modifier names a happening kind (INV-WE-4): removing happenings and
    /// leaving the modifiers behind is refused by name, never silently inert.
    fn requires(configuration: &EconomyConfiguration) -> Vec<SystemId> {
        if configuration.price_modifiers().is_empty() {
            Vec::new()
        } else {
            vec![HappeningsSystem::ID]
        }
    }

    /// One `economy-configured`, SystemInternal, about nobody (`ARC-61` item 8), each shop resolved.
    fn seed(
        seeding: &Seeding<'_, '_>,
        configuration: &EconomyConfiguration,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        let resolve = |key: &EntityKey| {
            seeding
                .resolve(key, EntityType::Place)
                .and_then(|entity| PlaceId::new(entity, EntityType::Place).ok())
                .ok_or_else(|| Rejection::System {
                    code: RejectionCode::from_static("economy-place-unknown"),
                    detail: Some(format!("'{}' is no place of this world", key.as_str())),
                })
        };
        let modifiers = configuration
            .price_modifiers()
            .iter()
            .map(|modifier| Ok(modifier.at(resolve(modifier.shop())?)))
            .collect::<Result<Vec<_>, Rejection>>()?;
        Ok(vec![Emission::new::<EconomyConfigured>(
            codec::encode(&EconomyConfigured::new(modifiers)),
            Visibility::SystemInternal,
        )])
    }
}

/// A fact this pack may not take, reached at reduction or at resolve: a defect in whoever stated it
/// past the rule, reported as the owner's refusal (`ARC-26`).
fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: EconomySystem::ID,
        event_type,
        reason,
    }
}

impl System for EconomySystem {
<<<<<<< HEAD
    /// 2 since PR LE-b (`ARC-287`): it answers `wage-deduction-due` and states `deduction-unpaid`.
=======
    /// 2 since WE-a: the declaration grew a configuration, two facts and two subscriptions, so a save
    /// written by version 1 is refused by name (`ARC-25`).
>>>>>>> origin/mvp0/pr-le
    const VERSION: SystemVersion = SystemVersion::new(2);

    /// Depends on inventory, whose `items-transferred` a purchase states and whose stock a listing
    /// reads (`ARC-26`), and on presence, whose positions it reads. Subscribes to employment's
    /// `wage-due` with **no** dependency on employment (`ARC-28`): a world without employment installs
    /// economy and sells, and simply hears no wages. Subscribes to happenings' `happening-began` and
    /// `happening-ended` the same way: a world without happenings hears none, and one whose economy is
    /// not configured hears them without answering.
    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([InventorySystem::ID, PresenceSystem::ID])
            .owning::<Wallet>()
            .owning::<Shop>()
            .providing::<Buy>()
            .emitting::<Funded>()
            .emitting::<ShopOpened>()
            .emitting::<MoneyTransferred>()
            .emitting::<WageUnpaid>()
            .emitting::<DeductionUnpaid>()
            .emitting::<ItemsTransferred>()
            .emitting::<EconomyConfigured>()
            .emitting::<PricesAdjusted>()
            .emitting::<PricesRestored>()
            .subscribing_to::<Funded>()
            .subscribing_to::<ShopOpened>()
            .subscribing_to::<MoneyTransferred>()
            .subscribing_to::<WageDue>()
<<<<<<< HEAD
            .subscribing_to::<WageDeductionDue>()
=======
            .subscribing_to::<EconomyConfigured>()
            .subscribing_to::<HappeningBegan>()
            .subscribing_to::<HappeningEnded>()
            .subscribing_to::<PricesAdjusted>()
            .subscribing_to::<PricesRestored>()
>>>>>>> origin/mvp0/pr-le
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Wallet>()?;
        tables.component::<Shop>()
    }

    /// Whether this buy may happen, in order:
    ///
    /// 1. the payload is a `buy`;
    /// 2. the buyer is a living Person, with no target — `NoSupportedInteraction` otherwise;
    /// 3. the buyer stands in a shop that prices the kind — `NoSupportedInteraction` otherwise;
    /// 4. the requirement, with availability from [`purchasable`]: in stock, affordable, carriable
    ///    (`TargetUnavailable`).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let buy = read_buy(intent)?;
        let buyer = intent.actor();
        if !living_person(world, buyer) || intent.target().is_some() {
            return Err(Rejection::NoSupportedInteraction);
        }
        let here = world
            .component::<Presence>(buyer)
            .map(Presence::location)
            .ok_or(Rejection::PreconditionFailed)?;
        let shop = world
            .component::<Shop>(here.place().entity_id())
            .ok_or(Rejection::NoSupportedInteraction)?;
        if shop.price(buy.item()).is_none() {
            return Err(Rejection::NoSupportedInteraction);
        }
        buy_requirement(here.place()).evaluate(
            &here,
            None,
            purchasable(world, buyer, shop, buy.item()),
        )
    }

    /// States the price paid, buyer → operator, heard in the shop (step-10 QS-45), and inventory's
    /// `items-transferred`, operator → buyer, through its checked constructor — and nothing else.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let money = |reason| refused(MoneyTransferred::EVENT_TYPE, reason);
        let buy = read_buy(intent).map_err(money)?;
        let read = world.read();
        let buyer = intent.actor();
        let (place, shop) =
            shop_here(&read, buyer).ok_or_else(|| money(Rejection::PreconditionFailed))?;
        let price = shop
            .effective_price(buy.item())
            .ok_or_else(|| money(Rejection::PreconditionFailed))?;
        let operator = shop.operator().entity_id();
        let paid = pay(&read, buyer, operator, price, Visibility::Place(place)).map_err(money)?;
        let goods = mineworld_inventory::transfer(&read, operator, buyer, buy.item(), 1).map_err(
            |reason| KernelError::FactRefusedByOwner {
                system: InventorySystem::ID,
                event_type: ItemsTransferred::EVENT_TYPE,
                reason,
            },
        )?;
        Ok(vec![paid, goods])
    }

    /// `funded`, `shop-opened` and `money-transferred` → [`Wallet`] and [`Shop`], the only writes of
    /// either; employment's `wage-due` → a payment or `wage-unpaid`, stated here and caused by it.
    ///
    /// **The owner still decides** (`ARC-26`): a payment is put to [`admit_payment`] before anything is
    /// written; a holder is funded once and a place has one shop. A refusal writes nothing and fails
    /// with [`KernelError::FactRefusedByOwner`].
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let kind = event.event_type();
        if *kind == Funded::EVENT_TYPE {
            let funded: Funded = codec::event_payload(event.payload())?;
            let read = world.read();
            if !is_holder(&read, funded.holder())
                || read.component::<Wallet>(funded.holder()).is_some()
            {
                return Err(refused(Funded::EVENT_TYPE, Rejection::PreconditionFailed));
            }
            world.insert(funded.holder(), Wallet::new(funded.balance()))?;
        } else if *kind == ShopOpened::EVENT_TYPE {
            let opened: ShopOpened = codec::event_payload(event.payload())?;
            let read = world.read();
            let place = opened.place().entity_id();
            let is_place = read.entity(place).is_some_and(|record| {
                record.entity_type() == EntityType::Place
                    && record.lifecycle() != LifecycleState::Destroyed
            });
            if !is_place
                || !is_holder(&read, opened.operator().entity_id())
                || read.component::<Shop>(place).is_some()
            {
                return Err(refused(
                    ShopOpened::EVENT_TYPE,
                    Rejection::PreconditionFailed,
                ));
            }
            world.insert(
                place,
                Shop::new(opened.operator(), opened.prices().to_vec()),
            )?;
        } else if *kind == MoneyTransferred::EVENT_TYPE {
            let moved: MoneyTransferred = codec::event_payload(event.payload())?;
            let read = world.read();
            admit_payment(&read, moved.from(), moved.to(), moved.amount())
                .map_err(|reason| refused(MoneyTransferred::EVENT_TYPE, reason))?;
            let overflow = || refused(MoneyTransferred::EVENT_TYPE, Rejection::PreconditionFailed);
            let from = balance(&read, moved.from()) - moved.amount();
            let to = balance(&read, moved.to())
                .checked_add(moved.amount())
                .ok_or_else(overflow)?;
            world.insert(moved.from(), Wallet::new(from))?;
            world.insert(moved.to(), Wallet::new(to))?;
        } else if *kind == WageDue::EVENT_TYPE {
            let due: WageDue = codec::event_payload(event.payload())?;
            return Ok(vec![answer(&world.read(), &due)]);
<<<<<<< HEAD
        } else if *kind == WageDeductionDue::EVENT_TYPE {
            let due: WageDeductionDue = codec::event_payload(event.payload())?;
            return Ok(vec![deduct(&world.read(), &due)]);
=======
        } else {
            return react_to_prices(world, event);
>>>>>>> origin/mvp0/pr-le
        }
        Ok(Vec::new())
    }
}

/// The price-modifier half of [`EconomySystem::react`] (WE-a design §6.2): `economy-configured` → the
/// price book; a configured `happening-began` → `prices-adjusted` for each modified shop, and
/// `happening-ended` → `prices-restored`, each caused by the happening fact; `prices-adjusted` and
/// `prices-restored` → the [`Shop`], the only writes of its adjustments.
fn react_to_prices(
    world: &mut WorldView<'_, EconomySystem>,
    event: &EventEnvelope,
) -> Result<Vec<Emission>, KernelError> {
    let kind = event.event_type();
    if *kind == EconomyConfigured::EVENT_TYPE {
        let configured: EconomyConfigured = codec::event_payload(event.payload())?;
        if price_book(&world.read())?.is_some() {
            return Err(refused(
                EconomyConfigured::EVENT_TYPE,
                Rejection::PreconditionFailed,
            ));
        }
        let state = PriceBookState::new(configured.price_modifiers().to_vec());
        world.start_process(
            ProcessStart::<PriceBookProcess>::new(codec::encode(&state)).uninterruptible(),
        )?;
    } else if *kind == HappeningBegan::EVENT_TYPE {
        let began: HappeningBegan = codec::event_payload(event.payload())?;
        return adjustments(&world.read(), began.happening());
    } else if *kind == HappeningEnded::EVENT_TYPE {
        let ended: HappeningEnded = codec::event_payload(event.payload())?;
        return restorations(&world.read(), ended.happening());
    } else if *kind == PricesAdjusted::EVENT_TYPE {
        let adjusted: PricesAdjusted = codec::event_payload(event.payload())?;
        let place = adjusted.place().entity_id();
        let shop = world
            .read()
            .component::<Shop>(place)
            .cloned()
            .ok_or_else(|| refused(PricesAdjusted::EVENT_TYPE, Rejection::PreconditionFailed))?;
        world.insert(
            place,
            shop.adjusted(adjusted.happening().clone(), adjusted.permille()),
        )?;
    } else if *kind == PricesRestored::EVENT_TYPE {
        let restored: PricesRestored = codec::event_payload(event.payload())?;
        let place = restored.place().entity_id();
        let shop = world
            .read()
            .component::<Shop>(place)
            .cloned()
            .ok_or_else(|| refused(PricesRestored::EVENT_TYPE, Rejection::PreconditionFailed))?;
        world.insert(place, shop.restored(restored.happening()))?;
    }
    Ok(Vec::new())
}

/// The world's price book, if its economy is configured.
fn price_book(world: &WorldRead<'_>) -> Result<Option<PriceBookState>, KernelError> {
    let Some(process) = world.processes().find(|process| {
        *process.owner() == EconomySystem::ID
            && *process.process_type() == PriceBookProcess::PROCESS_TYPE
    }) else {
        return Ok(None);
    };
    serde_json::from_slice(process.state_for::<PriceBookProcess>()?)
        .map(Some)
        .map_err(|_| KernelError::ProcessNotRunning {
            process: process.id(),
        })
}

/// A price fact at the shop's place, public, about the place.
fn at_shop<E: Event>(fact: &impl serde::Serialize, place: PlaceId) -> Emission {
    Emission::new::<E>(codec::encode(fact), Visibility::Public)
        .about(vec![place.entity_id()])
        .at_place(place)
}

/// `prices-adjusted` for each configured shop whose modifier names `happening`'s kind and where a shop
/// is open; none when the world's economy is not configured.
fn adjustments(
    world: &WorldRead<'_>,
    happening: &HappeningRef,
) -> Result<Vec<Emission>, KernelError> {
    let Some(book) = price_book(world)? else {
        return Ok(Vec::new());
    };
    Ok(book
        .price_modifiers()
        .iter()
        .filter(|modifier| modifier.during() == happening.kind())
        .filter(|modifier| world.has_component::<Shop>(modifier.shop().entity_id()))
        .map(|modifier| {
            at_shop::<PricesAdjusted>(
                &PricesAdjusted::new(
                    *modifier.shop(),
                    happening.clone(),
                    modifier.permille().get(),
                ),
                *modifier.shop(),
            )
        })
        .collect())
}

/// `prices-restored` for each shop `happening` adjusted.
fn restorations(
    world: &WorldRead<'_>,
    happening: &HappeningRef,
) -> Result<Vec<Emission>, KernelError> {
    let Some(book) = price_book(world)? else {
        return Ok(Vec::new());
    };
    Ok(book
        .price_modifiers()
        .iter()
        .filter(|modifier| modifier.during() == happening.kind())
        .filter(|modifier| {
            world
                .component::<Shop>(modifier.shop().entity_id())
                .is_some_and(|shop| shop.is_adjusted_by(happening))
        })
        .map(|modifier| {
            at_shop::<PricesRestored>(
                &PricesRestored::new(*modifier.shop(), happening.clone()),
                *modifier.shop(),
            )
        })
        .collect())
}

/// The answer to a `wage-due`: the payment, employer → employee, heard by the two of them, when the
/// employer can make it; otherwise `wage-unpaid`, and nothing moves.
fn answer(world: &WorldRead<'_>, due: &WageDue) -> Emission {
    let employee = due.employee().entity_id();
    let employer = due.employer().entity_id();
    if admit_payment(world, employer, employee, due.amount()).is_ok() {
        return pay(
            world,
            employer,
            employee,
            due.amount(),
            Visibility::Participants,
        )
        .expect("admitted a moment ago");
    }
    Emission::new::<WageUnpaid>(
        codec::encode(&WageUnpaid::new(
            due.employee(),
            due.employer(),
            due.amount(),
        )),
        Visibility::Participants,
    )
    .about(vec![employee, employer])
    .with_participants(vec![employee, employer])
}

/// The answer to a `wage-deduction-due` (`ARC-287`): the payment, employee → employer, heard by the two
/// of them, when the employee can make it; otherwise `deduction-unpaid`, and nothing moves.
fn deduct(world: &WorldRead<'_>, due: &WageDeductionDue) -> Emission {
    let employee = due.employee().entity_id();
    let employer = due.employer().entity_id();
    if admit_payment(world, employee, employer, due.amount()).is_ok() {
        return pay(
            world,
            employee,
            employer,
            due.amount(),
            Visibility::Participants,
        )
        .expect("admitted a moment ago");
    }
    Emission::new::<DeductionUnpaid>(
        codec::encode(&DeductionUnpaid::new(
            due.employee(),
            due.employer(),
            due.amount(),
        )),
        Visibility::Participants,
    )
    .about(vec![employee, employer])
    .with_participants(vec![employee, employer])
}

impl PerceptionProvider for EconomySystem {
    /// One complete buy per priced kind to a person standing in a shop (`src/offer.rs`).
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        buys(world, observer, target)
    }

    /// A [`Wallet`] to its holder alone (`INV-13`); a shop's place's listing to whoever perceives the
    /// place — the operator, and per priced kind its price and how many are in stock (step-10 QS-45).
    fn discloses(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        subject: EntityId,
    ) -> Vec<ComponentRecord<Value>> {
        let mut disclosed = Vec::new();
        if observer == subject
            && let Some(wallet) = world.component::<Wallet>(subject)
        {
            disclosed.push(ComponentRecord::new::<Wallet>(
                subject,
                codec::to_value(wallet),
            ));
        }
        if let Some(shop) = world.component::<Shop>(subject) {
            disclosed.push(ComponentRecord::new::<Shop>(
                subject,
                codec::to_value(&listing(world, shop)),
            ));
        }
        disclosed
    }
}
