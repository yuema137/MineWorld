//! A response curve: a piecewise-linear map from a consideration's input to a per-mille output, in integers
//! (`pr-dl-b-motivated.md` SD-DLb-2 … SD-DLb-4, §5.3).

use core::fmt;

/// The most points a curve may have (SD-DLb-7): one for each hour of a day, plus room for the ends.
pub const POINTS_AT_MOST: usize = 24;

/// The largest output of a curve, in per-mille: 1 000 is the top of the scale (`1 000 = 100 %`).
pub const SCALE: u32 = 1_000;

/// A piecewise-linear curve. Its points are `(x, y)` with `x` strictly increasing and `y` in `0 ..= SCALE`.
///
/// Evaluated by integer interpolation with floor division, and clamped to the first and last points outside
/// their `x` range. A curve is a value: it holds no state, so evaluating it twice gives the same answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Curve {
    points: Vec<(i64, u32)>,
}

/// Why a list of points is not a curve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurveError {
    /// No points at all.
    Empty,
    /// More than [`POINTS_AT_MOST`] points.
    TooMany(usize),
    /// The point at this index does not have an `x` greater than the one before it.
    NotIncreasing(usize),
    /// The point at this index has a `y` above [`SCALE`].
    OutOfScale(usize, u32),
}

impl fmt::Display for CurveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a curve needs at least one point"),
            Self::TooMany(count) => write!(
                formatter,
                "a curve has at most {POINTS_AT_MOST} points, not {count}"
            ),
            Self::NotIncreasing(index) => write!(
                formatter,
                "point {index} does not have a greater x than the point before it"
            ),
            Self::OutOfScale(index, value) => write!(
                formatter,
                "point {index} has y {value}, outside 0 to {SCALE}"
            ),
        }
    }
}

impl Curve {
    /// A curve through `points`.
    ///
    /// # Errors
    ///
    /// No points, more than [`POINTS_AT_MOST`], an `x` that does not strictly increase, or a `y` above
    /// [`SCALE`].
    pub fn new(points: Vec<(i64, u32)>) -> Result<Self, CurveError> {
        if points.is_empty() {
            return Err(CurveError::Empty);
        }
        if points.len() > POINTS_AT_MOST {
            return Err(CurveError::TooMany(points.len()));
        }
        for (index, &(x, y)) in points.iter().enumerate() {
            if y > SCALE {
                return Err(CurveError::OutOfScale(index, y));
            }
            if index > 0 && x <= points[index - 1].0 {
                return Err(CurveError::NotIncreasing(index));
            }
        }
        Ok(Self { points })
    }

    /// The points, in order.
    pub fn points(&self) -> &[(i64, u32)] {
        &self.points
    }

    /// The curve's output at `x`, in per-mille.
    ///
    /// Between two points the output is `y0 + floor((y1 - y0) × (x - x0) / (x1 - x0))`; outside the points it is
    /// the nearest end's `y`.
    pub fn at(&self, x: i64) -> u32 {
        let (first_x, first_y) = self.points[0];
        let (last_x, last_y) = self.points[self.points.len() - 1];
        if x <= first_x {
            return first_y;
        }
        if x >= last_x {
            return last_y;
        }
        // The first point at or beyond `x`: strictly after the first point, so there is a segment before it.
        let end = self.points.partition_point(|&(px, _)| px < x);
        let (x0, y0) = self.points[end - 1];
        let (x1, y1) = self.points[end];
        let span = x1 - x0;
        let rise = i64::from(y1) - i64::from(y0);
        let value = i64::from(y0) + (rise * (x - x0)).div_euclid(span);
        u32::try_from(value)
            .expect("an interpolation between two points in 0 to 1000 stays in that range")
    }
}
