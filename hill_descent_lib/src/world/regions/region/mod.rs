mod execute_reproduction_passes;
mod reproduce;

use std::rc::Rc;

use crate::world::organisms::organism::Organism;

#[derive(Debug, Clone)]
// Represents a single spatial subdivision that groups organisms and stores metadata like minimum score and carrying capacity.
pub struct Region {
    min_score: Option<f64>,
    carrying_capacity: Option<usize>,
    organisms: Vec<Rc<Organism>>,
}

impl Region {
    /// Maximum number of reproduction passes when population is low relative to carrying capacity.
    /// This allows rapid population growth when a region has few organisms but high carrying capacity.
    const REPRODUCTION_FACTOR: usize = 3;
}

impl Region {
    pub fn new() -> Self {
        Self {
            min_score: None,
            carrying_capacity: None,
            organisms: Vec::new(),
        }
    }

    /// Adds an organism to the region.
    ///
    /// Uses `Rc<Organism>` to allow shared ownership without unnecessary clones.
    pub fn add_organism(&mut self, organism: Rc<Organism>) {
        // kept name for minimal call-site changes; now stores organisms
        self.organisms.push(organism);
    }

    // Optional: A way to get the number of organisms in the region
    pub fn organism_count(&self) -> usize {
        self.organisms.len()
    }

    /// Removes all organisms from the region.
    pub fn clear_organisms(&mut self) {
        self.organisms.clear();
    }

    /// Returns true if this region currently has no organisms.
    pub fn is_empty(&self) -> bool {
        self.organisms.is_empty()
    }

    pub fn organisms(&self) -> &[Rc<Organism>] {
        &self.organisms
    }

    // Setter for carrying capacity
    pub fn set_carrying_capacity(&mut self, capacity: Option<usize>) {
        self.carrying_capacity = capacity;
    }

    // Setter for min_score
    pub fn set_min_score(&mut self, score: Option<f64>) {
        self.min_score = score;
    }

    // Getter for carrying capacity
    pub fn carrying_capacity(&self) -> Option<usize> {
        self.carrying_capacity
    }

    // Getter for min_score
    pub fn min_score(&self) -> Option<f64> {
        self.min_score
    }

    /// Removes any organisms that have been marked as dead from this region.
    pub fn retain_live(&mut self) {
        self.organisms.retain(|o| !o.is_dead());
    }
}

// Implementing Default for convenience if Region::new() with defaults is common
impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}
