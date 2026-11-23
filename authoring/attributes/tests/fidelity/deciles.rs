//! Independent decile boundaries from profile parameters; never calls sampler quantile helpers.
use super::*;
use mineworld_attributes_model::{
    Traits, derive,
    profile::{Profile, Shape},
};
use mineworld_contracts::WorldTime;

fn value(
    catalogue: &Catalogue,
    traits: &Traits,
    id: &QualifiedId,
    at: WorldTime,
) -> Option<AttributeValue> {
    traits
        .value(id)
        .cloned()
        .or_else(|| derive::value(catalogue, catalogue.get(id)?, traits, at))
}

fn rounded(n: i128, d: i128) -> i128 {
    n.signum() * ((n.abs() + d / 2) / d)
}

pub(super) fn check(
    catalogue: &Catalogue,
    profile: &Profile,
    population: &[(String, Traits)],
    failures: &mut Vec<String>,
) {
    let at = WorldTime::from_seconds(999 * 60);
    for (id, distribution) in &profile.distributions {
        if matches!(distribution.shape, Shape::Prior { .. }) {
            continue;
        }
        let mut groups: BTreeMap<Vec<ValueSlug>, Vec<i128>> = BTreeMap::new();
        for (_, traits) in population {
            let parents: Option<Vec<_>> = distribution
                .given
                .iter()
                .map(|id| {
                    value(catalogue, traits, id, at).and_then(|v| ValueSlug::new(&slug_of(&v)).ok())
                })
                .collect();
            let Some(parents) = parents else {
                continue;
            };
            let position = match value(catalogue, traits, id, at) {
                Some(AttributeValue::Quantity(q)) => i128::from(q.value()),
                Some(AttributeValue::Instant(birth)) => i128::from(at.seconds() - birth.seconds()),
                _ => continue,
            };
            groups.entry(parents).or_default().push(position);
        }
        for (parents, values) in groups {
            if values.len() < 50 {
                continue;
            }
            // Standard-normal inverse CDF at 0.1,...,0.9, rounded to micro-units.
            const Z: [i128; 9] = [
                -1_281_552, -841_621, -524_401, -253_347, 0, 253_347, 524_401, 841_621, 1_281_552,
            ];
            let edges: Vec<i128> = (1..10)
                .map(|k| match &distribution.shape {
                    Shape::Normal { rows, .. } => {
                        let row = rows.row(&parents).unwrap();
                        i128::from(row.mean)
                            + rounded(Z[k - 1] * i128::from(row.p95 - row.p5), 3_289_707)
                    }
                    Shape::Uniform { min, max } => {
                        i128::from(*min) + k as i128 * i128::from(max - min + 1) / 10
                    }
                    Shape::AgeWithinBand { bands } => {
                        let domain = catalogue
                            .get(&distribution.given[0])
                            .unwrap()
                            .kind
                            .values()
                            .unwrap();
                        let index = domain.iter().position(|p| *p == parents[0]).unwrap();
                        let (lo, hi) = bands[index];
                        let days = |y| (i128::from(y) * 146_097 + 399) / 400;
                        (days(lo) + k as i128 * (days(hi) - days(lo)) / 10) * 86_400
                    }
                    Shape::Table { points } => {
                        let q = k as i128 * 100_000;
                        if q <= i128::from(points[0].0) {
                            return i128::from(points[0].1);
                        }
                        let last = points.last().unwrap();
                        if q >= i128::from(last.0) {
                            return i128::from(last.1);
                        }
                        let pair = points
                            .windows(2)
                            .find(|pair| q < i128::from(pair[1].0))
                            .unwrap();
                        let ((q0, v0), (q1, v1)) = (pair[0], pair[1]);
                        i128::from(v0)
                            + rounded(
                                i128::from(v1 - v0) * (q - i128::from(q0)),
                                i128::from(q1 - q0),
                            )
                    }
                    Shape::Prior { .. } => unreachable!(),
                })
                .collect();
            let mut bins = [0usize; 10];
            for v in &values {
                bins[edges.iter().filter(|edge| v >= edge).count()] += 1;
            }
            for (bin, count) in bins.into_iter().enumerate() {
                if (count * 100).abs_diff(values.len() * 10) > values.len() * 2 {
                    failures.push(format!("{} {id} {parents:?} decile{bin} count{count} N{} expected=10% threshold=2pp", profile.id, values.len()));
                }
            }
        }
    }
}
