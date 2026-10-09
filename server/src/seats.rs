//! Who drives each seat: the binding state machine, and its only writer (`PROTOCOL.md` §4.2).
//!
//! ```text
//! free        nobody drives the Person
//! hosted      an in-server controller drives it
//! connected   one connection drives it
//! held        its connection dropped; nobody drives it until `until` or the resume comes back
//! ```
//!
//! Every transition here is **host state**: none is journaled, none states a fact, none moves the
//! world's revision (`docs/DECISIONS.md` `ARC-40`). The table touches no world and no socket. It is
//! owned by the world thread, which is the only caller, so there is one writer of bindings and two
//! controllers can never drive one seat (step-12 I-3). Time is injected — a wall-clock `Instant` for
//! holds, a `WorldTime` for binding a controller — so every transition is testable with no thread.

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use mineworld_contracts::{EntityKey, WorldTime};

use crate::admission::{OfferedResume, ResumeSecret};
use crate::host::SubscriptionId;
use crate::hosted::{HostedFactory, HostedSlot};
use crate::protocol::{ClosingReason, Refusal, RefusalCode, SessionId, TookOver};

/// What a connection asks for when it joins: a seat, and how it means to take it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinRequest {
    /// The seat, by the authoring key of its Person.
    pub seat: EntityKey,
    /// Take the seat from another connection, or from a dropped one's hold (`join.take_over`).
    pub take_over: bool,
    /// The secret of an earlier binding of this seat, to re-take it (`join.resume`).
    pub resume: Option<OfferedResume>,
    /// Which connection is asking.
    pub session: SessionId,
}

impl JoinRequest {
    /// A plain join: no takeover, no resume.
    pub const fn plain(seat: EntityKey, session: SessionId) -> Self {
        Self {
            seat,
            take_over: false,
            resume: None,
            session,
        }
    }
}

/// How a connection ended, which decides what becomes of its seat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Departure {
    /// It said `leave`, or the server closed it: the seat returns to its default at once.
    Left,
    /// Its socket ended without a word: the seat is held for the server's hold.
    Dropped,
}

/// One seat's binding.
enum Binding {
    Free,
    Hosted(HostedSlot),
    Connected(Connected),
    Held(Held),
}

struct Connected {
    subscription: SubscriptionId,
    /// Which connection, for the admin surface (S11-D).
    #[allow(dead_code)]
    session: SessionId,
    resume: ResumeSecret,
}

struct Held {
    until: Instant,
    resume: ResumeSecret,
}

/// A granted join: what changed hands, and which older connection, if any, must now be closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Grant {
    pub(crate) took_over: TookOver,
    pub(crate) displaced: Option<(SubscriptionId, ClosingReason)>,
}

/// Every seat's binding.
pub(crate) struct SeatTable {
    seats: BTreeMap<EntityKey, Binding>,
    factories: BTreeMap<EntityKey, HostedFactory>,
    hold: Duration,
}

impl SeatTable {
    /// Every seat at its default, with controllers bound at `at`.
    pub(crate) fn new(
        seats: impl IntoIterator<Item = EntityKey>,
        factories: BTreeMap<EntityKey, HostedFactory>,
        hold: Duration,
        at: WorldTime,
    ) -> Self {
        let mut table = Self {
            seats: BTreeMap::new(),
            factories,
            hold,
        };
        for seat in seats {
            let default = table.default(&seat, at);
            table.seats.insert(seat, default);
        }
        table
    }

    /// The binding a seat returns to: its controller, built afresh at `at`, or nobody.
    fn default(&self, seat: &EntityKey, at: WorldTime) -> Binding {
        self.factories.get(seat).map_or(Binding::Free, |factory| {
            Binding::Hosted(HostedSlot::bind(factory, at))
        })
    }

    /// Decides a join (`PROTOCOL.md` §4.2's rules, in order) and, if granted, binds the seat to
    /// `subscription` with the fresh secret `resume`. `now` is the wall clock, `at` the world's.
    pub(crate) fn join(
        &mut self,
        request: &JoinRequest,
        subscription: SubscriptionId,
        resume: ResumeSecret,
        now: Instant,
        at: WorldTime,
    ) -> Result<Grant, Refusal> {
        self.expire_seat(&request.seat, now, at);
        let binding = self
            .seats
            .get(&request.seat)
            .ok_or_else(|| Refusal::new(RefusalCode::UnknownSeat))?;
        let grant = decide(binding, request)?;
        self.seats.insert(
            request.seat.clone(),
            Binding::Connected(Connected {
                subscription,
                session: request.session,
                resume,
            }),
        );
        Ok(grant)
    }

