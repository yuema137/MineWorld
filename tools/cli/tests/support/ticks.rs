//! CP-B4's bound on the world thread's tick, read from the server's shutdown statistics line.
//!
//! `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms, cpu p50 C ms, cpu p99 D ms`
//! (`docs/MODULE_SPEC.md` §8.1, `ARC-42`). The first three duration columns are wall time, the last two
//! the world thread's own CPU time.
//!
//! Two hard bounds, both half the 100 ms observation cadence (operator ruling of 2026-10-15 amending
//! D-SB12, F-tick-1; step-12 §16.11):
//!
//! - **CPU p99 ≤ 50 ms** — the tail of the work the tick itself does. A host that deschedules the world
//!   thread lengthens wall time and not CPU time (step-12 E-SB9), so runner load cannot fail it, and a
//!   tick that computes too long in more than one tick in a hundred does.
//! - **Wall p50 ≤ 50 ms** — the typical tick as a client feels it. It catches a tick that waits — a
//!   sleep, a blocking call, an fsync grown slow — which uses no CPU; one stalled tick in two is needed
//!   to move it, which no runner load has come near (every recorded p50 is under 1 ms).
//!
//! The wall p99 and the longest tick are printed beside them and do not gate. Accepted gap: an off-CPU
//! wait in fewer than half the ticks (the journal's fsync on a slow disk) is seen only in those advisory
//! columns; E-SB9 measured a commit at ≤ 5.1 ms under eight fsync-heavy neighbours.

/// Half the observation cadence (`HostConfig::default`'s 100 ms), in milliseconds.
pub const TICK_BUDGET_MS: f64 = 50.0;

/// One shutdown statistics line, parsed.
pub struct TickStats {
    line: String,
    pub wall_p50: f64,
    pub wall_p99: f64,
    pub longest: f64,
    pub cpu_p50: f64,
    pub cpu_p99: f64,
}

impl TickStats {
    /// Parses the `[world] ticks …` line; a missing column fails the test by name.
    pub fn parse(line: &str) -> Self {
        let field = |name: &str| -> f64 {
            line.split(", ")
                .find_map(|part| part.strip_prefix(name))
                .and_then(|rest| rest.strip_suffix(" ms"))
                .and_then(|number| number.parse().ok())
                .unwrap_or_else(|| panic!("{name}… ms in {line:?}"))
        };
        Self {
            wall_p50: field("p50 "),
            wall_p99: field("p99 "),
            longest: field("longest tick "),
            cpu_p50: field("cpu p50 "),
            cpu_p99: field("cpu p99 "),
            line: line.to_owned(),
        }
    }

    /// The numbers an operator reads, the advisory ones included.
    pub fn summary(&self) -> String {
        format!(
            "cpu p50 {} ms, cpu p99 {} ms; wall p50 {} ms; advisory: wall p99 {} ms, max {} ms ({})",
            self.cpu_p50, self.cpu_p99, self.wall_p50, self.wall_p99, self.longest, self.line
        )
    }

    /// CP-B4: the CPU p99 and the wall p50 within the budget.
    pub fn assert_within_budget(&self) {
        assert!(
            self.cpu_p99 <= TICK_BUDGET_MS,
            "the p99 tick's CPU time was {} ms, over {TICK_BUDGET_MS} ms — {}",
            self.cpu_p99,
            self.summary()
        );
        assert!(
            self.wall_p50 <= TICK_BUDGET_MS,
            "the median tick took {} ms of wall time, over {TICK_BUDGET_MS} ms — {}",
            self.wall_p50,
            self.summary()
        );
    }
}
