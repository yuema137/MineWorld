//! The levels a relationship is named by — what a biography and a client can say, derived from values.

use serde::{Deserialize, Serialize};

/// How well one person knows and regards another, as a name (`step-09-social.md` SD-7).
///
/// Ordered, so a crossing has a direction. Derived from the values, never stored: the values are the
/// state, the level is a reading of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Known, and no more.
    Acquaintance,
    /// Familiarity at least 300 and regard at least 100.
    Friendly,
    /// Familiarity at least 600 and regard at least 300.
    Friend,
    /// Familiarity at least 900 and regard at least 600.
    Close,
}

/// The level these values read as. Both thresholds must hold: knowing somebody well and thinking
/// little of them is not friendship.
pub const fn level(familiarity: i32, regard: i32) -> Level {
    if familiarity >= 900 && regard >= 600 {
        Level::Close
    } else if familiarity >= 600 && regard >= 300 {
        Level::Friend
    } else if familiarity >= 300 && regard >= 100 {
        Level::Friendly
    } else {
        Level::Acquaintance
    }
}

#[cfg(test)]
mod tests {
    use super::{Level, level};

    /// Each boundary, one either side, as SD-7's literals.
    #[test]
    fn every_boundary_is_where_sd_7_puts_it() {
        for (familiarity, regard, expected) in [
            (299, 1_000, Level::Acquaintance),
            (300, 99, Level::Acquaintance),
            (300, 100, Level::Friendly),
            (599, 1_000, Level::Friendly),
            (600, 299, Level::Friendly),
            (600, 300, Level::Friend),
            (899, 1_000, Level::Friend),
            (900, 599, Level::Friend),
            (900, 600, Level::Close),
            (1_000, 1_000, Level::Close),
            (1_000, -1_000, Level::Acquaintance),
        ] {
            assert_eq!(
                level(familiarity, regard),
                expected,
                "familiarity {familiarity}, regard {regard}"
            );
        }
    }
}
