use crate::world::dimensions::dimension::Dimension;

pub mod dimension;
pub mod double_regions;
pub mod new;

#[derive(Debug, Clone)]
pub struct Dimensions {
    dimensions: Vec<Dimension>,
    last_division_index: usize,
}

impl Dimensions {
    pub fn get_dimensions(&self) -> &Vec<Dimension> {
        &self.dimensions
    }

    pub fn get_last_division_index(&self) -> usize {
        self.last_division_index
    }
}
