pub mod generate_random_phenotypes;
pub mod increment_ages;
pub mod new;

pub mod run_all;
pub mod update_all_region_keys;

pub use generate_random_phenotypes::generate_random_phenotypes;
pub use organism::Organism;
use std::rc::Rc;
pub mod best;
pub mod distinct_locations_count;
pub mod find_spacial_limits;
pub mod organism;

/// Represents a collection of `Organism` instances within the world.
// Collection wrapper providing convenience methods over a vector of Organism instances.
#[derive(Debug, Clone)]
pub struct Organisms {
    organisms: Vec<Rc<Organism>>,
}

impl Organisms {
    /// Returns an iterator over the organisms.
    pub fn iter(&self) -> std::slice::Iter<'_, Rc<Organism>> {
        self.organisms.iter()
    }

    /// Returns the number of organisms.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.organisms.len()
    }

    // Note: no mutable iterator needed since interior mutability
    // pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Rc<Organism>> { ... }

    /// Removes all organisms that have been marked as dead.
    ///
    /// This performs an in-place, linear `retain` scan, so the cost is
    /// O(live + dead) and no extra allocation is required.
    pub fn retain_live(&mut self) {
        self.organisms.retain(|o| !o.is_dead());
    }

    /// Adds a batch of organisms to the collection.
    pub fn extend(&mut self, mut others: Vec<Rc<Organism>>) {
        self.organisms.append(&mut others);
    }

    /// Creates an empty `Organisms` collection.
    pub fn new_empty() -> Self {
        Self {
            organisms: Vec::new(),
        }
    }

    /// Consumes the collection and returns the underlying vector.
    pub fn into_inner(self) -> Vec<Rc<Organism>> {
        self.organisms
    }
}

#[cfg(test)]
mod tests {
    use super::Organisms;

    use crate::world::organisms::Organism;

    use std::rc::Rc;

    impl Organisms {
        // Note: get_organisms was moved to the main impl block

        /// Creates a new `Organisms` instance directly from a vector of phenotypes.
        /// This is intended for testing purposes only.
        pub(crate) fn new_from_phenotypes(phenotypes: Vec<crate::phenotype::Phenotype>) -> Self {
            Self {
                organisms: phenotypes
                    .into_iter()
                    .map(|p| Rc::new(Organism::new(Rc::new(p), 0)))
                    .collect(),
            }
        }

        pub fn new_from_organisms(organisms: Vec<Organism>) -> Self {
            Self {
                organisms: organisms.into_iter().map(Rc::new).collect(),
            }
        }
    }
}
