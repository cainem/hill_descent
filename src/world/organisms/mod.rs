pub mod generate_random_phenotypes;
pub mod new;
pub mod update_all_region_keys;

pub use generate_random_phenotypes::generate_random_phenotypes;
pub mod distinct_locations_count;
pub mod find_spacial_limits;
pub mod organism;

#[allow(unused_imports)]
use crate::{Phenotype, world::organisms::organism::Organism};

/// Represents a collection of `Organism` instances within the world.
#[derive(Debug, Clone)]
pub struct Organisms {
    organisms: Vec<Organism>,
}

impl Organisms {
    /// Returns an iterator over the organisms.
    pub fn iter(&self) -> std::slice::Iter<'_, Organism> {
        self.organisms.iter()
    }

    /// Returns the number of organisms.
    pub fn count(&self) -> usize {
        self.organisms.len()
    }
}

#[cfg(test)]
impl Organisms {
    /// Creates a new `Organisms` instance directly from a vector of phenotypes.
    /// This is intended for testing purposes only.
    pub(crate) fn new_from_phenotypes(phenotypes: Vec<crate::phenotype::Phenotype>) -> Self {
        use std::rc::Rc;
        Self {
            organisms: phenotypes
                .into_iter()
                .map(|p| Organism::new(Rc::new(p)))
                .collect(),
        }
    }
}
