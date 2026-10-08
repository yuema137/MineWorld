//! Walking pushes objects (step-11 §4.5.4, SD-O8, SD-O10): **one function, used twice** — by the
//! resolver's prediction before presence records an arrival, and by this pack's reaction to every
//! recorded `arrived`.
//!
//! Every input of a push is the same in both: the person's end point, the room, and the objects as
//! they lay before the request. Reduction is breadth-first (step-11 F-O2): when the reactions run,
//! no `object-moved` of the request has been reduced yet, so the objects are exactly where the
//! prediction saw them. People are not obstacles to the cast — in a reaction they are not yet where
//! the request leaves them — so the prediction checks them in the final state instead.

use std::cell::OnceCell;

use mineworld_contracts::ItemId;

use crate::footprint::{Footprint, Placed, Resting, flaw, push_offset};
use crate::geometry::{Point, Room, SNAP, TOLERANCE, distance2};
use crate::rapier::Pile;

/// One object pushed: which (its index among the objects as they lay, and its id), where it lay, and
/// where its centre ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Push {
    pub(crate) index: usize,
    pub(crate) object: ItemId,
    pub(crate) from: Placed,
    pub(crate) to: Point,
}

/// A push that cannot be made: the object would be stopped short by the walls, a solid or another
/// object, or no push within the search clears it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Jam {
    pub(crate) object: ItemId,
}

/// The objects of one place as they lay before the request, and the room they lie in. The cast's
/// scene is built the first time a push needs it, and dropped with this value.
pub(crate) struct Lay<'a> {
    room: &'a Room,
    objects: &'a [(ItemId, Placed)],
    pile: OnceCell<Pile>,
}

impl<'a> Lay<'a> {
    pub(crate) fn new(room: &'a Room, objects: &'a [(ItemId, Placed)]) -> Self {
        Self {
            room,
            objects,
            pile: OnceCell::new(),
        }
    }

    /// The objects as they lay, as footprints.
    pub(crate) fn footprints(&self) -> Vec<Footprint> {
        self.objects
            .iter()
            .map(|(_, placed)| placed.footprint())
            .collect()
    }

    /// The pushes a person standing at `p` makes (step-11 SD-O8), in `ItemId` order: every object
    /// lying on the floor whose footprint the person's disc overlaps by more than `TOLERANCE`
    /// (§18.11 DO-5), moved straight away from `p` just far enough to stand `PERSON_RADIUS + GAP`
    /// clear of it, by one shape cast against the fixed geometry and the other objects as they lay.
    /// A cast cut short by more than [`SNAP`] is a [`Jam`].
    pub(crate) fn pushes(&self, p: Point) -> Result<Vec<Push>, Jam> {
        let mut pushes = Vec::new();
        for (index, (object, placed)) in self.objects.iter().enumerate() {
            if placed.resting(self.room, TOLERANCE.value()) != Some(Resting::Floor)
                || !placed.footprint().under(p)
            {
                continue;
            }
            let jam = Jam { object: *object };
            let offset = push_offset(placed.footprint(), p).ok_or(jam)?;
            let target = placed.centre.plus(offset);
            let pile = self
                .pile
                .get_or_init(|| Pile::build(self.room, &self.placed()));
            let end = pile.cast(index, offset);
            if distance2(end, target) > i64::from(SNAP.value()).pow(2) {
                return Err(jam);
            }
            pushes.push(Push {
                index,
                object: *object,
                from: *placed,
                to: target,
            });
        }
        Ok(pushes)
    }

    fn placed(&self) -> Vec<Placed> {
        self.objects.iter().map(|(_, placed)| *placed).collect()
    }

    /// The resolver's prediction (step-11 SD-O9 step 6): the pushes of every arrival of one emission
    /// list — `arrivals`, the walker's end then each displaced person's, in that order — accepted
    /// only if nothing jams, no object is pushed twice, and in the final state (`people`: everybody in
    /// the place where the list leaves them) every pushed object keeps the stored-state invariant and
    /// no person overlaps any object. On acceptance, the objects' final footprints come with the
    /// pushes.
    pub(crate) fn predict(&self, arrivals: &[Point], people: &[Point]) -> Option<Predicted> {
        let mut pushes: Vec<Push> = Vec::new();
        for at in arrivals {
            for push in self.pushes(*at).ok()? {
                if pushes.iter().any(|done| done.index == push.index) {
                    return None;
                }
                pushes.push(push);
            }
        }
        let mut after = self.footprints();
        for push in &pushes {
            after[push.index] = after[push.index].at(push.to);
        }
        for push in &pushes {
            let others: Vec<Footprint> = after
                .iter()
                .enumerate()
                .filter(|(index, _)| *index != push.index)
                .map(|(_, footprint)| *footprint)
                .collect();
            let end = Placed {
                centre: push.to,
                ..push.from
            };
            if flaw(self.room, &end, &others, people, TOLERANCE.value()).is_some() {
                return None;
            }
        }
        if people
            .iter()
            .any(|person| after.iter().any(|footprint| footprint.under(*person)))
        {
            return None;
        }
        Some(Predicted { pushes, after })
    }
}

/// An accepted prediction: the pushes, and every object's footprint once they are made.
pub(crate) struct Predicted {
    pub(crate) pushes: Vec<Push>,
    pub(crate) after: Vec<Footprint>,
}
