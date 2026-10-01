//! Driving a Person with a controller, over the same seam a client's socket uses.
//!
//! ```text
//! WebSocket session   join(seat) ─► observations ─► submit(observer, request)   server/src/session.rs
//! this driver         join(seat) ─► observations ─► submit(observer, request)   and nothing else
//! ```
//!
//! Those are the same two calls on the same handle. Everything that makes a request authoritative is
//! on this side of the WebSocket: the seat is resolved from the **world's** roster, the observer comes
//! back from the world rather than being named, the actor check refuses a request that acts as
//! somebody else, the `ActionId` and the instant are allocated by the world, and the observation the
//! controller reads was computed for that observer and for nobody else. `INV-1` is therefore
//! demonstrated rather than asserted: the world cannot tell this Person from a played one.
//!
//! What this driver skips is the JSON framing and the TCP hop. That is a deliberate limit and it is
//! recorded as one in the PR ledger: it proves the agent is on the client *authority* path, not that
//! it is across the client *transport*. A controller in another process, speaking the wire protocol,
//! is a driver swap and no architectural change — and it would put a fourth terminal in front of a
//! demonstration whose whole point is that there is one server.
//!
//! # Why the composition root owns this
//!
//! `mineworld-server` must not depend on a controller any more than on a System Pack — a transport
//! that knew what a conversation was would be `ENGINEERING_STANDARDS.md` §4's one-way rule inverted —
//! and `mineworld-rule-controller` must not depend on the transport. So the wiring lives in the only
//! crate that depends on both, which is this binary.

use mineworld_contracts::{ActionResult, EntityKey};
use mineworld_rule_controller::RuleController;
use mineworld_server::WorldHost;

/// Occupies `seat` and acts on every observation it is sent, until the world stops.
///
/// Runs as its own task, so the world is never waiting for a controller's decision — the same
/// property a client's session has, and for the same reason: the world delivers observations with
/// `try_send` and answers requests without touching anybody.
///
/// A refusal to seat the controller ends the task with a message rather than the process. The world
/// is already hosting clients by then, and taking it down because one controller could not start
/// would be the transport deciding a world's fate.
pub async fn drive(host: WorldHost, seat: EntityKey) {
    let mut seated = match host.join(seat.clone()).await {
        Ok(seated) => seated,
        Err(refusal) => {
            eprintln!(
                "[mineworld] the agent could not occupy '{seat}': {:?}. The world is running \
                 without it.",
                refusal.code()
            );
            return;
        }
    };
    let observer = seated.observer();
    println!("[mineworld] agent: driving '{seat}' as entity {observer}");

    let mut controller = RuleController::new();
    // `None` means the world has stopped, which is the only way this loop ends — the same condition
    // that ends a client's stream.
    while let Some(perceived) = seated.observations().recv().await {
        let Some(request) = controller.decide(&perceived.observation) else {
            continue;
        };
        let target = request.target();
        match host.submit(observer, request).await {
            Ok(submitted) => match submitted.result() {
                ActionResult::Accepted { events } => println!(
                    "[mineworld] agent: {observer} -> {target:?} accepted as action {} ({} \
                     fact(s))",
                    submitted.action_id(),
                    events.len(),
                ),
                // Printed rather than retried: a rejection is the world's considered answer, and a
                // controller that argued with it would be a controller deciding a rule.
                other => println!("[mineworld] agent: {observer} -> {target:?} {other:?}"),
            },
            Err(refusal) => {
                eprintln!("[mineworld] agent: refused ({:?})", refusal.code());
            }
        }
    }
    host.leave(seated.subscription());
}
