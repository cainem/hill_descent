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

    // Method to add a phenotype to the region.
    // Takes ownership of the phenotype, assuming phenotypes are cloned before being added.
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
}

// Implementing Default for convenience if Region::new() with defaults is common
impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}
