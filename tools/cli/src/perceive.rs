//! Adapting a loaded world's perception onto the seam the server asks for.
//!
//! Two layers meet here and neither may know about the other:
//!
//! ```text
//! mineworld-server    asks: given this observer, what does it perceive?  (Perception)
//! systems/presence    answers it, for a world that has locations         (observe)
//! ```
//!
//! The server must not depend on a System Pack — a transport that knew what a conversation was would
//! be the one-way dependency rule inverted — and a System Pack must not depend on the transport. So
//! the adapter lives in the **composition root**, which is this binary: the only crate in the
//! workspace that depends on both.
//!
//! # What this adapter decides
//!
//! Nothing. It holds the providers the pack composed, calls
//! [`observe`](mineworld_presence::observe), and re-parameterizes the result onto the payload type
//! the wire carries. Every judgement about what an observer may know or may attempt is made inside
//! the perception system and the kernel's route map (`ENGINEERING_RULES.md` §8).

use mineworld_contracts::Observation;
use mineworld_presence::{InteractionProvider, observe};
use mineworld_server::{Perception, PerceptionContext, WireObservation};

/// What a loaded World Pack's observers perceive.
///
/// Holds the [`InteractionProvider`]s the pack composed, in the order it composed them, because that
/// order is the order affordances reach an observation and an observation that reordered between two
/// runs of one world would break replay comparison (`AC-12`).
pub struct PackPerception {
    providers: Vec<Box<dyn InteractionProvider>>,
}

impl PackPerception {
    /// Perceives a world through these packs.
    pub const fn new(providers: Vec<Box<dyn InteractionProvider>>) -> Self {
        Self { providers }
    }
}

impl Perception for PackPerception {
    fn observe(&self, context: &PerceptionContext<'_>) -> WireObservation {
        let borrowed: Vec<&dyn InteractionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        wire(observe(
            context.world(),
            context.observer(),
            context.at(),
            &borrowed,
        ))
    }
}

/// The same observation, on the payload type the transport carries.
///
/// `mineworld_presence::observe` builds an `Observation<Vec<u8>>` — the contract's default, where a
/// payload is opaque bytes — and the wire carries `Observation<serde_json::Value>`, because
/// `spike/FINDINGS.md` F8.2 measured the default reaching a client as an array of byte integers.
///
/// The conversion goes through the contract's own `serde` in both directions rather than a
/// hand-written mirror of its shapes, exactly as
/// [`into_kernel_request`](mineworld_server::protocol::into_kernel_request) does for the inbound
/// direction and for the same reason: a `ComponentRecord` cannot be rebuilt without naming its
/// component's Rust type, deliberately, so that a record's label cannot lie. Every identity survives
/// it as the decimal string the contract writes when the format is human-readable (`PR 04`).
///
/// **What this is exact about, and where it would stop being.** Today presence's observation carries
/// no component record and no event — its own documentation says why — so the round trip is lossless.
/// The day a perception system exposes a component through it, that record's payload arrives here as
/// bytes and would reach a client as F8.2's array of integers. The fix then belongs in the perception
/// system, which knows the component's type; this is the place that would have to change, and the PR
/// ledger records it as a follow-up rather than leaving it to be discovered.
///
/// A failure is impossible rather than unlikely: an `Observation` is a structure of integers, strings
/// and validated identifiers with no non-finite numbers and no non-string map keys, which are
/// `serde_json`'s only failure modes. The empty observation is the honest fallback — the same one the
/// server's default perception returns — rather than a panic that would take a running world down.
fn wire(observation: Observation) -> WireObservation {
    let observer = observation.observer();
    let at = observation.at();
    serde_json::to_value(&observation)
        .and_then(serde_json::from_value)
        .unwrap_or_else(|_| Observation::new(observer, at))
}
