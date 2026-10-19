//! Micro-units and named rounding (`pr-s30d-sampler.md` §5.2): a probability, a ratio or a numeric
//! parameter is decoded once, by the exact decimal decoder, to millionths; after that nothing is a float.

use crate::value::{DecimalError, ScaledInt};

/// One million: a probability of 1 in micro-units.
pub const MILLION: i64 = 1_000_000;

/// The decimal `text` in millionths (at most 6 decimals).
///
/// # Errors
///
/// When `text` is not a decimal, has more than 6 decimals, or overflows.
pub fn micro(text: &str) -> Result<i64, DecimalError> {
    ScaledInt::parse(text, 6).map(ScaledInt::value)
}

/// `n / d` rounded half away from zero; `d > 0`.
pub fn round_half_away(n: i128, d: i128) -> i128 {
    let twice = 2 * n + n.signum() * d;
    twice / (2 * d)
}

/// `n / d` rounded up, for non-negative `n` and positive `d`.
pub const fn ceil_div(n: i128, d: i128) -> i128 {
    (n + d - 1).div_euclid(d)
}

/// `value` (millionths) as a decimal with 6 places.
pub fn micro_text(value: i64) -> String {
    ScaledInt::new(value, 6).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M-d-1's oracle: halves go away from zero, and everything else to the nearest.
    #[test]
    fn rounding_is_half_away_from_zero() {
        assert_eq!(round_half_away(5, 2), 3);
        assert_eq!(round_half_away(-5, 2), -3);
        assert_eq!(round_half_away(4, 3), 1);
        assert_eq!(round_half_away(5, 3), 2);
        assert_eq!(round_half_away(-4, 3), -1);
        assert_eq!(ceil_div(7, 2), 4);
        assert_eq!(ceil_div(6, 2), 3);
        assert_eq!(micro("0.4986"), Ok(498_600));
        assert!(micro("0.1234567").is_err());
        assert_eq!(micro_text(1_500_000), "1.500000");
    }
}
