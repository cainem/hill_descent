use std::sync::Arc;

pub mod generate_random_phenotypes;
pub mod increment_ages;
pub mod new;

pub mod run_all;
pub mod update_all_region_keys;

pub use generate_random_phenotypes::generate_random_phenotypes;
pub use organism::Organism;
pub mod best;
pub mod find_spacial_limits;
pub mod organism;

/// Represents a collection of `Organism` instances within the world.
// Collection wrapper providing convenience methods over a vector of Organism instances.
#[derive(Debug, Clone)]
pub struct Organisms {
    organisms: Vec<Arc<Organism>>,
}

impl Organisms {
    /// Returns an iterator over the organisms.
    pub fn iter(&self) -> std::slice::Iter<'_, Arc<Organism>> {
        self.organisms.iter()
    }

    /// Returns the number of organisms.
    pub fn len(&self) -> usize {
        self.organisms.len()
    }

    /// Returns true if the collection contains no organisms.
    pub fn is_empty(&self) -> bool {
        self.organisms.is_empty()
    }

    // Note: no mutable iterator needed since interior mutability
    // pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Arc<Organism>> { ... }

    /// Removes all organisms that have been marked as dead.
    ///
    /// This performs an in-place, linear `retain` scan, so the cost is
    /// O(live + dead) and no extra allocation is required.
    pub fn retain_live(&mut self) {
        self.organisms.retain(|o| !o.is_dead());
    }

    /// Adds a batch of organisms to the collection.
    pub fn extend(&mut self, mut others: Vec<Arc<Organism>>) {
        self.organisms.append(&mut others);
    }

    /// Adds a single organism to the collection.
    pub fn push(&mut self, organism: Arc<Organism>) {
        self.organisms.push(organism);
    }

    /// Creates an empty `Organisms` collection.
    pub fn new_empty() -> Self {
        Self {
            organisms: Vec::new(),
        }
    }

    /// Creates an empty `Organisms` collection with space for `capacity` organisms.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            organisms: Vec::with_capacity(capacity),
        }
    }

    /// Returns the capacity of the underlying vector.
    pub fn capacity(&self) -> usize {
        self.organisms.capacity()
    }

    /// Creates a new `Organisms` collection from a vector of `Arc<Organism>`.
    pub fn new_from_arc_vec(organisms: Vec<Arc<Organism>>) -> Self {
        Self { organisms }
    }

    /// Consumes the collection and returns the underlying vector.
    pub fn into_inner(self) -> Vec<Arc<Organism>> {
        self.organisms
    }
}

impl IntoIterator for Organisms {
    type Item = Arc<Organism>;
    type IntoIter = std::vec::IntoIter<Arc<Organism>>;

    fn into_iter(self) -> Self::IntoIter {
        self.organisms.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Organisms;

    use crate::world::organisms::Organism;

    use std::sync::Arc;

    impl Organisms {
        // Note: get_organisms was moved to the main impl block

        /// Creates a new `Organisms` instance directly from a vector of phenotypes.
        /// This is intended for testing purposes only.
        pub(crate) fn new_from_phenotypes(phenotypes: Vec<crate::phenotype::Phenotype>) -> Self {
            Self {
                organisms: phenotypes
                    .into_iter()
                    .map(|p| Arc::new(Organism::new(Arc::new(p), 0, (None, None))))
                    .collect(),
            }
        }

        pub fn new_from_organisms(organisms: Vec<Organism>) -> Self {
            Self {
                organisms: organisms.into_iter().map(Arc::new).collect(),
            }
        }
    }
}
