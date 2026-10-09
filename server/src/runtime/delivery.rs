//! Delivery: what the world thread sends each connected client, and how a client stops receiving.
//!
//! Moved out of `runtime.rs` before it grew (step-12 §17 SD-C12): the subscribers, the sweep that
//! hands each one its own observation, and the two ways a subscription ends — released by the seat
//! table, or departed because its connection ended. Since S11-C it also fans recorded facts out to
//! the observers who learned of them (`docs/DECISIONS.md` `ARC-43`):
//!
//! ```text
//! learn    each new fact, in log order: the event perception advances past it, then every
//!          subscriber's observer is asked about it — judged at record time, never at a sweep
//! queue    an admitted fact waits for the subscriber's next observation (bounded; the oldest are
//!          dropped and counted) and, for a connection that asked for the perceived stream, for
//!          that stream (bounded; past the bound the connection is released `lagged`)
//! sweep    per subscriber: waiting perceived facts first; only once they are queued, the
//!          observation carrying its events. A full channel skips the observation and keeps
//!          everything waiting, so no fact is lost to a dropped frame and none arrives late.
//! ```
//!
//! One channel per subscriber carries both, so the order a client reads is the order queued here.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;

use mineworld_contracts::{ActionId, EntityId, EventEnvelope, EventId};
use tokio::sync::{mpsc, oneshot};

use super::WorldRuntime;
use crate::host::{Backfill, Perceived, PerceivedStart, Streamed, SubscriptionId, WireFact};
use crate::perception::PerceptionContext;
use crate::protocol::{
    ClosingReason, PayloadForm, PerceivedJoin, Refusal, RefusalCode, report_not_json, wire_fact,
};
use crate::seats::Departure;

/// One connected client, as the world knows it: which observer, where to put what it is sent, how
/// to tell it that it no longer holds its seat, and what is waiting for it.
pub(super) struct Subscriber {
    pub(super) subscription: SubscriptionId,
    pub(super) observer: EntityId,
    pub(super) observations: mpsc::Sender<Streamed>,
    pub(super) released: oneshot::Sender<ClosingReason>,
    /// Facts learned since the last observation this connection was sent, oldest first.
    events: VecDeque<WireFact>,
    /// The connection's perceived stream, when its join asked for one.
    perceiving: Option<Perceiving>,
    /// The newest request this connection submitted that has been dispatched.
    acted_through: Option<ActionId>,
}

/// A connection's reliable stream: what is waiting, and the cursor after it.
struct Perceiving {
    pending: Vec<WireFact>,
    /// The newest fact considered for this connection, admitted or not.
    through: Option<EventId>,
}

impl Subscriber {
    /// A newly seated connection; `perceiving` from `head` when its join asked for the stream.
    pub(super) fn new(
        subscription: SubscriptionId,
        observer: EntityId,
        streams: (mpsc::Sender<Streamed>, oneshot::Sender<ClosingReason>),
        perceiving: Option<Option<EventId>>,
    ) -> Self {
        let (observations, released) = streams;
        Self {
            subscription,
            observer,
            observations,
            released,
            events: VecDeque::new(),
            perceiving: perceiving.map(|head| Perceiving {
                pending: Vec::new(),
                through: head,
            }),
            acted_through: None,
        }
    }
}

impl WorldRuntime {
    /// Where a joining connection's perceived stream starts, or why its cursor cannot be served
    /// (`PROTOCOL.md` §5.8). Asks nothing of the seat table, so a refusal grants nothing.
    pub(super) fn perceived_start(
        &self,
        asked: Option<PerceivedJoin>,
    ) -> Result<Option<PerceivedStart>, Refusal> {
        let Some(asked) = asked else {
            return Ok(None);
        };
        let unavailable = |detail: &str| {
            Err(Refusal::new(RefusalCode::CursorUnavailable).detail(detail.to_owned()))
        };
        let head = self.head;
        let backfill = match (asked.since, head) {
            (Some(since), Some(head)) if since > head => {
                return unavailable("the cursor is newer than any fact this world has recorded");
            }
            (Some(_), None) => {
                return unavailable("the cursor is newer than any fact this world has recorded");
            }
            (Some(since), Some(head)) if since == head => None,
            (_, None) => None,
            (since, Some(through)) => match &self.history {
                Some(history) => Some(Backfill {
                    history: Arc::clone(history),
                    since,
                    through,
                }),
                None => {
                    return unavailable(
                        "this world keeps no history; it serves the perceived stream from the \
                         join on (perceived.since = the newest fact id, or join without perceived)",
                    );
                }
            },
        };
        Ok(Some(PerceivedStart { head, backfill }))
    }

    /// The facts a dispatch or an advance just recorded, judged one at a time, in log order.
    pub(super) fn learn(&mut self, facts: &[EventEnvelope]) {
        for fact in facts {
            self.learn_one(fact);
        }
    }

