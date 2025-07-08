use std::ops::RangeInclusive;

pub mod expand_bounds;
pub mod get_interval;
pub mod new;

#[derive(Debug, Clone)]
// Represents a single axis in the simulation space with its bounds and division count.
pub struct Dimension {
    range: RangeInclusive<f64>,
    number_of_divisions: usize,
}

impl Dimension {
    /// Returns the range of the dimension.
    pub fn range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    /// Returns the number of times the dimension has been divided.
    pub fn number_of_divisions(&self) -> usize {
        self.number_of_divisions
    }

    /// Sets the number of times the dimension has been divided.
    pub fn set_number_of_divisions(&mut self, new_divisions: usize) {
        self.number_of_divisions = new_divisions;
    }

    /// Returns the number of intervals the dimension is divided into.
    pub fn num_intervals(&self) -> usize {
        // Number of intervals is 2^d, where d is the number of divisions.
        2_usize.pow(self.number_of_divisions as u32)
    }
}
