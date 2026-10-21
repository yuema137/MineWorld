//! The facts this pack records. Only this pack states them, and only its reductions turn them into
//! state (`I-S31-2`). Both are `SystemInternal` (SD-S31c-7, Q-S31c-1): the surveyor reads its notes
//! through its own `survey-book` record, and its inbox keeps only what happened in the world.
//!
//! ```text
//! survey-book-opened   genesis: a surveyor's settings      SystemInternal, about the surveyor
//! price-noted          a witnessed sale or a survey         SystemInternal, about the surveyor
//! ```

use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, ItemId, PersonId, PlaceId, SystemId,
    Visibility,
};
use mineworld_item::Category;
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::system::PriceSurveySystem;

/// A person became a surveyor, with these settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyBookOpened {
    surveyor: PersonId,
    categories: Vec<Category>,
    depth: u8,
    window_s: u32,
    resurvey_after_s: u32,
}

impl Event for SurveyBookOpened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("survey-book-opened");
    const OWNER: SystemId = PriceSurveySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl SurveyBookOpened {
    pub(crate) const fn new(
        surveyor: PersonId,
        categories: Vec<Category>,
        depth: u8,
        window_s: u32,
        resurvey_after_s: u32,
    ) -> Self {
        Self {
            surveyor,
            categories,
            depth,
            window_s,
            resurvey_after_s,
        }
    }

    /// Who.
    pub const fn surveyor(&self) -> PersonId {
        self.surveyor
    }

    /// The tracked categories, sorted.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// Notes kept per kind.
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// The oldest note kept, in seconds.
    pub const fn window_s(&self) -> u32 {
        self.window_s
    }

    /// The cooldown per place, in seconds.
    pub const fn resurvey_after_s(&self) -> u32 {
        self.resurvey_after_s
    }

    pub(crate) fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::SystemInternal)
            .about(vec![self.surveyor.entity_id()])
    }
}

/// Where a note came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// An ask on display, copied by `survey`.
    Ask,
    /// A sale the surveyor perceived.
    Sale,
}

/// One price in a `price-noted` fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Noted {
    item: ItemId,
    seller: EntityId,
    price: u64,
    in_stock: Option<u32>,
}

impl Noted {
    pub(crate) const fn new(
        item: ItemId,
        seller: EntityId,
        price: u64,
        in_stock: Option<u32>,
    ) -> Self {
        Self {
            item,
            seller,
            price,
            in_stock,
        }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// Who asked or sold: a Person, or the Organization that runs the shop.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// The unit price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// A shop ask's stock count; `None` for a person's ask and for a sale.
    pub const fn in_stock(&self) -> Option<u32> {
        self.in_stock
    }
}

/// A surveyor noted prices: one witnessed sale, or the asks of one survey.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceNoted {
    surveyor: PersonId,
    place: PlaceId,
    source: Source,
    notes: Vec<Noted>,
}

impl Event for PriceNoted {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("price-noted");
    const OWNER: SystemId = PriceSurveySystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PriceNoted {
    /// `notes` are sorted by `(item, seller)` here, so the fact has one form.
    pub(crate) fn new(
        surveyor: PersonId,
        place: PlaceId,
        source: Source,
        mut notes: Vec<Noted>,
    ) -> Self {
        notes.sort_by_key(|note| (note.item, note.seller));
        Self {
            surveyor,
            place,
            source,
            notes,
        }
    }

    /// Whose book.
    pub const fn surveyor(&self) -> PersonId {
        self.surveyor
    }

    /// Where it was seen.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Witnessed or surveyed.
    pub const fn source(&self) -> Source {
        self.source
    }

    /// The prices, by `(item, seller)`.
    pub fn notes(&self) -> &[Noted] {
        &self.notes
    }

    /// SystemInternal (Q-S31c-1), about the surveyor.
    pub(crate) fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::SystemInternal)
            .about(vec![self.surveyor.entity_id()])
    }
}
