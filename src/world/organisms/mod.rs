pub mod generate_random_phenotypes;
pub mod new;
pub mod update_all_dimensions_keys;

pub use generate_random_phenotypes::generate_random_phenotypes;
pub mod distinct_locations_count;
pub mod find_spacial_limits;

use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Organisms {
    organisms: Vec<Phenotype>,
}

#[cfg(test)]
impl Organisms {
    /// Creates a new `Organisms` instance directly from a vector of phenotypes.
    /// This is intended for testing purposes only.
    pub(crate) fn new_from_phenotypes(phenotypes: Vec<Phenotype>) -> Self {
        Self {
            organisms: phenotypes,
        }
    }
}