    fn learn_one(&mut self, fact: &EventEnvelope) {
        self.audience.record(fact);
        self.head = Some(fact.id());
        let mut rendered: Option<WireFact> = None;
        let mut lagged = Vec::new();
        let (event_backlog, perceived_backlog) =
            (self.config.event_backlog, self.config.perceived_backlog);
        for subscriber in &mut self.subscribers {
            if let Some(perceiving) = &mut subscriber.perceiving {
                perceiving.through = Some(fact.id());
            }
            if !self.audience.admits(fact, subscriber.observer) {
                continue;
            }
            let wire = match &rendered {
                Some(wire) => Arc::clone(wire),
                None => match render(fact) {
                    Some(wire) => Arc::clone(rendered.insert(wire)),
                    None => continue,
                },
            };
            subscriber.events.push_back(Arc::clone(&wire));
            if subscriber.events.len() > event_backlog {
                subscriber.events.pop_front();
                self.events_dropped += 1;
            }
            if let Some(perceiving) = &mut subscriber.perceiving {
                perceiving.pending.push(wire);
                if perceiving.pending.len() > perceived_backlog {
                    lagged.push(subscriber.subscription);
                }
            }
        }
        for subscription in lagged {
            self.lag(subscription);
        }
    }

    /// Records that a request submitted on this connection has been dispatched.
    pub(super) fn acted(&mut self, subscription: SubscriptionId, action: ActionId) {
        if let Some(subscriber) = self
            .subscribers
            .iter_mut()
            .find(|subscriber| subscriber.subscription == subscription)
        {
            subscriber.acted_through = subscriber.acted_through.max(Some(action));
        }
    }

    /// Releases a connection whose perceived stream overflowed: it is told `lagged`, and its seat is
    /// held as for a dropped socket, so that it resumes with its `resume` and its cursor.
    fn lag(&mut self, subscription: SubscriptionId) {
        self.release(subscription, ClosingReason::Lagged);
        self.table.depart(
            subscription,
            Departure::Dropped,
            Instant::now(),
            self.clock.now(),
        );
    }

    /// Tells a connection it no longer holds its seat, and stops streaming to it.
    pub(super) fn release(&mut self, subscription: SubscriptionId, reason: ClosingReason) {
        if let Some(index) = self
            .subscribers
            .iter()
            .position(|subscriber| subscriber.subscription == subscription)
        {
            let subscriber = self.subscribers.swap_remove(index);
            let _ = subscriber.released.send(reason);
        }
    }

    /// A connection ended: it stops counting as a client, and its seat is held or returned.
    pub(super) fn depart(&mut self, subscription: SubscriptionId, departure: Departure) {
        self.subscribers
            .retain(|subscriber| subscriber.subscription != subscription);
        self.table
            .depart(subscription, departure, Instant::now(), self.clock.now());
    }

    /// Sends every connected client the observation *its* observer is entitled to.
    ///
    /// One perception call per subscriber, which is the whole of `INV-13` at this layer: there is no
    /// world frame that is then filtered per client, and no client receives anything that was
    /// computed for anybody else. Each carries the world's persisted revision, which every input
    /// so far has already been committed to.
    ///
    /// `try_send`, never `send`: a client that has stopped reading loses frames, and the world does
    /// not wait. A client whose channel has closed is dropped here — as a dropped connection, whose
    /// seat is held — which is how a killed connection is reaped if its session never got to
    /// release the subscription.
    ///
    /// A connection's waiting perceived facts go first. While they cannot be queued its observation
    /// is skipped (and counted as dropped), so that no observation reaches a client ahead of the
    /// facts it reflects (`PROTOCOL.md` §5.8).
    pub(super) fn sweep(&mut self) {
        let at = self.clock.now();
        let revision = self.world.revision();
        let mut dropped = 0;
        let mut closed: Vec<SubscriptionId> = Vec::new();

        for subscriber in &mut self.subscribers {
            if let Some(perceiving) = &mut subscriber.perceiving
                && let Some(through) = perceiving.through
                && !perceiving.pending.is_empty()
            {
                let facts = Streamed::Facts {
                    through,
                    events: perceiving.pending.clone(),
                };
                match subscriber.observations.try_send(facts) {
                    Ok(()) => perceiving.pending.clear(),
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        dropped += 1;
                        continue;
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        closed.push(subscriber.subscription);
                        continue;
                    }
                }
            }
            let context =
                PerceptionContext::new(self.world.world(), subscriber.observer, at, &self.recent);
            // Appended to whatever the perception seam itself put there (presence's `observe` puts
            // nothing; a perception that states events of its own keeps them).
            let observation = self.perception.observe(&context);
            let events = observation
                .events()
                .iter()
                .cloned()
                .chain(subscriber.events.iter().map(|wire| (**wire).clone()))
                .collect();
            let observation = observation.with_events(events);
            match subscriber
                .observations
                .try_send(Streamed::Observation(Perceived {
                    revision,
                    acted_through: subscriber.acted_through,
                    observation,
                })) {
                Ok(()) => subscriber.events.clear(),
                Err(mpsc::error::TrySendError::Full(_)) => dropped += 1,
                Err(mpsc::error::TrySendError::Closed(_)) => closed.push(subscriber.subscription),
            }
        }

        self.dropped += dropped;
        for subscription in closed {
            self.depart(subscription, Departure::Dropped);
        }
    }
}

/// A fact in its wire form, rendered once however many connections learned of it.
fn render(fact: &EventEnvelope) -> Option<WireFact> {
    match wire_fact(fact) {
        Ok((event, form)) => {
            if form == PayloadForm::NotJson {
                report_not_json(fact.event_type());
            }
            Some(Arc::new(event))
        }
        Err(error) => {
            eprintln!(
                "[world] fact {} could not be written for a client: {error}",
                fact.id().raw()
            );
            None
        }
    }
}
