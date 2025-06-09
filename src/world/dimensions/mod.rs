use crate::world::dimensions::dimension::Dimension;

pub mod dimension;
pub mod new;
pub mod double;

#[derive(Debug, Clone)]
pub struct Dimensions {
    _dimensions: Vec<Dimension>,
    _last_division_index: usize,
}

impl Dimensions {
    pub fn get_dimensions(&self) -> &Vec<Dimension> {
        &self._dimensions
    }

    pub fn get_last_division_index(&self) -> usize {
        self._last_division_index
    }
}
