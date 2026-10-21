//! The installable system: what it declares, how it decides a purchase, how it answers a wage, the
//! facts it reduces, and what it offers and discloses.

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Reference, Seeding};
use mineworld_contracts::{
    Action, ActionIntent, ComponentRecord, EntityId, EntityKey, EntityType, Event, EventEnvelope,
    EventTypeId, LifecycleState, PlaceId, Rejection, RejectionCode, SystemId, Visibility,
};
use mineworld_employment::{WageDeductionDue, WageDue};
use mineworld_happenings::{HappeningBegan, HappeningEnded, HappeningRef, HappeningsSystem};
use mineworld_inventory::{InventorySystem, ItemsTransferred};
use mineworld_kernel::{
    Declarations, Emission, KernelError, ProcessKind, ProcessStart, System, SystemDeclaration,
    SystemIdentity, SystemVersion, WorldRead, WorldView,
};
use mineworld_presence::{Offer, PerceptionProvider, PresenceSystem};
use mineworld_sdk::SystemPack;
use serde_json::Value;

use crate::action::{Buy, ListForSale, SellerRequest, SetPrice, Withdraw, seller_requirement};
use crate::book::{PriceBookProcess, PriceBookState};
use crate::code::{self, CURRENCY_UNDECLARED};
use crate::codec;
use crate::component::{Ask, Asks, ReferenceValue, Shop, ShopStock, Wallet, Wealth};
use crate::configuration::EconomyConfiguration;
use crate::currency::{self, CurrencyProcess, CurrencyRecord, currency_process};
use crate::event::{
    CurrencyDeclared, DeductionUnpaid, EconomyConfigured, Funded, Listed, MoneyTransferred,
    PriceSet, PricesAdjusted, PricesRestored, ReferenceDeclared, ShopOpened, Sold, WageUnpaid,
    Withdrawn,
};
use crate::listing::{self, judge_listing, seller_offers};
use crate::money::{admit_payment, balance, is_holder, pay, price_in_range};
use crate::offer::{asks_of, asks_view, listing, living_person, shop_stock, wealth};
use crate::sale;

/// Money: wallets, shops, buying, and wages.
#[derive(Default)]
pub struct EconomySystem;

impl SystemIdentity for EconomySystem {
    const ID: SystemId = SystemId::from_static("economy");
}

/// What the build needs to know about this pack beyond [`System`] (`DECISIONS.md` `ARC-33`): nothing
/// biographical — purchases and wages are thousands of facts that would bury a biography (step-10
/// QS-49) — the `economy:` section, which its `AuthoredSection` impl (`src/section.rs`) describes,
/// and the required `configure/economy.yaml` (`src/configuration.rs`, `ARC-61`, `ARC-111`).
impl SystemPack for EconomySystem {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_section!();
    mineworld_sdk::configures!();
}

/// `configure/economy.yaml` (`ARC-61`, `ARC-111`; WE-a design §6.2): the world's currency, required, and
/// its optional price modifiers.
impl PackConfiguration for EconomySystem {
    type Configuration = EconomyConfiguration;
    const FACTS: &'static [EventTypeId] =
        &[CurrencyDeclared::EVENT_TYPE, EconomyConfigured::EVENT_TYPE];

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

    /// One `currency-declared`, then — when any modifier is configured — one `economy-configured` with
    /// each shop resolved; both
    /// SystemInternal, about nobody (`ARC-61` item 8; Q-S31a-2): the currency reaches observers only
    /// through the `currency` record.
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
        let mut facts = vec![Emission::new::<CurrencyDeclared>(
            codec::encode(&CurrencyDeclared::new(configuration.currency())),
            Visibility::SystemInternal,
        )];
        // A file with its currency only configures no price book: nothing answers a happening.
        if !modifiers.is_empty() {
            facts.push(Emission::new::<EconomyConfigured>(
                codec::encode(&EconomyConfigured::new(modifiers)),
                Visibility::SystemInternal,
            ));
        }
        Ok(facts)
    }
}

/// A fact this pack may not take, reached at reduction or at resolve: a defect in whoever stated it
/// past the rule, reported as the owner's refusal (`ARC-26`).
pub(crate) fn refused(event_type: EventTypeId, reason: Rejection) -> KernelError {
    KernelError::FactRefusedByOwner {
        system: EconomySystem::ID,
        event_type,
        reason,
    }
}

