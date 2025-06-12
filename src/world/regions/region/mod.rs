pub mod reset;

use std::rc::Rc;

use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Region {
    _min_score: Option<f64>,
    _carrying_capacity: Option<usize>,
    organisms: Vec<Rc<Phenotype>>, // Renamed for consistency, or keep _organisms if preferred internal style
}

impl Region {
    pub fn new() -> Self {
        Self {
            _min_score: None,
            _carrying_capacity: None,
            organisms: Vec::new(),
        }
    }

    /// Adds a phenotype to the region's collection of organisms.
    ///
    /// This method takes a `Rc<Phenotype>` to allow for shared ownership of the
    /// phenotype data, avoiding unnecessary clones.
    pub fn add_phenotype(&mut self, phenotype: Rc<Phenotype>) {
        self.organisms.push(phenotype);
    }

    // Optional: A way to get the number of organisms in the region
    pub fn organism_count(&self) -> usize {
        self.organisms.len()
    }

    // Optional: A way to get a slice of the organisms if needed for read-only access
    pub fn get_organisms(&self) -> &[Rc<Phenotype>] {
        &self.organisms
    }

    // Setter for carrying capacity
    pub fn set_carrying_capacity(&mut self, capacity: Option<usize>) {
        self._carrying_capacity = capacity;
    }

    // Setter for min_score
    pub fn set_min_score(&mut self, score: Option<f64>) {
        self._min_score = score;
    }

    // Getter for carrying capacity
    pub fn carrying_capacity(&self) -> Option<usize> {
        self._carrying_capacity
    }

    // Getter for min_score
    pub fn min_score(&self) -> Option<f64> {
        self._min_score
    }
}

// Implementing Default for convenience if Region::new() with defaults is common
impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}
