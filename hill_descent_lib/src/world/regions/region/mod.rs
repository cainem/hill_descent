mod execute_reproduction_passes;
mod execute_single_reproduction_pass;
mod pair_organisms_for_reproduction;
mod perform_sexual_reproduction;
mod process_region;
pub mod region_key;
mod reproduce;

use std::sync::Arc;

use crate::world::organisms::organism::Organism;

#[derive(Debug, Clone)]
// Represents a single spatial subdivision that groups organisms and stores metadata like minimum score and carrying capacity.
pub struct Region {
    min_score: Option<f64>,
    carrying_capacity: Option<usize>,
    organisms: Vec<Arc<Organism>>,
}

impl Region {
    /// Maximum number of reproduction passes when population is low relative to carrying capacity.
    /// This allows rapid population growth when a region has few organisms but high carrying capacity.
    const REPRODUCTION_FACTOR: usize = 10;
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
    /// Uses `Arc<Organism>` to allow shared ownership without unnecessary clones.
    /// Also updates the region's min_score if the organism has a score
    /// that is lower than the current min_score.
    pub fn add_organism(&mut self, organism: Arc<Organism>) {
        // Update min_score if this organism has a score
        if let Some(score) = organism.score() {
            match self.min_score {
                Some(current_min) => {
                    if score < current_min {
                        self.min_score = Some(score);
                    }
                }
                None => {
                    self.min_score = Some(score);
                }
            }
        }

        // Add organism to the region
        self.organisms.push(organism);
    }

    // Optional: A way to get the number of organisms in the region
    pub fn organism_count(&self) -> usize {
        self.organisms.len()
    }

    /// Removes all organisms from the region.
    ///
    /// Vec capacity is preserved for reuse in subsequent generations.
    pub fn clear_organisms(&mut self) {
        self.organisms.clear();
    }

    /// Returns true if this region currently has no organisms.
    pub fn is_empty(&self) -> bool {
        self.organisms.is_empty()
    }

    /// Takes all organisms out of the region, leaving it empty.
    ///
    /// This avoids cloning Arcs when moving organisms between collections.
    pub fn take_organisms(&mut self) -> Vec<Arc<Organism>> {
        std::mem::take(&mut self.organisms)
    }

    pub fn organisms(&self) -> &[Arc<Organism>] {
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