    /// A connection ended. Its seat is held or returned to its default; a connection that no longer
    /// holds a seat — it was taken over or superseded — changes nothing. Returns the seat, if any.
    pub(crate) fn depart(
        &mut self,
        subscription: SubscriptionId,
        departure: Departure,
        now: Instant,
        at: WorldTime,
    ) -> Option<EntityKey> {
        let seat = self
            .seats
            .iter()
            .find_map(|(seat, binding)| match binding {
                Binding::Connected(connected) if connected.subscription == subscription => {
                    Some(seat.clone())
                }
                _ => None,
            })?;
        let next = match departure {
            Departure::Dropped if !self.hold.is_zero() => {
                let Some(Binding::Connected(connected)) = self.seats.remove(&seat) else {
                    unreachable!("the seat was just found connected");
                };
                Binding::Held(Held {
                    until: now + self.hold,
                    resume: connected.resume,
                })
            }
            Departure::Dropped | Departure::Left => self.default(&seat, at),
        };
        self.seats.insert(seat.clone(), next);
        Some(seat)
    }

    /// Returns every seat whose hold has ended to its default.
    pub(crate) fn expire(&mut self, now: Instant, at: WorldTime) {
        let ended: Vec<EntityKey> = self
            .seats
            .iter()
            .filter(|(_, binding)| matches!(binding, Binding::Held(held) if held.until <= now))
            .map(|(seat, _)| seat.clone())
            .collect();
        for seat in ended {
            self.expire_seat(&seat, now, at);
        }
    }

    fn expire_seat(&mut self, seat: &EntityKey, now: Instant, at: WorldTime) {
        if matches!(self.seats.get(seat), Some(Binding::Held(held)) if held.until <= now) {
            let default = self.default(seat, at);
            self.seats.insert(seat.clone(), default);
        }
    }

    /// The hosted seats due by `now`, in instant order and then seat order.
    pub(crate) fn due(&self, now: WorldTime) -> Vec<(WorldTime, EntityKey)> {
        let mut due: Vec<(WorldTime, EntityKey)> = self
            .seats
            .iter()
            .filter_map(|(seat, binding)| match binding {
                Binding::Hosted(slot) if slot.next() <= now => Some((slot.next(), seat.clone())),
                _ => None,
            })
            .collect();
        due.sort();
        due
    }

    /// The controller bound to a seat, if an in-server controller drives it now.
    pub(crate) fn hosted_mut(&mut self, seat: &EntityKey) -> Option<&mut HostedSlot> {
        match self.seats.get_mut(seat) {
            Some(Binding::Hosted(slot)) => Some(slot),
            _ => None,
        }
    }
}

/// `PROTOCOL.md` §4.2's join rules over one seat's binding, in order.
fn decide(binding: &Binding, request: &JoinRequest) -> Result<Grant, Refusal> {
    let granted = |took_over, displaced| {
        Ok(Grant {
            took_over,
            displaced,
        })
    };
    if let Some(offered) = &request.resume {
        return match binding {
            Binding::Held(held) if held.resume.matches(offered) => granted(TookOver::Held, None),
            Binding::Connected(connected) if connected.resume.matches(offered) => granted(
                TookOver::Held,
                Some((connected.subscription, ClosingReason::Superseded)),
            ),
            _ => Err(Refusal::new(RefusalCode::InvalidResume)
                .detail("no hold of this seat matches that resume; join again without it")),
        };
    }
    match binding {
        Binding::Free => granted(TookOver::None, None),
        Binding::Hosted(_) => granted(TookOver::Hosted, None),
        Binding::Connected(connected) if request.take_over => granted(
            TookOver::Connection,
            Some((connected.subscription, ClosingReason::TakenOver)),
        ),
        Binding::Held(_) if request.take_over => granted(TookOver::Connection, None),
        Binding::Connected(_) | Binding::Held(_) => Err(Refusal::new(RefusalCode::SeatOccupied)
            .detail("another player holds this seat; join with take_over to take it")),
    }
}

#[cfg(test)]
mod tests;
