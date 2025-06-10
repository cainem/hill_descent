use std::ops::{Range, RangeInclusive};

pub mod get_interval;
pub mod new;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, PartialEq)]
pub enum IntervalType {
    Standard(Range<f64>),
    EndOfRange(RangeInclusive<f64>),
}
