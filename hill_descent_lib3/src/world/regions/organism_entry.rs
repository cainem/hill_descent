//! Organism entry in a region.
//!
//! A lightweight struct containing just the information needed for
//! region processing (ID, age, score).

use std::cmp::Ordering;

/// Entry representing an organism in a region.
///
/// Contains only the data needed for sorting and reproduction selection,
/// not the full organism data.
///
/// # Ordering
///
/// Entries are ordered by:
/// 1. Score (ascending - lower/better scores first)
/// 2. Age (descending - older organisms first for tie-breaking)
///
/// Entries with `None` scores are considered worse than any scored entry
/// and sorted to the end.
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

impl PartialEq for OrganismEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for OrganismEntry {}

impl PartialOrd for OrganismEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrganismEntry {
    /// Compares entries for ordering.
    ///
    /// Ordering priority:
    /// 1. Entries with scores come before entries without scores
    /// 2. Lower scores come before higher scores (ascending)
    /// 3. For equal scores, older organisms come first (descending age)
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.score, other.score) {
            // Both have scores: compare by score (ascending), then by age (descending)
            (Some(self_score), Some(other_score)) => {
                match self_score.partial_cmp(&other_score) {
                    Some(Ordering::Equal) | None => {
                        // For equal scores or NaN, older organisms (higher age) come first
                        other.age.cmp(&self.age)
                    }
                    Some(ordering) => ordering,
                }
            }
            // Self has score, other doesn't: self comes first
            (Some(_), None) => Ordering::Less,
            // Self doesn't have score, other does: other comes first
            (None, Some(_)) => Ordering::Greater,
            // Neither has score: compare by age (older first)
            (None, None) => other.age.cmp(&self.age),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Basic construction tests ====================

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

    // ==================== Equality tests ====================

    #[test]
    fn given_same_id_when_eq_then_returns_true() {
        let entry1 = OrganismEntry::new(42, 5, Some(1.5));
        let entry2 = OrganismEntry::new(42, 10, Some(2.5)); // Different age and score
        assert_eq!(entry1, entry2);
    }

    #[test]
    fn given_different_id_when_eq_then_returns_false() {
        let entry1 = OrganismEntry::new(42, 5, Some(1.5));
        let entry2 = OrganismEntry::new(43, 5, Some(1.5)); // Same age and score, different id
        assert_ne!(entry1, entry2);
    }

    // ==================== Ordering by score tests ====================

    #[test]
    fn given_entries_with_scores_when_sorted_then_ordered_by_score_ascending() {
        let mut entries = [
            OrganismEntry::new(1, 0, Some(3.0)),
            OrganismEntry::new(2, 0, Some(1.0)),
            OrganismEntry::new(3, 0, Some(2.0)),
        ];

        entries.sort();

        assert_eq!(entries[0].id(), 2); // score 1.0
        assert_eq!(entries[1].id(), 3); // score 2.0
        assert_eq!(entries[2].id(), 1); // score 3.0
    }

    // ==================== Ordering by age tie-breaker tests ====================

    #[test]
    fn given_entries_with_same_score_when_sorted_then_older_first() {
        let mut entries = [
            OrganismEntry::new(1, 5, Some(1.0)),  // younger
            OrganismEntry::new(2, 10, Some(1.0)), // older
            OrganismEntry::new(3, 7, Some(1.0)),  // middle
        ];

        entries.sort();

        assert_eq!(entries[0].id(), 2); // age 10 (oldest)
        assert_eq!(entries[1].id(), 3); // age 7
        assert_eq!(entries[2].id(), 1); // age 5 (youngest)
    }

    // ==================== None score handling tests ====================

    #[test]
    fn given_entries_with_and_without_scores_when_sorted_then_scored_first() {
        let mut entries = [
            OrganismEntry::new(1, 0, None),
            OrganismEntry::new(2, 0, Some(5.0)),
            OrganismEntry::new(3, 0, None),
            OrganismEntry::new(4, 0, Some(2.0)),
        ];

        entries.sort();

        // Scored entries come first, ordered by score
        assert_eq!(entries[0].id(), 4); // score 2.0
        assert_eq!(entries[1].id(), 2); // score 5.0
        // Unscored entries come last
        assert!(entries[2].score().is_none());
        assert!(entries[3].score().is_none());
    }

    #[test]
    fn given_entries_without_scores_when_sorted_then_older_first() {
        let mut entries = [
            OrganismEntry::new(1, 3, None),
            OrganismEntry::new(2, 7, None),
            OrganismEntry::new(3, 5, None),
        ];

        entries.sort();

        assert_eq!(entries[0].id(), 2); // age 7 (oldest)
        assert_eq!(entries[1].id(), 3); // age 5
        assert_eq!(entries[2].id(), 1); // age 3 (youngest)
    }

    // ==================== Mixed scenario tests ====================

    #[test]
    fn given_mixed_entries_when_sorted_then_correct_order() {
        let mut entries = [
            OrganismEntry::new(1, 2, Some(5.0)),
            OrganismEntry::new(2, 5, None),
            OrganismEntry::new(3, 8, Some(3.0)),
            OrganismEntry::new(4, 3, Some(3.0)), // Same score as 3, but younger
            OrganismEntry::new(5, 1, None),
        ];

        entries.sort();

        // First: scored entries ordered by score (ascending), then age (descending)
        assert_eq!(entries[0].id(), 3); // score 3.0, age 8
        assert_eq!(entries[1].id(), 4); // score 3.0, age 3
        assert_eq!(entries[2].id(), 1); // score 5.0
        // Then: unscored entries ordered by age (descending)
        assert_eq!(entries[3].id(), 2); // no score, age 5
        assert_eq!(entries[4].id(), 5); // no score, age 1
    }

    // ==================== Comparison method tests ====================

    #[test]
    fn given_lower_score_when_cmp_then_less() {
        let entry1 = OrganismEntry::new(1, 0, Some(1.0));
        let entry2 = OrganismEntry::new(2, 0, Some(2.0));

        assert_eq!(entry1.cmp(&entry2), Ordering::Less);
        assert_eq!(entry2.cmp(&entry1), Ordering::Greater);
    }

    #[test]
    fn given_equal_score_and_older_when_cmp_then_less() {
        let entry1 = OrganismEntry::new(1, 10, Some(1.0)); // older
        let entry2 = OrganismEntry::new(2, 5, Some(1.0)); // younger

        assert_eq!(entry1.cmp(&entry2), Ordering::Less); // older comes first
        assert_eq!(entry2.cmp(&entry1), Ordering::Greater);
    }

    #[test]
    fn given_score_vs_none_when_cmp_then_scored_is_less() {
        let entry1 = OrganismEntry::new(1, 0, Some(100.0)); // Even high score
        let entry2 = OrganismEntry::new(2, 0, None);

        assert_eq!(entry1.cmp(&entry2), Ordering::Less); // scored comes first
        assert_eq!(entry2.cmp(&entry1), Ordering::Greater);
    }
}
