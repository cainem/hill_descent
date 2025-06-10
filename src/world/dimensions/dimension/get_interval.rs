use crate::world::dimensions::dimension::{Dimension, IntervalType};

impl Dimension {
    pub fn get_interval(&self, value: f64) -> Option<usize> {
        let range = self.range();
        let divisions = self.number_of_divisions();

        if value < *range.start() || value > *range.end() {
            return None; // Value is out of bounds
        }

        let step = (range.end() - range.start()) / divisions as f64;
        let index = ((value - *range.start()) / step).floor() as usize;

        if index >= divisions {
            Some(divisions - 1) // Return the last index if it exceeds the range
        } else {
            Some(index)
        }
    }
}