use dimension::Dimension;
use organisms::Organisms;
use regions::Regions;

pub mod dimension;
pub mod organisms;
pub mod regions;

#[derive(Debug, Clone)]
pub struct World {
    _dimensions: Vec<Dimension>,
    _last_division_index: usize,
    _organisms: Organisms,
    _regions: Regions,
}
