//! How a hosted world's pacing starts, and what the host journal records of it (`ARC-69`, items 6 to 8).
//!
//! Pure: this module reads a journal and a choice and answers where the host clock starts and at what
//! scale. It writes nothing; the runtime writes the records (`runtime.rs`, `control.rs`).
//!
//! ```text
//! scale    --time-scale                              that scale
//!          else the journal's newest Started or Scaled with scale ≤ 24     from the save
//!          else world.yaml hosting.time_scale                              from world.yaml
//!          else 1                                                          by default
//! instant  max(world.now(), the newest Stopped record's instant)           a persisted world
//!          the epoch                                                       a world that is not persisted
//! ```
//!
//! A fast-forward scale (above 24×) is never taken from the journal, so it never survives a restart.

use std::fmt;
use std::num::NonZeroU32;

use mineworld_contracts::WorldTime;
use mineworld_persistence::{HostRecord, PacingChange};

use crate::host::TimeScaleChoice;

/// The largest scale a world may be restarted at from its own history: a day in 24 hours (`ARC-69` item 5).
pub(crate) const DAY_LENGTH_SCALE: u32 = 24;

/// Where the scale a world starts at came from, as the startup line names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScaleSource {
    /// `--time-scale`.
    Flag,
    /// The save's host journal.
    Save,
    /// `world.yaml`'s `hosting.time_scale`.
    Pack,
    /// Nothing said: 1.
    Default,
}

impl fmt::Display for ScaleSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Flag => "from --time-scale",
            Self::Save => "from the save",
            Self::Pack => "from world.yaml",
            Self::Default => "by default",
        })
    }
}

/// Where a hosted world's host clock starts: the instant, and the scale in force from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Start {
    pub(crate) at: WorldTime,
    pub(crate) scale: NonZeroU32,
    pub(crate) source: ScaleSource,
}

/// Resolves a start (`ARC-69` items 8 and 9). `now` is the world's own instant: the epoch for a world that is
/// not persisted, and the save's head for one that is; `journal` is empty for the former.
pub(crate) fn resolve(choice: TimeScaleChoice, journal: &[HostRecord], now: WorldTime) -> Start {
    let stopped = journal
        .iter()
        .filter(|record| matches!(record.change, PacingChange::Stopped))
        .map(|record| record.at)
        .max();
    let at = stopped.map_or(now, |stopped| stopped.max(now));
    if let Some(scale) = choice.flag {
        return Start {
            at,
            scale,
            source: ScaleSource::Flag,
        };
    }
    let saved = journal.iter().rev().find_map(|record| match record.change {
        PacingChange::Started { scale, .. } | PacingChange::Scaled { to: scale }
            if scale.get() <= DAY_LENGTH_SCALE =>
        {
            Some(scale)
        }
        _ => None,
    });
    if let Some(scale) = saved {
        return Start {
            at,
            scale,
            source: ScaleSource::Save,
        };
    }
    if let Some(scale) = choice.pack {
        return Start {
            at,
            scale,
            source: ScaleSource::Pack,
        };
    }
    Start {
        at,
        scale: NonZeroU32::MIN,
        source: ScaleSource::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn non_zero(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).expect("non-zero")
    }

    fn record(at: i64, change: PacingChange) -> HostRecord {
        HostRecord {
            at: WorldTime::from_seconds(at),
            revision: mineworld_persistence::WorldRevision::GENESIS,
            change,
        }
    }

    fn started(scale: u32) -> PacingChange {
        PacingChange::Started {
            scale: non_zero(scale),
            paused: false,
        }
    }

    /// Every row of step-19 §5.5's table, one case each.
    #[test]
    fn the_scale_is_the_flag_then_the_save_then_the_pack_then_one() {
        let flag = TimeScaleChoice {
            flag: Some(non_zero(6)),
            pack: Some(non_zero(12)),
        };
        let pack = TimeScaleChoice {
            flag: None,
            pack: Some(non_zero(12)),
        };
        let none = TimeScaleChoice::default();
        let now = WorldTime::from_seconds(100);
        let saved = [
            record(0, started(12)),
            record(50, PacingChange::Scaled { to: non_zero(6) }),
        ];

        let given = resolve(flag, &saved, now);
        assert_eq!(
            (given.scale.get(), given.source),
            (6, ScaleSource::Flag),
            "the flag wins"
        );
        let from_save = resolve(pack, &saved, now);
        assert_eq!(
            (from_save.scale.get(), from_save.source),
            (6, ScaleSource::Save),
            "the save's newest day-length scale beats the pack's default"
        );
        let from_pack = resolve(pack, &[], now);
        assert_eq!(
            (from_pack.scale.get(), from_pack.source),
            (12, ScaleSource::Pack)
        );
        let default = resolve(none, &[], now);
        assert_eq!(
            (default.scale.get(), default.source),
            (1, ScaleSource::Default)
        );
    }

    /// M-TWc-9: a fast-forward scale the save holds is never restored; the newest day-length one is.
    #[test]
    fn a_fast_forward_scale_in_the_save_never_survives_a_restart() {
        let journal = [
            record(0, started(12)),
            record(
                10,
                PacingChange::Scaled {
                    to: non_zero(3_600),
                },
            ),
        ];
        let restart = resolve(
            TimeScaleChoice::default(),
            &journal,
            WorldTime::from_seconds(20),
        );
        assert_eq!(
            (restart.scale.get(), restart.source),
            (12, ScaleSource::Save),
            "the 3 600× the save was last at is not restored; its 12× is"
        );
        let only_fast = [record(0, started(3_600))];
        let restart = resolve(
            TimeScaleChoice::default(),
            &only_fast,
            WorldTime::from_seconds(20),
        );
        assert_eq!(
            restart.scale.get(),
            1,
            "no day-length scale in the save: the default"
        );
    }

    /// M-TWc-5: a restart resumes no earlier than the last stop, so the HUD never runs backwards.
    #[test]
    fn a_restart_resumes_no_earlier_than_the_last_stop() {
        let journal = [
            record(0, started(12)),
            record(500, PacingChange::Stopped),
            record(900, started(12)),
        ];
        let restart = resolve(
            TimeScaleChoice::default(),
            &journal,
            WorldTime::from_seconds(100),
        );
        assert_eq!(
            restart.at,
            WorldTime::from_seconds(500),
            "the stop at 500 is later than the world's own 100"
        );
        let saved_past_stop = resolve(
            TimeScaleChoice::default(),
            &journal,
            WorldTime::from_seconds(700),
        );
        assert_eq!(saved_past_stop.at, WorldTime::from_seconds(700));
    }
}
