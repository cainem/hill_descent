use std::ops::{Range, RangeInclusive};

pub mod get_intervals;
pub mod new;

#[derive(Debug, Clone)]
pub struct Dimension {
    range: RangeInclusive<f64>,
    number_of_divisions: usize,
}

#[derive(Debug, PartialEq)]
pub enum IntervalType {
    Standard(Range<f64>),
    EndOfRange(RangeInclusive<f64>),
}
