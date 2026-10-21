//! The price book: a surveyor's notes of the prices it saw, on the person (component type
//! `survey-book`; not WE-a's `price-book` Process, SD-S31c-1).

use mineworld_contracts::{EntityId, ItemId, PlaceId, WorldTime};
use mineworld_item::Category;
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::event::{Noted, Source};
use crate::system::PriceSurveySystem;

/// A surveyor's notes: what it tracks, and the prices it saw. On a Person; written only by this
/// pack's reductions of `survey-book-opened` and `price-noted` (I-c-2).
///
/// One ring per kind holds at most `depth` notes, oldest first; kinds never evict each other. Notes
/// older than `window` are pruned at each reduction and filtered out at disclosure
/// ([`PriceBook::visible_at`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceBook {
    categories: Vec<Category>,
    depth: u8,
    window: u32,
    resurvey_after: u32,
    kinds: Vec<KindNotes>,
    surveyed: Vec<Surveyed>,
}

owned_component! {
    component = PriceBook,
    owner = PriceSurveySystem,
    component_type = "survey-book",
    schema_version = 1,
}

/// The notes of one kind, oldest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindNotes {
    item: ItemId,
    notes: Vec<Note>,
}

/// One price the surveyor saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    at: WorldTime,
    place: PlaceId,
    seller: EntityId,
    price: u64,
    source: Source,
    in_stock: Option<u32>,
}

/// When a place was last surveyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Surveyed {
    place: PlaceId,
    at: WorldTime,
}

impl PriceBook {
    pub(crate) const fn opened(
        categories: Vec<Category>,
        depth: u8,
        window: u32,
        resurvey_after: u32,
    ) -> Self {
        Self {
            categories,
            depth,
            window,
            resurvey_after,
            kinds: Vec::new(),
            surveyed: Vec::new(),
        }
    }

    /// The tracked categories, sorted.
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }

    /// Whether `category` is tracked.
    pub fn tracks(&self, category: &Category) -> bool {
        self.categories.binary_search(category).is_ok()
    }

    /// Notes kept per kind.
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// The oldest note kept, in seconds.
    pub const fn window(&self) -> u32 {
        self.window
    }

    /// The cooldown per place, in seconds.
    pub const fn resurvey_after(&self) -> u32 {
        self.resurvey_after
    }

    /// Every kind with at least one note, by item.
    pub fn kinds(&self) -> &[KindNotes] {
        &self.kinds
    }

    /// The notes of `item`, if any.
    pub fn kind(&self, item: ItemId) -> Option<&KindNotes> {
        self.kinds
            .binary_search_by_key(&item, |kind| kind.item)
            .ok()
            .map(|index| &self.kinds[index])
    }

    /// The last survey of each place, by place.
    pub fn surveyed(&self) -> &[Surveyed] {
        &self.surveyed
    }

    /// When `place` was last surveyed.
    pub fn last_surveyed(&self, place: PlaceId) -> Option<WorldTime> {
        self.surveyed
            .binary_search_by_key(&place, |entry| entry.place)
            .ok()
            .map(|index| self.surveyed[index].at)
    }

    /// Appends `notes`, seen at `at` in `place`: per kind, truncated to `depth` from the front; then
    /// every note and survey older than the window at `at` pruned; then, for a survey, `place`
    /// marked surveyed at `at`.
    pub(crate) fn note(&mut self, at: WorldTime, place: PlaceId, source: Source, notes: &[Noted]) {
        for noted in notes {
            let note = Note {
                at,
                place,
                seller: noted.seller(),
                price: noted.price(),
                source,
                in_stock: noted.in_stock(),
            };
            let index = match self
                .kinds
                .binary_search_by_key(&noted.item(), |kind| kind.item)
            {
                Ok(index) => index,
                Err(index) => {
                    self.kinds.insert(
                        index,
                        KindNotes {
                            item: noted.item(),
                            notes: Vec::new(),
                        },
                    );
                    index
                }
            };
            let ring = &mut self.kinds[index].notes;
            ring.push(note);
            let excess = ring.len().saturating_sub(usize::from(self.depth));
            ring.drain(..excess);
        }
        self.prune(at);
        if source == Source::Ask {
            match self
                .surveyed
                .binary_search_by_key(&place, |entry| entry.place)
            {
                Ok(index) => self.surveyed[index].at = at,
                Err(index) => self.surveyed.insert(index, Surveyed { place, at }),
            }
        }
    }

    /// Drops every note and survey that is no longer current at `now`, and every kind left empty.
    fn prune(&mut self, now: WorldTime) {
        // A note seen at `at` is current while `at + window > now`: gone exactly at `at + window`.
        let window = self.window;
        let current = |at: WorldTime| at.seconds() + i64::from(window) > now.seconds();
        for kind in &mut self.kinds {
            kind.notes.retain(|note| current(note.at));
        }
        self.kinds.retain(|kind| !kind.notes.is_empty());
        self.surveyed.retain(|entry| current(entry.at));
    }

    /// The book as its holder sees it at `now`: every note and survey older than the window removed.
    #[must_use]
    pub fn visible_at(&self, now: WorldTime) -> Self {
        let mut visible = self.clone();
        visible.prune(now);
        visible
    }
}

impl KindNotes {
    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// Its notes, oldest first.
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }
}

impl Note {
    /// When it was seen.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// Where.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Who asked or sold.
    pub const fn seller(&self) -> EntityId {
        self.seller
    }

    /// The unit price, in minor units.
    pub const fn price(&self) -> u64 {
        self.price
    }

    /// Asked or sold.
    pub const fn source(&self) -> Source {
        self.source
    }

    /// A shop ask's stock count; `None` for a person's ask and for every sale.
    pub const fn in_stock(&self) -> Option<u32> {
        self.in_stock
    }
}

impl Surveyed {
    /// Which place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// When.
    pub const fn at(&self) -> WorldTime {
        self.at
    }
}
