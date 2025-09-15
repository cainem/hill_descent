use crate::world::dimensions::dimension::Dimension;

impl Dimension {
    /// Returns the [start, end] bounds of the given 0-indexed interval.
    ///
    /// Semantics match `get_interval`:
    /// - Intervals partition the range into `2^doublings` chunks.
    /// - All intervals are half-open [start, end) except the last which is closed [start, end].
    /// - A single-point range or zero doublings yields exactly one interval [range.start, range.end].
    ///
    /// Returns `None` if `interval_index` is outside `[0, num_intervals)`, which
    /// should not happen when the index comes from `get_interval`.
    pub fn interval_bounds(&self, interval_index: usize) -> Option<(f64, f64)> {
        let range = self.range();
        let start = *range.start();
        let end = *range.end();

        // Number of intervals is 2^doublings. With 0 doublings it's 1.
        let num_intervals_f = self.num_intervals();
        let num_intervals = num_intervals_f as usize;

        // Guard against invalid indices
        if interval_index >= num_intervals {
            return None;
        }

        // Single-point range: regardless of doublings, only interval 0 is valid
        if start == end {
            return if interval_index == 0 {
                Some((start, end))
            } else {
                None
            };
        }

        // Zero doublings means exactly one interval; only index 0 is valid
        if num_intervals == 1 {
            return if interval_index == 0 {
                Some((start, end))
            } else {
                None
            };
        }

        let interval_size = (end - start) / num_intervals_f;
        // Compute start for this interval via scaled offset
        let i_start = start + (interval_index as f64) * interval_size;
        // Last interval closes on the exact `end` to avoid precision gaps
        let i_end = if interval_index + 1 == num_intervals {
            end
        } else {
            i_start + interval_size
        };

        Some((i_start, i_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dim(range: std::ops::RangeInclusive<f64>, d: usize) -> Dimension {
        Dimension::new(range, d)
    }

    #[test]
    fn given_zero_doublings_when_interval_bounds_then_full_range_returned() {
        let d = dim(0.0..=10.0, 0); // 1 interval
        assert_eq!(d.interval_bounds(0), Some((0.0, 10.0)));
        assert_eq!(d.interval_bounds(1), None);
    }

    #[test]
    fn given_single_point_range_when_interval_bounds_then_single_bounds_returned() {
        let d = dim(5.0..=5.0, 5); // still a single interval
        assert_eq!(d.interval_bounds(0), Some((5.0, 5.0)));
        assert_eq!(d.interval_bounds(1), None);
    }

    #[test]
    fn given_basic_divisions_when_interval_bounds_then_correct_bounds_for_each_interval() {
        // 0..=10 with 2 doublings => 4 intervals of size 2.5
        let d = dim(0.0..=10.0, 2);
        assert_eq!(d.interval_bounds(0), Some((0.0, 2.5)));
        assert_eq!(d.interval_bounds(1), Some((2.5, 5.0)));
        assert_eq!(d.interval_bounds(2), Some((5.0, 7.5)));
        // last interval must close on exact end
        assert_eq!(d.interval_bounds(3), Some((7.5, 10.0)));
        assert_eq!(d.interval_bounds(4), None);
    }

    #[test]
    fn given_last_interval_when_interval_bounds_then_end_is_exact_range_end() {
        // -10..=-5 with 5 doublings => 32 intervals
        let d = dim(-10.0..=-5.0, 5);
        let n = d.num_intervals() as usize;
        // last index
        let (_, e_last) = d.interval_bounds(n - 1).unwrap();
        assert_eq!(e_last, -5.0);
        // first index
        let (s0, _) = d.interval_bounds(0).unwrap();
        assert_eq!(s0, -10.0);
    }

    #[test]
    fn given_out_of_range_index_when_interval_bounds_then_none() {
        let d = dim(0.0..=1.0, 3); // 8 intervals
        assert_eq!(d.interval_bounds(8), None);
        assert_eq!(d.interval_bounds(100), None);
    }
}
