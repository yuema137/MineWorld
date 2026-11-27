//! What a guide run reports (`docs/GUIDE_FORMAT.md` §11).

use std::time::Duration;

/// One check's verdict. A pending or inconclusive item is reported by name and is never counted as passed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The check ran and held.
    Pass,
    /// The check ran and did not hold; the text is what a writer must fix.
    Fail(String),
    /// The check could not be run in this build, and the text says why. It neither passes nor fails.
    Inconclusive(String),
}

/// A whole guide run: how many blocks passed, which blocks are pending and for what, and every check.
#[derive(Debug, Default)]
pub struct Summary {
    /// The number of executed blocks the pages hold, whatever became of them.
    pub blocks: usize,
    /// The number of executed blocks that ran and matched.
    pub passed: usize,
    /// The blocks waiting for a feature or an operator, each named by page, step and feature.
    pub pending: Vec<String>,
    /// The same blocks as `page | step | reason`, with no line number: the form the real guide's committed
    /// list of blocks that are not run is written in (§11.1).
    pub not_run: Vec<String>,
    /// The shell blocks written for another operating system's shell, by page and step (§4.6).
    pub other_os: Vec<String>,
    /// Every failed block, and every guard, with its name and verdict.
    pub checks: Vec<(String, Verdict)>,
    /// Each page that ran, in run order, with its wall time (§11 item 5).
    pub timings: Vec<(String, Duration)>,
    /// Each executed block, in run order: its page, step, line, wall time and outcome word (`PASS`, `FAIL`,
    /// `PENDING`, `OTHER OS`). A cold-start report splits page 01 into these.
    pub block_times: Vec<BlockTime>,
}

/// One executed block's time, for the cold-start report (§4.7).
#[derive(Clone, Debug)]
pub struct BlockTime {
    pub page: String,
    pub step: String,
    pub line: usize,
    pub elapsed: Duration,
    pub outcome: &'static str,
}

impl Summary {
    /// Whether nothing failed. Pending and inconclusive items do not fail a run, but they are listed by
    /// name and are never counted as passed.
    pub fn no_failures(&self) -> bool {
        !self
            .checks
            .iter()
            .any(|(_, verdict)| matches!(verdict, Verdict::Fail(_)))
    }

    /// The report a person reads: the counts, then every pending item, then every check that did not pass.
    pub fn render(&self) -> String {
        let mut lines = vec![format!(
            "guide: {} block(s), {} passed, {} pending, {} for another system's shell, {} check(s)",
            self.blocks,
            self.passed,
            self.pending.len(),
            self.other_os.len(),
            self.checks.len()
        )];
        lines.extend(self.pending.iter().map(|item| format!("PENDING  {item}")));
        lines.extend(self.other_os.iter().map(|item| format!("OTHER OS {item}")));
        for (name, verdict) in &self.checks {
            match verdict {
                Verdict::Pass => {}
                Verdict::Fail(reason) => lines.push(format!("FAIL     {name}\n{reason}")),
                Verdict::Inconclusive(reason) => {
                    lines.push(format!("INCONCLUSIVE  {name}: {reason}"));
                }
            }
        }
        if !self.timings.is_empty() {
            let pages: Vec<String> = self
                .timings
                .iter()
                .map(|(page, time)| format!("{page} {:.1}s", time.as_secs_f64()))
                .collect();
            lines.push(format!("timings: {}", pages.join(", ")));
        }
        lines.join("\n")
    }
}
