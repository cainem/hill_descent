use std::ops::RangeInclusive;

use crate::world::dimensions::dimension::Dimension;

pub mod dimension;

#[derive(Debug, Clone)]
pub struct Dimensions {
    _dimensions: Vec<Dimension>,
    _last_division_index: usize,
}

impl Dimensions {
    pub fn new(_limits: Vec<RangeInclusive<f64>>) -> Self {
        todo!();
    }

    pub fn get_dimensions(&self) -> &Vec<dimension::Dimension> {
        &self._dimensions
    }

    pub fn get_last_division_index(&self) -> usize {
        self._last_division_index
    }
}
