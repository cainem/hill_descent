//! Single region containing organisms.

use super::OrganismEntry;

/// A single spatial subdivision that groups organisms.
///
/// Regions are ephemeral - rebuilt each training run from organism region keys.
#[derive(Debug, Clone)]
pub struct Region {
    /// Organisms in this region
    organisms: Vec<OrganismEntry>,
    /// Minimum fitness score in this region
    min_score: Option<f64>,
    /// Carrying capacity (calculated from relative fitness)
    carrying_capacity: Option<usize>,
}

impl Region {
    /// Maximum number of reproduction passes when population is low.
    pub const REPRODUCTION_FACTOR: usize = 10;

    /// Creates a new empty region.
    pub fn new() -> Self {
        Self {
            organisms: Vec::new(),
            min_score: None,
            carrying_capacity: None,
        }
    }

    /// Adds an organism entry to the region.
    ///
    /// Also updates the min_score if the organism has a lower score.
    pub fn add_organism(&mut self, entry: OrganismEntry) {
        if let Some(score) = entry.score() {
            match self.min_score {
                Some(current_min) if score < current_min => {
                    self.min_score = Some(score);
                }
                None => {
                    self.min_score = Some(score);
                }
                _ => {}
            }
        }
        self.organisms.push(entry);
    }

    /// Returns the number of organisms in the region.
    pub fn organism_count(&self) -> usize {
        self.organisms.len()
    }

    /// Clears all organisms from the region.
    pub fn clear_organisms(&mut self) {
        self.organisms.clear();
        self.min_score = None;
    }

    /// Returns true if the region has no organisms.
    pub fn is_empty(&self) -> bool {
        self.organisms.is_empty()
    }

    /// Returns a slice of organisms in the region.
    pub fn organisms(&self) -> &[OrganismEntry] {
        &self.organisms
    }

    /// Returns a mutable slice of organisms in the region.
    pub fn organisms_mut(&mut self) -> &mut Vec<OrganismEntry> {
        &mut self.organisms
    }

    /// Returns the minimum score in this region.
    pub fn min_score(&self) -> Option<f64> {
        self.min_score
    }

    /// Sets the minimum score for this region.
    pub fn set_min_score(&mut self, score: Option<f64>) {
        self.min_score = score;
    }

    /// Returns the carrying capacity of this region.
    pub fn carrying_capacity(&self) -> Option<usize> {
        self.carrying_capacity
    }

    /// Sets the carrying capacity for this region.
    pub fn set_carrying_capacity(&mut self, capacity: usize) {
        self.carrying_capacity = Some(capacity);
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_new_region_when_is_empty_then_returns_true() {
        let region = Region::new();
        assert!(region.is_empty());
        assert_eq!(region.organism_count(), 0);
    }

    #[test]
    fn given_region_when_add_organism_then_count_increases() {
        let mut region = Region::new();
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        assert_eq!(region.organism_count(), 1);
        assert!(!region.is_empty());
    }

    #[test]
    fn given_region_with_organisms_when_clear_then_empty() {
        let mut region = Region::new();
        region.add_organism(OrganismEntry::new(1, 0, Some(1.0)));
        region.add_organism(OrganismEntry::new(2, 0, Some(2.0)));
        region.clear_organisms();
        assert!(region.is_empty());
        assert_eq!(region.min_score(), None);
    }

    #[test]
    fn given_region_when_add_organism_with_lower_score_then_min_score_updated() {
        let mut region = Region::new();
        region.add_organism(OrganismEntry::new(1, 0, Some(5.0)));
        assert_eq!(region.min_score(), Some(5.0));
        region.add_organism(OrganismEntry::new(2, 0, Some(3.0)));
        assert_eq!(region.min_score(), Some(3.0));
        region.add_organism(OrganismEntry::new(3, 0, Some(7.0)));
        assert_eq!(region.min_score(), Some(3.0)); // Still 3.0
    }

    #[test]
    fn given_region_when_set_carrying_capacity_then_capacity_set() {
        let mut region = Region::new();
        assert_eq!(region.carrying_capacity(), None);
        region.set_carrying_capacity(10);
        assert_eq!(region.carrying_capacity(), Some(10));
    }
}
