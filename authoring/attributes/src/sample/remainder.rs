//! Largest remainder (`pr-s30d-sampler.md` §5.5): integer shares of a whole, each within one unit of its
//! exact share, the units left over going to the largest remainders, ties broken by a stated order.

/// `total` split in proportion to `weights` (`Σ weights > 0`): `base = total·w div W`, then one more
/// each to the largest `total·w mod W`, ties by the larger `tie`, then the lower index.
pub fn largest_remainder(total: u128, weights: &[u128], tie: impl Fn(usize) -> u64) -> Vec<u128> {
    let sum: u128 = weights.iter().sum();
    if sum == 0 {
        return vec![0; weights.len()];
    }
    let mut shares: Vec<u128> = weights.iter().map(|weight| total * weight / sum).collect();
    let given: u128 = shares.iter().sum();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by(|a, b| {
        let (ra, rb) = (total * weights[*a] % sum, total * weights[*b] % sum);
        rb.cmp(&ra)
            .then_with(|| tie(*b).cmp(&tie(*a)))
            .then_with(|| a.cmp(b))
    });
    let left = usize::try_from(total - given).unwrap_or(usize::MAX);
    for index in order.into_iter().take(left) {
        shares[index] += 1;
    }
    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seven people in thirds: 2, 2, 2 and one left over, to the tie's choice.
    #[test]
    fn the_leftover_goes_to_the_largest_remainder_ties_by_tie_then_index() {
        assert_eq!(largest_remainder(7, &[1, 1, 1], |_| 0), [3, 2, 2]);
        assert_eq!(
            largest_remainder(7, &[1, 1, 1], |index| index as u64),
            [2, 2, 3]
        );
        assert_eq!(largest_remainder(10, &[5, 3, 2], |_| 0), [5, 3, 2]);
        assert_eq!(
            largest_remainder(1_000_000, &[1, 2], |_| 0),
            [333_333, 666_667]
        );
        assert_eq!(largest_remainder(4, &[0, 0], |_| 0), [0, 0]);
    }
}
