//! Delivery: what the world thread sends each connected client, and how a client stops receiving.
//!
//! Moved out of `runtime.rs` before it grows (step-12 §17 SD-C12): the subscribers, the sweep that
//! hands each one its own observation, and the two ways a subscription ends — released by the seat
//! table, or departed because its connection ended.

use std::time::Instant;

use mineworld_contracts::EntityId;
use tokio::sync::{mpsc, oneshot};

use super::WorldRuntime;
use crate::host::{Perceived, SubscriptionId};
use crate::perception::PerceptionContext;
use crate::protocol::ClosingReason;
use crate::seats::Departure;

/// One connected client, as the world knows it: which observer, where to put its observations, and
/// how to tell it that it no longer holds its seat.
pub(super) struct Subscriber {
    pub(super) subscription: SubscriptionId,
    pub(super) observer: EntityId,
    pub(super) observations: mpsc::Sender<Perceived>,
    pub(super) released: oneshot::Sender<ClosingReason>,
}

impl WorldRuntime {
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
    pub(super) fn sweep(&mut self) {
        let at = self.clock.now();
        let revision = self.world.revision();
        let mut dropped = 0;
        let mut closed: Vec<SubscriptionId> = Vec::new();

        for subscriber in &self.subscribers {
            let context =
                PerceptionContext::new(self.world.world(), subscriber.observer, at, &self.recent);
            let observation = self.perception.observe(&context);
            match subscriber.observations.try_send(Perceived {
                revision,
                observation,
            }) {
                Ok(()) => {}
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
