use std::ops::{Range, RangeInclusive};

pub mod get_intervals;
pub mod new;

#[derive(Debug, Clone)]
pub struct Dimension {
    range: RangeInclusive<f64>,
    number_of_divisions: usize,
}

impl Dimension {
    // Getter for the 'range' field
    pub fn range(&self) -> &RangeInclusive<f64> {
        &self.range
    }

    // Getter for the 'number_of_divisions' field
    pub fn number_of_divisions(&self) -> usize {
        self.number_of_divisions
    }
}

#[derive(Debug, PartialEq)]
pub enum IntervalType {
    Standard(Range<f64>),
    EndOfRange(RangeInclusive<f64>),
}
