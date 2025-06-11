use crate::Phenotype;

#[derive(Debug, Clone)]
pub struct Region {
    // Consider if these fields should be public or have getters/setters if needed externally
    organisms: Vec<Phenotype>, // Renamed for consistency, or keep _organisms if preferred internal style
}

impl Region {
    pub fn new() -> Self {
        Self {
            organisms: Vec::new(),
        }
    }

    // Method to add a phenotype to the region.
    // Takes ownership of the phenotype, assuming phenotypes are cloned before being added.
    pub fn add_phenotype(&mut self, phenotype: Phenotype) {
        self.organisms.push(phenotype);
    }

    // Optional: A way to get the number of organisms in the region
    pub fn organism_count(&self) -> usize {
        self.organisms.len()
    }

    // Optional: A way to get a slice of the organisms if needed for read-only access
    pub fn get_organisms(&self) -> &[Phenotype] {
        &self.organisms
    }
}

// Implementing Default for convenience if Region::new() with defaults is common
impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}
