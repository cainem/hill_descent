pub mod generate_random_phenotypes;
pub mod new;
pub mod update_all_region_keys;

pub use generate_random_phenotypes::generate_random_phenotypes;
pub use organism::Organism;
pub mod distinct_locations_count;
pub mod find_spacial_limits;
pub mod organism;

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

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Organism> {
        self.organisms.iter_mut()
    }

    /// Returns the number of organisms.
    pub fn count(&self) -> usize {
        self.organisms.len()
    }
}

#[cfg(test)]
impl Organisms {
    // Note: get_organisms was moved to the main impl block

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

    pub fn new_from_organisms(organisms: Vec<Organism>) -> Self {
        Self { organisms }
    }
}

impl Organisms {
    // This was moved from the #[cfg(test)] block to be generally available
    pub fn get_organisms(&self) -> &Vec<Organism> {
        &self.organisms
    }
}
