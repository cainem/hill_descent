use super::Dimension;

impl Dimension {
    /// Expands the dimension's range by 50% on each side.
    pub fn expand_bounds(&mut self) {
        let start = *self.range.start();
        let end = *self.range.end();
        let width = end - start;

        if width == 0.0 {
            // If the range has no width, expand by a fixed amount.
            self.range = (start - 0.5)..=(end + 0.5);
        } else {
            let expansion = width / 2.0;
            self.range = (start - expansion)..=(end + expansion);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::world::dimensions::dimension::Dimension;

    #[test]
    fn given_dimension_with_zero_width_when_expand_bounds_then_range_expands_by_fixed_amount() {
        let mut dimension = Dimension::new(0.0..=0.0, 0);
        dimension.expand_bounds();
        assert_eq!(*dimension.range(), -0.5..=0.5);
    }

    #[test]
    fn given_dimension_with_non_zero_width_when_expand_bounds_then_range_expands_by_50_percent() {
        let mut dimension = Dimension::new(10.0..=20.0, 0);
        dimension.expand_bounds();
        assert_eq!(*dimension.range(), 5.0..=25.0);
    }
}
