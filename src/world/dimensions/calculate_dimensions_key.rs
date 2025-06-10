use crate::world::dimensions::Dimensions;

pub enum CalculateDimensionsKeyResult {
    Key(Vec<usize>),
    FailedDimension(usize),
}

impl Dimensions {
    fn calculate_dimensions_key(&self, expressed_values: &[f64]) -> CalculateDimensionsKeyResult {
        todo!();

        // let mut dimensions_key = Vec::new();
        // for (index, dimension) in self.dimensions.iter().enumerate() {
        //     match dimension.get_interval(expressed_values) {
        //         Some(interval) => dimensions_key.push(interval),
        //         None => return CalculateDimensionsKeyResult::FailedDimension(index),
        //     }
        // }
        // CalculateDimensionsKeyResult::Key(dimensions_key)
    }
}
