//! Organism entry in a region.
//!
//! A lightweight struct containing just the information needed for
//! region processing (ID, age, score).

/// Entry representing an organism in a region.
///
/// Contains only the data needed for sorting and reproduction selection,
/// not the full organism data.
#[derive(Debug, Clone)]
pub struct OrganismEntry {
    /// Organism ID (for sending messages)
    id: u64,
    /// Age (for sorting tie-breaker)
    age: usize,
    /// Fitness score (for sorting and capacity calculation)
    score: Option<f64>,
}

impl OrganismEntry {
    /// Creates a new organism entry.
    pub fn new(id: u64, age: usize, score: Option<f64>) -> Self {
        Self { id, age, score }
    }

    /// Returns the organism ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the organism's age.
    pub fn age(&self) -> usize {
        self.age
    }

    /// Returns the organism's score.
    pub fn score(&self) -> Option<f64> {
        self.score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_values_when_new_then_entry_created() {
        let entry = OrganismEntry::new(42, 5, Some(1.5));
        assert_eq!(entry.id(), 42);
        assert_eq!(entry.age(), 5);
        assert_eq!(entry.score(), Some(1.5));
    }

    #[test]
    fn given_none_score_when_new_then_score_is_none() {
        let entry = OrganismEntry::new(1, 0, None);
        assert_eq!(entry.score(), None);
    }
}
