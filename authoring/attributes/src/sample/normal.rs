//! The standard-normal quantile of a rational, by integer interpolation in the committed table
//! (`pr-s30d-sampler.md` §5.6).

use super::micro::{MILLION, round_half_away};
use super::normal_table::Z;

/// z(a / b) in micro-units, for `0 < a < b`: linear between `Z[k]` and `Z[k+1]` at `k = 1000a div b`;
/// clamped to z(0.001) and z(0.999) outside the table; then clipped to `± clip_sd` (1 … 3).
pub fn z(a: u128, b: u128, clip_sd: u8) -> i64 {
    let n = 1000 * a;
    let k = n / b;
    let f = n % b;
    let at = |k: u128| i128::from(Z[usize::try_from(k - 1).unwrap_or(0)]);
    let value = if k < 1 {
        at(1)
    } else if k >= 999 {
        at(999)
    } else {
        let (low, high) = (at(k), at(k + 1));
        low + round_half_away(
            (high - low) * i128::try_from(f).unwrap_or(0),
            i128::try_from(b).unwrap_or(1),
        )
    };
    let clip = i128::from(clip_sd) * i128::from(MILLION);
    i64::try_from(value.clamp(-clip, clip)).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_is_the_published_quantiles_symmetric_and_increasing() {
        assert_eq!(Z[975 - 1], 1_959_964);
        assert_eq!(Z[950 - 1], 1_644_854);
        assert_eq!(Z[900 - 1], 1_281_552);
        assert_eq!(Z[500 - 1], 0);
        for k in 1..1000 {
            assert_eq!(Z[k - 1], -Z[1000 - k - 1], "{k}");
        }
        assert!(Z.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn a_rational_quantile_interpolates_and_clips() {
        assert_eq!(z(1, 2, 3), 0);
        assert_eq!(z(975, 1000, 3), 1_959_964);
        // Halfway between k = 975 and 976.
        let half = i64::from(Z[974]) + ((i64::from(Z[975]) - i64::from(Z[974])) + 1) / 2;
        assert_eq!(z(1951, 2000, 3), half);
        assert_eq!(z(1, 1_000_000, 2), -2_000_000);
        assert_eq!(z(999_999, 1_000_000, 3), 3_000_000);
    }
}
