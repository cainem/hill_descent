use std::ops::RangeInclusive;

pub mod expand_bounds;
pub mod get_interval;
pub mod interval_bounds;
pub mod new;

#[derive(Debug, Clone)]
// Represents a single axis in the simulation space with its bounds and doubling count.
pub struct Dimension {
    range: RangeInclusive<f64>,
    number_of_doublings: usize,
}

impl Dimension {
    /// Returns the range of the dimension.
    pub fn range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    /// Returns the number of times the dimension has been doubled (split in half).
    pub fn number_of_doublings(&self) -> usize {
        self.number_of_doublings
    }

    /// Sets the number of times the dimension has been doubled (split in half).
    pub fn set_number_of_doublings(&mut self, new_doublings: usize) {
        self.number_of_doublings = new_doublings;
    }

    /// Returns the number of intervals the dimension is divided into.
    pub fn num_intervals(&self) -> f64 {
        // Number of intervals is 2^d, where d is the number of doublings.
        2.0_f64.powi(self.number_of_doublings as i32)
    }

    /// Sets the range of the dimension.
    pub fn set_range(&mut self, new_range: RangeInclusive<f64>) {
        assert!(
            *new_range.end() >= *new_range.start(),
            "Dimension max must be greater than or equal to min. Start: {}, End: {}",
            new_range.start(),
            new_range.end()
        );
        self.range = new_range;
    }
}