/// Refuses `event_type` with `economy-currency-undeclared` when the world declared no currency: an
/// amount with no scale (`ARC-111`, SD-S31a-1). Configuration is seeded before sections, so a world
/// that authors an amount without `configure/economy.yaml` fails at genesis, naming the code.
pub(crate) fn currency_declared(
    world: &WorldRead<'_>,
    event_type: &EventTypeId,
) -> Result<(), KernelError> {
    match currency_process(world) {
        Some(_) => Ok(()),
        None => Err(refused(
            event_type.clone(),
            code::refusal(CURRENCY_UNDECLARED),
        )),
    }
}

impl System for EconomySystem {
    /// 2 since WE-a: the declaration grew a configuration, two facts and two subscriptions, so a save
    /// written by version 1 is refused by name (`ARC-25`). 3 since PR LE-b (`ARC-287`): it answers
    /// `wage-deduction-due` and states `deduction-unpaid`. 4 since S31-a (`ARC-111`, `ARC-112`): the
    /// currency is required configuration, and prices are the sellers' asks.
    const VERSION: SystemVersion = SystemVersion::new(4);

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
            .owning::<Asks>()
            .owning::<Shop>()
            .owning::<ReferenceValue>()
            .owning::<CurrencyRecord>()
            .owning::<Wealth>()
            .owning::<ShopStock>()
            .providing::<Buy>()
            .providing::<ListForSale>()
            .providing::<SetPrice>()
            .providing::<Withdraw>()
            .emitting::<CurrencyDeclared>()
            .emitting::<ReferenceDeclared>()
            .emitting::<Funded>()
            .emitting::<ShopOpened>()
            .emitting::<Listed>()
            .emitting::<PriceSet>()
            .emitting::<Withdrawn>()
            .emitting::<Sold>()
            .emitting::<MoneyTransferred>()
            .emitting::<WageUnpaid>()
            .emitting::<DeductionUnpaid>()
            .emitting::<ItemsTransferred>()
            .emitting::<EconomyConfigured>()
            .emitting::<PricesAdjusted>()
            .emitting::<PricesRestored>()
            .subscribing_to::<CurrencyDeclared>()
            .subscribing_to::<ReferenceDeclared>()
            .subscribing_to::<Funded>()
            .subscribing_to::<ShopOpened>()
            .subscribing_to::<Listed>()
            .subscribing_to::<PriceSet>()
            .subscribing_to::<Withdrawn>()
            .subscribing_to::<MoneyTransferred>()
            .subscribing_to::<WageDue>()
            .subscribing_to::<WageDeductionDue>()
            .subscribing_to::<EconomyConfigured>()
            .subscribing_to::<HappeningBegan>()
            .subscribing_to::<HappeningEnded>()
            .subscribing_to::<PricesAdjusted>()
            .subscribing_to::<PricesRestored>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<Wallet>()?;
        tables.component::<Asks>()?;
        tables.component::<Shop>()?;
        tables.component::<ReferenceValue>()?;
        tables.component::<CurrencyRecord>()?;
        tables.component::<Wealth>()?;
        tables.component::<ShopStock>()
    }

    /// Whether this request may happen. A `buy` is judged by [`sale::validate`] (§5.7); a seller action
    /// by [`listing::judge_listing`], then its requirement (§5.6). The pack's own reasons come before
    /// the requirement, as an offer reports them (SD-S31a-2).
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let action = intent.action_type();
        if *action == Buy::ACTION_TYPE {
            let buy: Buy = codec::read(intent)?;
            return sale::validate(world, intent.actor(), intent.target(), &buy);
        }
        if intent.target().is_some() {
            return Err(Rejection::NoSupportedInteraction);
        }
        let request = seller_request(intent)?;
        let judged = judge_listing(world, intent.actor(), &request)?;
        match judged.shop() {
            None => Ok(()),
            Some(_) => {
                let here =
                    sale::located(world, intent.actor()).ok_or(Rejection::PreconditionFailed)?;
                seller_requirement(judged.shop()).evaluate(&here, None, true)
            }
        }
    }

    /// A `buy` states its three facts through [`sale::settle`]; a seller action states its one fact,
    /// re-judged from the state at resolve. A refusal here is the owner's `FactRefusedByOwner`.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let read = world.read();
        if *intent.action_type() == Buy::ACTION_TYPE {
            let buy: Buy =
                codec::read(intent).map_err(|reason| refused(Sold::EVENT_TYPE, reason))?;
            return sale::settle(&read, intent.actor(), intent.target(), &buy);
        }
        let refusal = |reason| refused(Listed::EVENT_TYPE, reason);
        let request = seller_request(intent).map_err(refusal)?;
        let judged = judge_listing(&read, intent.actor(), &request).map_err(refusal)?;
        Ok(vec![listing::fact(&read, intent.actor(), &judged)])
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
        if *kind == CurrencyDeclared::EVENT_TYPE {
            let declared: CurrencyDeclared = codec::event_payload(event.payload())?;
            if currency_process(&world.read()).is_some() {
                return Err(refused(
                    CurrencyDeclared::EVENT_TYPE,
                    Rejection::PreconditionFailed,
                ));
            }
            world.start_process(
                ProcessStart::<CurrencyProcess>::new(codec::encode(declared.currency()))
                    .uninterruptible(),
            )?;
        } else if *kind == Funded::EVENT_TYPE {
            let funded: Funded = codec::event_payload(event.payload())?;
            let read = world.read();
            currency_declared(&read, kind)?;
            if !is_holder(&read, funded.holder())
                || read.component::<Wallet>(funded.holder()).is_some()
            {
                return Err(refused(Funded::EVENT_TYPE, Rejection::PreconditionFailed));
            }
            world.insert(funded.holder(), Wallet::new(funded.balance()))?;
        } else if *kind == ShopOpened::EVENT_TYPE {
            let opened: ShopOpened = codec::event_payload(event.payload())?;
            let read = world.read();
            currency_declared(&read, kind)?;
            let place = opened.place().entity_id();
            let is_place = read.entity(place).is_some_and(|record| {
                record.entity_type() == EntityType::Place
                    && record.lifecycle() != LifecycleState::Destroyed
            });
            let keepers_live = opened
                .keepers()
                .iter()
                .all(|keeper| living_person(&read, keeper.entity_id()));
            if !is_place
                || !is_holder(&read, opened.operator().entity_id())
                || read.component::<Shop>(place).is_some()
                || !keepers_live
            {
                return Err(refused(
                    ShopOpened::EVENT_TYPE,
                    Rejection::PreconditionFailed,
                ));
            }
            world.insert(
                place,
                Shop::new(opened.operator(), opened.keepers().to_vec()),
            )?;
        } else if *kind == Listed::EVENT_TYPE {
            let listed: Listed = codec::event_payload(event.payload())?;
            reduce_listed(world, &listed)?;
        } else if *kind == PriceSet::EVENT_TYPE {
            let set: PriceSet = codec::event_payload(event.payload())?;
            let read = world.read();
            let asks = asks_of(&read, set.seller())
                .repriced(set.item(), set.from(), set.to())
                .filter(|_| price_in_range(set.to()))
                .ok_or_else(|| refused(PriceSet::EVENT_TYPE, Rejection::PreconditionFailed))?;
            world.insert(set.seller(), asks)?;
        } else if *kind == Withdrawn::EVENT_TYPE {
            let gone: Withdrawn = codec::event_payload(event.payload())?;
            let asks = asks_of(&world.read(), gone.seller())
                .withdrawn(gone.item())
                .ok_or_else(|| refused(Withdrawn::EVENT_TYPE, Rejection::PreconditionFailed))?;
            world.insert(gone.seller(), asks)?;
        } else if *kind == ReferenceDeclared::EVENT_TYPE {
            let declared: ReferenceDeclared = codec::event_payload(event.payload())?;
            reduce_reference(world, &declared)?;
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
        } else if *kind == WageDeductionDue::EVENT_TYPE {
            let due: WageDeductionDue = codec::event_payload(event.payload())?;
            return Ok(vec![deduct(&world.read(), &due)]);
        } else {
            return react_to_prices(world, event);
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

/// A seller action's request, whichever of the three it is.
fn seller_request(intent: &ActionIntent) -> Result<SellerRequest, Rejection> {
    let action = intent.action_type();
    if *action == ListForSale::ACTION_TYPE {
        Ok(codec::read::<ListForSale>(intent)?.into())
    } else if *action == SetPrice::ACTION_TYPE {
        Ok(codec::read::<SetPrice>(intent)?.into())
    } else if *action == Withdraw::ACTION_TYPE {
        Ok(codec::read::<Withdraw>(intent)?.into())
    } else {
        Err(Rejection::NoSupportedInteraction)
    }
}

/// Whether `entity` is a living Item: an item kind (`ARC-36`).
fn living_item(world: &WorldRead<'_>, entity: EntityId) -> bool {
    world.entity(entity).is_some_and(|record| {
        record.entity_type() == EntityType::Item && record.lifecycle() != LifecycleState::Destroyed
    })
}

/// `listed` → the seller's [`Asks`] gain the ask. The owner still decides (`ARC-26`): a currency must be
/// declared, the seller a living holder, the kind a living Item, the price in `1 ..= PRICE_MAX`, and
/// the kind not asked already; otherwise nothing is written.
fn reduce_listed(
    world: &mut WorldView<'_, EconomySystem>,
    listed: &Listed,
) -> Result<(), KernelError> {
    let read = world.read();
    currency_declared(&read, &Listed::EVENT_TYPE)?;
    let refusal = || refused(Listed::EVENT_TYPE, Rejection::PreconditionFailed);
    if !is_holder(&read, listed.seller())
        || !living_item(&read, listed.item().entity_id())
        || !price_in_range(listed.price())
    {
        return Err(refusal());
    }
    let asks = asks_of(&read, listed.seller())
        .listing(Ask::new(listed.item(), listed.price()))
        .ok_or_else(refusal)?;
    world.insert(listed.seller(), asks)?;
    Ok(())
}

/// `reference-declared` → the kind's [`ReferenceValue`]: a currency declared, a living Item, a value in
/// `1 ..= PRICE_MAX`, and the kind given one value only.
fn reduce_reference(
    world: &mut WorldView<'_, EconomySystem>,
    declared: &ReferenceDeclared,
) -> Result<(), KernelError> {
    let read = world.read();
    currency_declared(&read, &ReferenceDeclared::EVENT_TYPE)?;
    let item = declared.item().entity_id();
    if !living_item(&read, item)
        || !price_in_range(declared.value())
        || read.component::<ReferenceValue>(item).is_some()
    {
        return Err(refused(
            ReferenceDeclared::EVENT_TYPE,
            Rejection::PreconditionFailed,
        ));
    }
    world.insert(item, ReferenceValue::new(declared.value()))?;
    Ok(())
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
    /// One complete buy per ask to a person standing in a shop, or perceiving a person seller
    /// (`src/sale.rs`); and the seller actions, each incomplete, once (`src/listing.rs`).
    fn offers(
        &self,
        world: &WorldRead<'_>,
        observer: EntityId,
        target: Option<EntityId>,
    ) -> Vec<Offer> {
        let mut offers = sale::buys(world, observer, target);
        offers.extend(seller_offers(world, observer, target));
        offers
    }

    /// A [`Wallet`] to its holder alone (`INV-13`); the world's [`CurrencyRecord`] about each living
    /// Person observer itself, wherever it is (`ARC-111`, SD-S31a-5); a shop's place's listing to whoever
    /// perceives the place — the operator, and per priced kind its price and how many are in stock
    /// (step-10 QS-45).
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
        if currency::tells(world, observer, subject)
            && let Some(record) = currency::record(world)
        {
            disclosed.push(ComponentRecord::new::<CurrencyRecord>(
                subject,
                codec::to_value(&record),
            ));
        }
        if observer == subject && world.component::<Wallet>(subject).is_some() {
            disclosed.push(ComponentRecord::new::<Wealth>(
                subject,
                codec::to_value(&wealth(world, subject)),
            ));
        }
        // Every shop's stocked kinds, to each living Person about itself (S35 N-h R5): only when the
        // world has a shop, so a world without one discloses exactly as before.
        if currency::tells(world, observer, subject) && world.component_count::<Shop>() > 0 {
            disclosed.push(ComponentRecord::new::<ShopStock>(
                subject,
                codec::to_value(&shop_stock(world, observer)),
            ));
        }
        if let Some(view) = asks_view(world, subject) {
            disclosed.push(ComponentRecord::new::<Asks>(
                subject,
                codec::to_value(&view),
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
